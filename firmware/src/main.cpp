// FGA iOS 브리지 펌웨어
//
// 역할 두 가지를 한 보드에서 동시에 수행한다.
//   1) BLE HID 절대 좌표 마우스: iPhone의 AssistiveTouch가 포인터로 인식한다.
//   2) 명령 수신: USB 시리얼 또는 BLE 커스텀 서비스(Nordic UART 호환)로 텍스트 명령을 받는다.
//
// 좌표는 항상 0..32767 (화면 왼쪽 위가 0,0).
//
// 명령 (한 줄에 하나, 줄바꿈으로 끝):
//   PING
//   STAT
//   MOVE x y
//   TAP x y [hold_ms]
//   DOWN x y
//   UP
//   SWIPE x1 y1 x2 y2 [duration_ms]
//   SET settle|hold|step|after value     (ms 단위 타이밍 조정)
// 응답: "OK ..." 또는 "ERR ..."

#include <Arduino.h>
#include <NimBLEDevice.h>
#include <NimBLEHIDDevice.h>

static const char* DEVICE_NAME = "FGA Bridge";

// Nordic UART Service UUID. iOS 앱(CoreBluetooth)이 이 서비스로 명령을 쓴다.
static const char* CMD_SERVICE_UUID = "6E400001-B5A3-F393-E0A9-E50E24DCCA9E";
static const char* CMD_RX_UUID      = "6E400002-B5A3-F393-E0A9-E50E24DCCA9E"; // 앱 -> 보드
static const char* CMD_TX_UUID      = "6E400003-B5A3-F393-E0A9-E50E24DCCA9E"; // 보드 -> 앱

static const uint8_t REPORT_ID = 1;
static const int32_t COORD_MAX = 32767;

// 절대 좌표 마우스: 버튼 3개 + X/Y 16비트 절대값.
static const uint8_t REPORT_MAP[] = {
    0x05, 0x01,        // Usage Page (Generic Desktop)
    0x09, 0x02,        // Usage (Mouse)
    0xA1, 0x01,        // Collection (Application)
    0x85, REPORT_ID,   //   Report ID
    0x09, 0x01,        //   Usage (Pointer)
    0xA1, 0x00,        //   Collection (Physical)
    0x05, 0x09,        //     Usage Page (Button)
    0x19, 0x01,        //     Usage Minimum (1)
    0x29, 0x03,        //     Usage Maximum (3)
    0x15, 0x00,        //     Logical Minimum (0)
    0x25, 0x01,        //     Logical Maximum (1)
    0x95, 0x03,        //     Report Count (3)
    0x75, 0x01,        //     Report Size (1)
    0x81, 0x02,        //     Input (Data, Variable, Absolute)
    0x95, 0x01,        //     Report Count (1)
    0x75, 0x05,        //     Report Size (5)
    0x81, 0x03,        //     Input (Constant) - 패딩
    0x05, 0x01,        //     Usage Page (Generic Desktop)
    0x09, 0x30,        //     Usage (X)
    0x09, 0x31,        //     Usage (Y)
    0x15, 0x00,        //     Logical Minimum (0)
    0x26, 0xFF, 0x7F,  //     Logical Maximum (32767)
    0x75, 0x10,        //     Report Size (16)
    0x95, 0x02,        //     Report Count (2)
    0x81, 0x02,        //     Input (Data, Variable, Absolute)
    0xC0,              //   End Collection
    0xC0               // End Collection
};

// 타이밍 (ms). SET 명령으로 실행 중에 바꿀 수 있다.
// settle: 이동 후 버튼을 누르기 전 대기. iOS는 버튼 상태가 바뀌는 리포트의 좌표를 무시하므로
//         먼저 이동만 보내고 기다렸다가 같은 좌표에서 눌러야 한다.
static uint32_t settleMs = 60;
static uint32_t holdMs   = 50;  // 탭에서 누르고 있는 시간
static uint32_t stepMs   = 12;  // 스와이프 중 이동 리포트 간격
// after: 버튼을 뗀 뒤 다음 명령까지 대기. 실측(iPhone 17, iOS 27)에서 30 ms면 뗀 것이 늦게
//        처리돼 탭이 다음 위치로 끌려가고, 60 ms 이상이면 정확했다. 여유를 두어 100.
static uint32_t afterMs  = 100;
static uint32_t easeMode = 1;  // 스와이프 감속 방식 (doSwipe 참고)

