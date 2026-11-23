# FGA-apple

[FGA-Modified](https://github.com/puruuun/FGA-Modified)를 탈옥하지 않은 iPhone/iPad에서 돌리기 위한 저장소입니다.

- 화면 읽기: iPhone 화면을 캡처해서 받습니다.
- 터치: ESP32가 블루투스 마우스로 붙어 AssistiveTouch를 통해 탭·스와이프를 넣습니다.
- 게임 로직: FGA-Modified의 `libautomata`·`scripts`(지금은 Kotlin, 이후 Rust 코어)를 그대로 씁니다.

최종 목표는 **iPhone과 ESP32 둘만으로** 주회가 도는 것입니다. 그 전 단계로 PC(JVM)에서 스크립트를 돌리는 호스트 버전을 먼저 만듭니다.

## 현재 상태

**실험 단계입니다. 설치해서 바로 쓸 수 있는 앱은 아직 없습니다.**

| 조각 | 상태 |
| :--- | :--- |
| ESP32로 탭·스와이프 넣기 | 동작 |
| FGA의 템플릿과 좌표를 iPhone 화면에 쓰기 | 동작. 수정 없이 실제 주회를 돌렸습니다 |
| PC 호스트 구성 (Mac이 화면을 받고 PC가 스크립트를 실행) | 동작. 전투 1회 단위로 확인 |
| iPhone 앱이 뒤에 있을 때 화면 캡처 | 동작 (iOS 27, 46분까지 확인) |
| iPhone 앱에서 ESP32로 명령 보내기 | 동작 (포인터 이동까지 확인) |
| iPhone 안에서 게임 로직 실행 | 아직 없음 |

필요한 것: iOS 27 이상의 iPhone, ESP32-C3 보드, 그리고 호스트 구성에는 Mac(화면 수신)과 PC가 필요합니다. iPhone 앱을 올리려면 Xcode가 있는 Mac이 필요합니다.

아래 수치는 모두 iPhone 17 한 대에서 잰 것입니다. 다른 기기와 iOS 버전에서는 다를 수 있습니다. 게임 운영 정책에 어긋날 수 있는 자동화 도구이므로 사용에 따르는 책임은 사용자에게 있습니다.

## 폴더

| 폴더 | 내용 |
| :--- | :--- |
| `firmware/` | ESP32-C3 펌웨어. 절대 좌표 BLE 마우스 + 명령 수신(USB 시리얼, BLE). PlatformIO 프로젝트 |
| `mac-capture/` | Mac에서 USB로 iPhone 화면을 받아 오는 도구 (Swift) |
| `tools/` | 측정 스크립트(Python)와 터치 기록 테스트 페이지 서버 |
| `fga/` | FGA-Modified 서브모듈. 읽기만 하고 고치지 않습니다 |
| `host/` | PC(JVM)에서 FGA의 `scripts`를 돌리는 호스트. 화면은 Mac에서 받고 터치는 ESP32로 보냅니다 |
| `ios/FGAProbe/` | iPhone 앱의 백그라운드 화면 캡처를 재는 검증 앱 |
| `configs/` | 호스트 실행 설정 예시 |

받은 뒤 서브모듈을 가져와야 빌드됩니다: `git submodule update --init`

## 펌웨어 명령

좌표는 항상 0~32767이고 화면 왼쪽 위가 (0, 0)입니다. 화면이 가로면 가로 기준으로 따라갑니다.

| 명령 | 동작 |
| :--- | :--- |
| `TAP x y [hold_ms]` | 탭 |
| `SWIPE x1 y1 x2 y2 [ms]` | 끌기. 끝으로 갈수록 감속해 관성 스크롤을 없앱니다 |
| `MOVE x y`, `DOWN x y`, `UP` | 저수준 동작 |
| `AVOID lo hi` | x가 `lo`~`hi` 사이면 가까운 바깥쪽으로 밀어냅니다. `AVOID 0 0`이면 끕니다. 전원을 켤 때마다 다시 보내야 합니다 |
| `SET settle\|hold\|step\|after\|ease 값` | 타이밍 조정 |
| `STAT`, `PING` | 상태 확인 |

## 실측으로 확인한 것 (iPhone 17, iOS 27, ESP32-C3, 2026-11-23)

- **절대 좌표가 보정 없이 맞습니다.** 포인터와 탭 모두 1픽셀 이내입니다.
- **AssistiveTouch가 켜져 있어야** 포인터가 나타나고 탭이 들어갑니다.
- **가로 화면에서 x 46.0%~50.0%(1206~1311픽셀 / 2622) 세로 띠는 누를 수 없습니다.** iOS가 그 구간의 포인터를 띠의 왼쪽 경계로 옮깁니다. 화면 맨 위부터 맨 아래까지 이어지고, 탭도 옮겨진 자리에 찍힙니다. FGO와 Safari에서 같았고 세로 화면에서는 없었습니다. 띠의 왼쪽 경계가 세로 화면 폭(402pt)과 같습니다. `AVOID 15040 16480`으로 피합니다. 다른 기기와 반대 방향 가로에서는 값이 다를 수 있고, 아직 재지 않았습니다.
- **버튼을 뗀 뒤 다음 동작까지 60 ms 이상 필요합니다.** 30 ms면 탭이 다음 위치로 끌려갑니다. 기본값은 100 ms이고, 이때 연속 탭은 약 210 ms에 한 번입니다.
- **등속 스와이프는 관성 스크롤을 일으킵니다.** 끝에서 0.25초를 멈춰도 불규칙하게 130~300pt를 더 굴러갑니다. 감속 스와이프(`ease 1`)는 30번 중 30번 관성이 0이었습니다.
- **스크롤은 손가락 이동보다 조금 덜 움직입니다.** Safari에서는 약 10pt, FGO 서포트 목록에서는 5~12% 정도입니다.
- **보드는 재시작 후 iPhone에 자동으로 다시 붙습니다.**
- **ESP32-C3의 USB 단자는 RTS 신호를 리셋으로 받습니다.** 시리얼 포트를 열 때 RTS를 꺼야 합니다.
- **Mac의 USB 화면 캡처에 포인터가 찍힙니다.** 캡처 시작 후 첫 프레임까지 약 10초가 걸리고, iPhone이 잠기면 멈춘 뒤 저절로 복구되지 않습니다.

## iPhone 앱의 백그라운드 화면 캡처 (iPhone 17, iOS 27.2, 2026-11-23)

`ios/FGAProbe`로 측정했습니다. 무료 Apple ID로 서명한 앱입니다.

- **앱이 뒤에 있고 FGO가 앞에 있는 동안에도 화면 프레임이 계속 들어옵니다.** `UIBackgroundModes`에 `screen-capture`를 넣고 `SCContentSharingPicker`로 전체 화면을 고른 경우입니다. 초당 약 30장이 끊김 없이 들어왔고 메모리는 22~44MB였습니다.
- **이전 버전으로 46분 동안 끊김 없이 받았습니다**(사용자가 중단할 때까지). 프레임만 받는 상태였고 메모리는 23~24MB였습니다.
- **앱에서 블루투스로 보드에 명령을 보낼 수 있습니다.** 보드가 iPhone에 마우스로 붙어 있는 상태에서 `retrieveConnectedPeripherals(withServices:)`로 같은 기기를 찾아 연결했고, `STAT` 응답과 포인터 이동을 확인했습니다. 앱이 뒤에 있는 동안에도 10초 간격의 `PING`에 응답이 왔습니다(`bluetooth-central` 백그라운드 모드).
- **프레임은 항상 세로 방향입니다.** 게임이 가로여도 내용이 90도 돌아간 세로 버퍼로 옵니다.
- **프레임 속도와 픽셀 형식은 지정할 수 없습니다.** iOS에서는 `minimumFrameInterval`과 `pixelFormat`이 막혀 있고 해상도만 정할 수 있습니다.
- **무료 프로파일로 서명한 앱은 기기당 3개까지만 설치됩니다.** 다른 계정으로 설치한 앱도 함께 셉니다.
- **SSH 세션에서는 코드 서명이 실패합니다**(`errSecInternalComponent`). Xcode에서 한 번 빌드해 서명한 뒤 `xcrun devicectl device install app`으로 설치합니다.

## 아직 확인하지 못한 것

- 백그라운드 캡처가 몇 시간 동안 유지되는지, 매칭을 돌릴 때의 메모리와 발열.
- 앱에서 보낸 탭이 실제로 게임을 누르는지. 포인터 이동까지만 확인했습니다.
- 게임 로직을 iPhone에서 돌리는 방법. `scripts`는 Kotlin/JVM이라 iPhone에서 그대로 돌지 않습니다.

## 개발 환경

- 펌웨어: `pio run -d firmware -e c3_usb -t upload`
- 측정 스크립트: `pip install -r tools/requirements.txt` 후 `tools/` 안에서 실행
- Mac 캡처 도구: Mac에서 `sh mac-capture/build.sh` 후 `open mac-capture/FGACapture.app`, 이어서 `python3 mac-capture/frame_server.py`
- 호스트: JDK 21 필요. Android SDK는 필요 없습니다. Mac 주소와 시리얼 포트는 `configs/host.example.json`을 `data/host.json`으로 복사해 적습니다. 측정 스크립트는 `FGA_MAC` 환경 변수(`사용자@Mac주소`)를 씁니다.
  - `./gradlew :host:run --args="probe Kr 20"` 템플릿 전부를 현재 화면 전체에서 찾아 점수 순으로 출력
  - `./gradlew :host:run --args="screen"` 스크립트의 화면 판별 코드를 현재 화면에 실행 (터치 없음)

## 호스트에서 확인한 것 (2026-11-23)

- FGA-Modified의 `libautomata`·`scripts`가 수정 없이 빌드되고, 전투 스크립트까지 부품이 조립됩니다.
- iPhone 17 화면(2622×1206)을 높이 720으로 줄인 뒤 한국 서버 템플릿을 매칭하면 서포트 화면 요소가 0.92~0.97로 잡힙니다(기준 0.8). JPEG로 받은 화면 기준입니다.
- 스크립트의 서포트 화면 판별(정해진 영역, 기준 0.85)이 iPhone 화면에서 참으로 나옵니다.
- 실제 전투가 처음부터 끝까지 돕니다. 스킬 없이 카드만 고르는 설정으로 1회, Android 앱에서 내보낸 설정(지정 서포트, 스킬 커맨드, 매 턴 보구)으로 1회 돌렸고 둘 다 정상 종료했습니다.
  - `./gradlew :host:run --args="battle configs/first-test.json --fga=<내보낸 전투 설정>"`
  - `--dry-run`은 터치를 보내지 않고, 설정에 AP 회복이 켜져 있으면 `--allow-refill` 없이는 시작하지 않습니다.
- 연속 실행은 아직 하지 않았습니다. OCR, 재료 이미지, 두 손가락 동작은 없습니다.

## 라이선스

MIT. `host/`의 OpenCV 매칭 구현(`CvPattern.kt`)과 부품 연결은 [FGA-Modified](https://github.com/puruuun/FGA-Modified)(원본 [Fate-Grand-Automata/FGA](https://github.com/Fate-Grand-Automata/FGA), MIT)의 코드를 옮긴 것입니다. `fga/` 서브모듈은 그 저장소의 라이선스를 따릅니다.