// iOS는 가로 화면에서 x가 세로 화면 폭을 조금 넘는 구간(iPhone 17: 46.0%~50.0%)의 포인터를
// 구간 왼쪽 경계로 옮겨 버린다. 이 구간에 들어오는 x는 가까운 바깥쪽으로 밀어낸다.
// AVOID lo hi 로 설정하고, AVOID 0 0 이면 끈다. 기기와 화면 방향마다 값이 다르다.
static int32_t avoidLo = 0;
static int32_t avoidHi = 0;

static NimBLEHIDDevice*      hid      = nullptr;
static NimBLECharacteristic* inputRpt = nullptr;
static NimBLECharacteristic* cmdTx    = nullptr;

static volatile bool connected = false;
static volatile bool encrypted = false;
static volatile int  hidSubscribed = -1;  // -1: 이번 부팅 후 구독 이벤트 없음, 0: 해제, 1: 알림 구독

static uint16_t curX = COORD_MAX / 2;
static uint16_t curY = COORD_MAX / 2;
static bool     buttonDown = false;

// BLE로 들어온 명령은 BLE 스택 태스크에서 도착하므로 큐에 넣고 loop()에서 처리한다.
static const size_t LINE_MAX_LEN = 96;
struct CmdLine {
    char text[LINE_MAX_LEN];
    bool fromBle;
};
static QueueHandle_t cmdQueue = nullptr;

static void reply(bool toBle, const char* msg) {
    // 블루투스 응답을 먼저 보낸다. USB 시리얼은 받는 쪽이 없으면 쓰기가 막힐 수 있어서,
    // 순서가 반대면 앱이 응답을 몇 초 늦게 받는다.
    if (toBle && cmdTx != nullptr && connected) {
        cmdTx->setValue((const uint8_t*)msg, strlen(msg));
        cmdTx->notify();
    }
    Serial.println(msg);
}

static bool sendReport(uint16_t x, uint16_t y, bool down) {
    if (!connected || inputRpt == nullptr) return false;
    uint8_t buf[5] = {
        (uint8_t)(down ? 0x01 : 0x00),
        (uint8_t)(x & 0xFF), (uint8_t)(x >> 8),
        (uint8_t)(y & 0xFF), (uint8_t)(y >> 8),
    };
    inputRpt->setValue(buf, sizeof(buf));
    inputRpt->notify();
    curX = x;
    curY = y;
    buttonDown = down;
    return true;
}

static uint16_t clampCoord(long v) {
    if (v < 0) return 0;
    if (v > COORD_MAX) return (uint16_t)COORD_MAX;
    return (uint16_t)v;
}

static uint16_t fixX(long v) {
    uint16_t x = clampCoord(v);
    if (avoidHi > avoidLo && x > avoidLo && x < avoidHi) {
        return (uint16_t)((x - avoidLo) < (avoidHi - x) ? avoidLo : avoidHi);
    }
    return x;
}

static bool doMove(uint16_t x, uint16_t y) {
    return sendReport(x, y, buttonDown);
}

static bool doDown(uint16_t x, uint16_t y) {
    if (!sendReport(x, y, false)) return false;  // 먼저 이동만
    delay(settleMs);
    return sendReport(x, y, true);               // 같은 좌표에서 누름
}

static bool doUp() {
    bool ok = sendReport(curX, curY, false);
    delay(afterMs);
    return ok;
}

static bool doTap(uint16_t x, uint16_t y, uint32_t hold) {
    if (!doDown(x, y)) return false;
    delay(hold);
    return doUp();
}

static bool doSwipe(uint16_t x1, uint16_t y1, uint16_t x2, uint16_t y2, uint32_t durationMs) {
    if (!doDown(x1, y1)) return false;
    delay(holdMs);
    uint32_t steps = durationMs / (stepMs == 0 ? 1 : stepMs);
    if (steps < 2) steps = 2;
    for (uint32_t i = 1; i <= steps; i++) {
        // ease: 0 = 등속, 1 = 끝으로 갈수록 감속(2차), 2 = 더 강한 감속(3차).
        // 등속으로 끌다 떼면 iOS가 남은 속도로 목록을 더 굴리므로, 끝에서 속도를 0에 가깝게 만든다.
        float t = (float)i / (float)steps;
        float f = t;
        if (easeMode == 1) f = 1.0f - (1.0f - t) * (1.0f - t);
        else if (easeMode >= 2) f = 1.0f - (1.0f - t) * (1.0f - t) * (1.0f - t);
        long x = (long)x1 + lroundf(((float)x2 - (float)x1) * f);
        long y = (long)y1 + lroundf(((float)y2 - (float)y1) * f);
        if (!sendReport(fixX(x), clampCoord(y), true)) return false;
        delay(stepMs);
    }
    delay(holdMs);  // 끝 지점에서 잠깐 멈춰 관성 스크롤(플링)을 줄인다
    return doUp();
}

static void handleLine(char* line, bool fromBle) {
    char* save = nullptr;
    char* cmd = strtok_r(line, " \t\r\n", &save);
    if (cmd == nullptr) return;

    long args[5];
    int argc = 0;
    char* word = nullptr;  // SET 명령의 이름
    if (strcasecmp(cmd, "SET") == 0) {
        word = strtok_r(nullptr, " \t\r\n", &save);
    }
    for (char* t = strtok_r(nullptr, " \t\r\n", &save); t != nullptr && argc < 5;
         t = strtok_r(nullptr, " \t\r\n", &save)) {
        args[argc++] = strtol(t, nullptr, 10);
    }

    if (strcasecmp(cmd, "PING") == 0) {
        reply(fromBle, "OK PONG");
        return;
    }
    if (strcasecmp(cmd, "STAT") == 0) {
        char buf[160];
        snprintf(buf, sizeof(buf),
                 "OK connected=%d encrypted=%d hidsub=%d x=%u y=%u down=%d avoid=%ld-%ld settle=%lu hold=%lu step=%lu after=%lu",
                 connected ? 1 : 0, encrypted ? 1 : 0, (int)hidSubscribed, curX, curY,
                 buttonDown ? 1 : 0, (long)avoidLo, (long)avoidHi,
                 (unsigned long)settleMs, (unsigned long)holdMs, (unsigned long)stepMs,
                 (unsigned long)afterMs);
        reply(fromBle, buf);
        return;
    }
    if (strcasecmp(cmd, "SET") == 0) {
        if (word == nullptr || argc < 1 || args[0] < 0 || args[0] > 5000) {
            reply(fromBle, "ERR usage: SET settle|hold|step|after|ease value");
            return;
        }
        uint32_t v = (uint32_t)args[0];
        if (strcasecmp(word, "settle") == 0) settleMs = v;
        else if (strcasecmp(word, "hold") == 0) holdMs = v;
        else if (strcasecmp(word, "step") == 0) stepMs = v;
        else if (strcasecmp(word, "after") == 0) afterMs = v;
        else if (strcasecmp(word, "ease") == 0) easeMode = v;
        else { reply(fromBle, "ERR unknown setting"); return; }
        reply(fromBle, "OK");
        return;
    }

    if (strcasecmp(cmd, "AVOID") == 0) {
        if (argc < 2 || args[0] < 0 || args[1] > COORD_MAX || args[1] < args[0]) {
            reply(fromBle, "ERR usage: AVOID lo hi (0 0 = off)");
            return;
        }
        avoidLo = (int32_t)args[0];
        avoidHi = (int32_t)args[1];
        reply(fromBle, "OK");
        return;
    }

    if (!connected) {
        reply(fromBle, "ERR not connected");
        return;
    }

    bool ok = false;
    if (strcasecmp(cmd, "MOVE") == 0 && argc >= 2) {
        ok = doMove(fixX(args[0]), clampCoord(args[1]));
    } else if (strcasecmp(cmd, "TAP") == 0 && argc >= 2) {
        ok = doTap(fixX(args[0]), clampCoord(args[1]),
                   argc >= 3 && args[2] >= 0 ? (uint32_t)args[2] : holdMs);
    } else if (strcasecmp(cmd, "DOWN") == 0 && argc >= 2) {
        ok = doDown(fixX(args[0]), clampCoord(args[1]));
    } else if (strcasecmp(cmd, "UP") == 0) {
        ok = doUp();
    } else if (strcasecmp(cmd, "SWIPE") == 0 && argc >= 4) {
        ok = doSwipe(fixX(args[0]), clampCoord(args[1]), fixX(args[2]),
                     clampCoord(args[3]), argc >= 5 && args[4] > 0 ? (uint32_t)args[4] : 400);
    } else {
        reply(fromBle, "ERR unknown command or missing arguments");
        return;
    }
    reply(fromBle, ok ? "OK" : "ERR send failed");
}

class ServerCallbacks : public NimBLEServerCallbacks {
    void onConnect(NimBLEServer* server, NimBLEConnInfo& info) override {
        connected = true;
        encrypted = false;
        // 짧은 연결 간격을 요청해 입력 지연을 줄인다 (11.25~15 ms, iOS 허용 범위).
        server->updateConnParams(info.getConnHandle(), 9, 12, 0, 400);
        Serial.println("# connected");
    }
    void onDisconnect(NimBLEServer* server, NimBLEConnInfo& info, int reason) override {
        connected = false;
        encrypted = false;
        buttonDown = false;
        Serial.printf("# disconnected reason=%d\n", reason);
        NimBLEDevice::startAdvertising();
    }
    void onAuthenticationComplete(NimBLEConnInfo& info) override {
        encrypted = info.isEncrypted();
        Serial.printf("# auth complete encrypted=%d bonded=%d\n", info.isEncrypted() ? 1 : 0,
                      info.isBonded() ? 1 : 0);
    }
};

class CmdCallbacks : public NimBLECharacteristicCallbacks {
    void onWrite(NimBLECharacteristic* chr, NimBLEConnInfo& info) override {
        std::string v = chr->getValue();
        CmdLine item;
        size_t n = v.size() < LINE_MAX_LEN - 1 ? v.size() : LINE_MAX_LEN - 1;
        memcpy(item.text, v.data(), n);
        item.text[n] = '\0';
        item.fromBle = true;
        xQueueSend(cmdQueue, &item, 0);
    }
};

// iPhone이 HID 입력 리포트 알림을 구독했는지 추적한다. 구독하지 않으면 iOS가 이 기기를
// 포인터로 쓰지 않는다는 뜻이다.
class InputCallbacks : public NimBLECharacteristicCallbacks {
    void onSubscribe(NimBLECharacteristic* chr, NimBLEConnInfo& info, uint16_t subValue) override {
        hidSubscribed = subValue;
        Serial.printf("# hid input subscribe=%u\n", subValue);
    }
};

static ServerCallbacks serverCallbacks;
static InputCallbacks  inputCallbacks;
static CmdCallbacks    cmdCallbacks;

void setup() {
    Serial.begin(115200);
#if ARDUINO_USB_CDC_ON_BOOT
    // 보드가 PC에 꽂혀 있어도 포트를 읽는 프로그램이 없으면 출력이 쌓여 쓰기가 막힌다.
    // 기다리지 않고 버리게 해서, 시리얼 출력이 블루투스 명령 처리를 늦추지 않게 한다.
    Serial.setTxTimeoutMs(0);
#endif
    cmdQueue = xQueueCreate(16, sizeof(CmdLine));

    NimBLEDevice::init(DEVICE_NAME);
    NimBLEDevice::setPower(9);
    // 본딩 사용, 패스키 없음(Just Works). iOS는 HID 기기에 암호화 연결을 요구한다.
    NimBLEDevice::setSecurityAuth(true, false, true);
    NimBLEDevice::setSecurityIOCap(BLE_HS_IO_NO_INPUT_OUTPUT);

    NimBLEServer* server = NimBLEDevice::createServer();
    server->setCallbacks(&serverCallbacks, false);

    hid = new NimBLEHIDDevice(server);
    inputRpt = hid->getInputReport(REPORT_ID);
    inputRpt->setCallbacks(&inputCallbacks);
    hid->setManufacturer("FGA");
    hid->setPnp(0x02, 0x303A, 0x4001, 0x0100);  // USB-IF 출처, Espressif VID
    hid->setHidInfo(0x00, 0x01);
    hid->setReportMap((uint8_t*)REPORT_MAP, sizeof(REPORT_MAP));
    hid->setBatteryLevel(100);

    NimBLEService* cmdService = server->createService(CMD_SERVICE_UUID);
    NimBLECharacteristic* cmdRx = cmdService->createCharacteristic(
        CMD_RX_UUID, NIMBLE_PROPERTY::WRITE | NIMBLE_PROPERTY::WRITE_NR);
    cmdRx->setCallbacks(&cmdCallbacks);
    cmdTx = cmdService->createCharacteristic(CMD_TX_UUID, NIMBLE_PROPERTY::NOTIFY);

    hid->startServices();
    cmdService->start();

    NimBLEAdvertising* adv = NimBLEDevice::getAdvertising();
    adv->setAppearance(HID_MOUSE);
    adv->addServiceUUID(hid->getHidService()->getUUID());
    adv->setName(DEVICE_NAME);
    adv->enableScanResponse(true);
    adv->start();

    Serial.println("# FGA Bridge ready");
}

void loop() {
    // USB 시리얼 명령
    static char   serialBuf[LINE_MAX_LEN];
    static size_t serialLen = 0;
    while (Serial.available() > 0) {
        char c = (char)Serial.read();
        if (c == '\n' || c == '\r') {
            if (serialLen > 0) {
                serialBuf[serialLen] = '\0';
                handleLine(serialBuf, false);
                serialLen = 0;
            }
        } else if (serialLen < LINE_MAX_LEN - 1) {
            serialBuf[serialLen++] = c;
        }
    }

    // BLE 명령
    CmdLine item;
    while (xQueueReceive(cmdQueue, &item, 0) == pdTRUE) {
        handleLine(item.text, item.fromBle);
    }

    delay(1);
}
