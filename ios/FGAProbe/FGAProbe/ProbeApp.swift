// 백그라운드 화면 캡처 검증 앱.
//
// 확인하려는 것: 이 앱이 뒤에 있고 FGO가 앞에 떠 있는 동안에도
//   1) 화면 프레임이 계속 들어오는가
//   2) iOS가 앱을 정지시키거나 메모리 문제로 종료시키지 않는가
//
// 1초마다 Documents/probe.log에 한 줄을 쓴다. 앱이 정지되면 기록이 그 시각에서 끊긴다.
// 10초마다 마지막 프레임을 Documents/last.jpg로 저장해, 받은 화면이 정말 다른 앱의 화면인지 볼 수 있게 한다.

import Accelerate
import CoreImage
import ScreenCaptureKit
import SwiftUI
import UIKit

@main
struct ProbeApp: App {
    var body: some Scene {
        WindowGroup { ProbeView() }
    }
}

struct ProbeView: View {
    @StateObject private var probe = Probe.shared

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("FGA 백그라운드 캡처 검증").font(.headline)
            HStack {
                Button(probe.running ? "중지" : "캡처 시작") {
                    probe.running ? probe.stop() : probe.start()
                }
                .buttonStyle(.borderedProminent)
                Button("기록 지우기") { probe.clearLog() }.buttonStyle(.bordered)
            }
            HStack {
                Button("보드 연결") { BoardLink.shared.connect() }.buttonStyle(.bordered)
                Button("STAT") { BoardLink.shared.send("STAT") }.buttonStyle(.bordered)
                // 포인터만 옮긴다. 탭은 보내지 않으므로 아무것도 눌리지 않는다.
                Button("포인터 이동") { BoardLink.shared.send("MOVE 8192 8192") }.buttonStyle(.bordered)
                Button(probe.pinging ? "주기 PING 끄기" : "주기 PING") { probe.pinging.toggle() }.buttonStyle(.bordered)
            }
            // 부하 시험: 실제 앱이 쓸 만큼의 메모리와 연산을 흉내 낸다.
            HStack {
                Stepper("메모리 채우기 \(probe.ballastMB)MB", value: $probe.ballastMB, in: 0...600, step: 50)
            }
            HStack {
                Toggle("연산 부하", isOn: $probe.cpuLoad)
                Toggle("5분마다 +50MB", isOn: $probe.autoRamp)
            }
            Text(probe.status).font(.subheadline)
            ScrollView {
                Text(probe.tail).font(.system(size: 11, design: .monospaced))
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .padding()
    }
}

final class Probe: NSObject, ObservableObject, SCContentSharingPickerObserver, SCStreamOutput, SCStreamDelegate {
    static let shared = Probe()

    @Published var running = false
    /// 켜 두면 10초마다 보드에 PING을 보낸다. 앱이 뒤에 있을 때도 블루투스 명령이 나가는지 보는 용도.
    @Published var pinging = false
    @Published var status = "대기 중"
    @Published var tail = ""

    /// 일부러 붙잡아 둘 메모리(MB). 실제 앱이 템플릿과 런타임으로 쓸 양을 흉내 낸다.
    @Published var ballastMB = 0 { didSet { frameQueue.async { self.resizeBallast(to: self.ballastMB) } } }
    /// 켜면 초당 3번, 프레임 하나를 흑백으로 바꾸고 매칭과 비슷한 연산을 약 80ms 돌린다.
    @Published var cpuLoad = false
    /// 켜면 5분마다 메모리 채우기를 50MB씩 올린다. 어느 선에서 iOS가 앱을 종료하는지 찾는 용도.
    @Published var autoRamp = false

    // 메모리 채우기. 10MB 덩어리들이고, 압축되지 않도록 무작위 값으로 채운다
    // (iOS는 뒤에 있는 앱의 메모리를 압축하므로 0으로 채우면 실제 부담이 되지 않는다).
    private var ballast: [UnsafeMutableRawPointer] = []
    private let chunkBytes = 10 * 1_048_576
    private var lastWork = Date.distantPast
    private var workMsThisSecond = 0.0
    private var workScratch = [Float](repeating: 0, count: 256 * 256)
    private var workTemplate = (0..<(256 * 256)).map { _ in Float.random(in: 0...1) }

    private func resizeBallast(to mb: Int) {
        let want = mb / 10
        while ballast.count < want {
            let p = UnsafeMutableRawPointer.allocate(byteCount: chunkBytes, alignment: 16384)
            arc4random_buf(p, chunkBytes)
            ballast.append(p)
        }
        while ballast.count > want { ballast.removeLast().deallocate() }
        log("ballast set to \(ballast.count * 10)MB")
    }

    private var stream: SCStream?
    private let frameQueue = DispatchQueue(label: "probe.frames")
    private let ciContext = CIContext()
    private var timer: DispatchSourceTimer?

    // frameQueue에서만 만진다.
    private var framesThisSecond = 0
    private var totalFrames = 0
    private var lastSize = CGSize.zero
    private var lastSave = Date.distantPast
    private var startedAt = Date()

    private var docs: URL { FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0] }
    private var logURL: URL { docs.appendingPathComponent("probe.log") }
    private var jpgURL: URL { docs.appendingPathComponent("last.jpg") }

    // MARK: 기록

    func log(_ text: String) {
        let stamp = ISO8601DateFormatter.string(from: Date(), timeZone: .current, formatOptions: [.withTime, .withColonSeparatorInTime])
        let line = "\(stamp) \(text)\n"
        if let handle = try? FileHandle(forWritingTo: logURL) {
            handle.seekToEndOfFile()
            handle.write(line.data(using: .utf8)!)
            try? handle.close()
        } else {
            try? line.data(using: .utf8)!.write(to: logURL)
        }
        DispatchQueue.main.async {
            let lines = (self.tail + line).split(separator: "\n", omittingEmptySubsequences: true).suffix(40)
            self.tail = lines.joined(separator: "\n") + "\n"
        }
    }

    func clearLog() {
        try? FileManager.default.removeItem(at: logURL)
        tail = ""
    }

    /// 앱이 실제로 쓰고 있는 메모리(MB). iOS가 종료 여부를 판단할 때 보는 값이다.
    private func footprintMB() -> Double {
        var info = task_vm_info_data_t()
        var count = mach_msg_type_number_t(MemoryLayout<task_vm_info_data_t>.size / MemoryLayout<integer_t>.size)
        let kr = withUnsafeMutablePointer(to: &info) {
            $0.withMemoryRebound(to: integer_t.self, capacity: Int(count)) {
                task_info(mach_task_self_, task_flavor_t(TASK_VM_INFO), $0, &count)
            }
        }
        return kr == KERN_SUCCESS ? Double(info.phys_footprint) / 1_048_576 : -1
    }

    // MARK: 시작과 중지

    func start() {
        let picker = SCContentSharingPicker.shared
        log("start: picker available=\(picker.isAvailable)")
        picker.add(self)
        picker.isActive = true
        picker.present(using: .display)
        status = "화면 공유 선택 창에서 전체 화면을 고르세요"
    }

    func stop() {
        timer?.cancel()
        timer = nil
        stream?.stopCapture { error in
            self.log("stopCapture error=\(String(describing: error))")
        }
        stream = nil
        running = false
        status = "중지됨"
    }

    private func begin(with filter: SCContentFilter) {
        let config = SCStreamConfiguration()
        // 짧은 변이 720이 되게 받는다. FGA의 비교 해상도(높이 720)와 같아 실제 앱이 받을 크기다.
        let shortSide = min(filter.contentRect.width, filter.contentRect.height)
        let scale = 720 / shortSide
        config.width = Int((filter.contentRect.width * scale).rounded())
        config.height = Int((filter.contentRect.height * scale).rounded())
        log("filter rect=\(filter.contentRect) scale=\(filter.pointPixelScale) -> request \(config.width)x\(config.height)")

        let stream = SCStream(filter: filter, configuration: config, delegate: self)
        do {
            try stream.addStreamOutput(self, type: .screen, sampleHandlerQueue: frameQueue)
        } catch {
            log("addStreamOutput failed: \(error)")
            return
        }
        self.stream = stream
        startedAt = Date()
        stream.startCapture { error in
            self.log("startCapture error=\(String(describing: error))")
            DispatchQueue.main.async {
                self.running = error == nil
                self.status = error == nil ? "캡처 중. 이제 FGO로 넘어가세요" : "시작 실패: \(error!.localizedDescription)"
                if error == nil { self.startTimer() }
            }
        }
    }

    private func startTimer() {
        let timer = DispatchSource.makeTimerSource(queue: frameQueue)
        timer.schedule(deadline: .now() + 1, repeating: 1)
        timer.setEventHandler { [weak self] in
            guard let self else { return }
            let fps = self.framesThisSecond
            self.framesThisSecond = 0
            let state: String = DispatchQueue.main.sync {
                switch UIApplication.shared.applicationState {
                case .active: return "active"
                case .inactive: return "inactive"
                case .background: return "background"
                @unknown default: return "unknown"
                }
            }
            let thermal = ProcessInfo.processInfo.thermalState.rawValue
            let elapsed = Int(Date().timeIntervalSince(self.startedAt))
            if elapsed % 10 == 0, DispatchQueue.main.sync(execute: { self.pinging }) {
                BoardLink.shared.send("PING")
            }
            if elapsed > 0, elapsed % 300 == 0, DispatchQueue.main.sync(execute: { self.autoRamp }) {
                DispatchQueue.main.async { self.ballastMB = min(600, self.ballastMB + 50) }
            }
            // avail: iOS가 이 앱을 종료시키기 전까지 더 쓸 수 있는 메모리. 0에 가까워지면 종료 직전이다.
            let avail = Double(os_proc_available_memory()) / 1_048_576
            let work = self.workMsThisSecond
            self.workMsThisSecond = 0
            self.log(String(format: "t=%ds state=%@ fps=%d total=%d size=%dx%d mem=%.1fMB avail=%.0fMB ballast=%dMB work=%.0fms thermal=%d",
                            elapsed, state, fps, self.totalFrames,
                            Int(self.lastSize.width), Int(self.lastSize.height), self.footprintMB(),
                            avail, self.ballast.count * 10, work, thermal))
        }
        timer.resume()
        self.timer = timer
    }

    // MARK: SCContentSharingPickerObserver

    func contentSharingPicker(_ picker: SCContentSharingPicker, didCancelFor stream: SCStream?) {
        log("picker cancelled")
        DispatchQueue.main.async { self.status = "선택이 취소됨" }
    }

    func contentSharingPicker(_ picker: SCContentSharingPicker, didUpdateWith filter: SCContentFilter, for stream: SCStream?) {
        log("picker selected style=\(filter.style.rawValue)")
        begin(with: filter)
    }

    func contentSharingPickerStartDidFailWithError(_ error: Error) {
        log("picker failed: \(error)")
        DispatchQueue.main.async { self.status = "선택 창 실패: \(error.localizedDescription)" }
    }

    // MARK: SCStreamOutput / SCStreamDelegate

    func stream(_ stream: SCStream, didOutputSampleBuffer sampleBuffer: CMSampleBuffer, of type: SCStreamOutputType) {
        guard type == .screen, let pixelBuffer = CMSampleBufferGetImageBuffer(sampleBuffer) else { return }
        framesThisSecond += 1
        totalFrames += 1
        lastSize = CGSize(width: CVPixelBufferGetWidth(pixelBuffer), height: CVPixelBufferGetHeight(pixelBuffer))

        let now = Date()
        if now.timeIntervalSince(lastWork) >= 0.33, DispatchQueue.main.sync(execute: { self.cpuLoad }) {
            lastWork = now
            simulateMatching(on: pixelBuffer)
        }
        if now.timeIntervalSince(lastSave) >= 10 {
            lastSave = now
            autoreleasepool {
                let image = CIImage(cvPixelBuffer: pixelBuffer)
                try? ciContext.writeJPEGRepresentation(of: image, to: jpgURL,
                                                       colorSpace: CGColorSpace(name: CGColorSpace.sRGB)!)
            }
        }
    }

    /// 템플릿 매칭과 비슷한 부담을 준다: 프레임의 256×256 영역을 실수로 바꾸고, 약 80ms 동안
    /// 같은 크기의 템플릿과 내적을 반복한다. 실제 매칭이 아니라 연산량과 메모리 접근만 흉내 낸 것이다.
    private func simulateMatching(on pixelBuffer: CVPixelBuffer) {
        let began = CFAbsoluteTimeGetCurrent()
        CVPixelBufferLockBaseAddress(pixelBuffer, .readOnly)
        defer { CVPixelBufferUnlockBaseAddress(pixelBuffer, .readOnly) }
        guard let base = CVPixelBufferGetBaseAddressOfPlane(pixelBuffer, 0) ?? CVPixelBufferGetBaseAddress(pixelBuffer) else { return }
        let rowBytes = CVPixelBufferGetBytesPerRowOfPlane(pixelBuffer, 0) > 0
            ? CVPixelBufferGetBytesPerRowOfPlane(pixelBuffer, 0) : CVPixelBufferGetBytesPerRow(pixelBuffer)
        let height = CVPixelBufferGetHeight(pixelBuffer)
        let rows = min(256, height)
        let cols = min(256, rowBytes)
        for y in 0..<rows {
            let src = base.advanced(by: y * rowBytes).assumingMemoryBound(to: UInt8.self)
            workScratch.withUnsafeMutableBufferPointer { dst in
                vDSP_vfltu8(src, 1, dst.baseAddress! + y * 256, 1, vDSP_Length(cols))
            }
        }
        var sink: Float = 0
        while CFAbsoluteTimeGetCurrent() - began < 0.08 {
            var dot: Float = 0
            vDSP_dotpr(workScratch, 1, workTemplate, 1, &dot, vDSP_Length(256 * 256))
            sink += dot
        }
        if sink.isNaN { log("work produced NaN") }
        workMsThisSecond += (CFAbsoluteTimeGetCurrent() - began) * 1000
    }

    func stream(_ stream: SCStream, didStopWithError error: Error) {
        log("STREAM STOPPED: \(error)")
        DispatchQueue.main.async {
            self.running = false
            self.status = "스트림이 멈춤: \(error.localizedDescription)"
        }
    }
}

// MARK: - ESP32 보드와의 블루투스 연결

import CoreBluetooth

/// 펌웨어(`firmware/`)의 명령 서비스에 연결해 텍스트 명령을 보낸다.
///
/// 보드는 이미 iPhone에 블루투스 마우스로 붙어 있다. iOS는 그 연결을 앱과 함께 쓰게 해 주므로,
/// 새로 검색하지 않고 "시스템에 이미 연결된 기기" 가운데 명령 서비스를 가진 것을 찾아 쓴다.
final class BoardLink: NSObject, CBCentralManagerDelegate, CBPeripheralDelegate {
    static let shared = BoardLink()

    private let service = CBUUID(string: "6E400001-B5A3-F393-E0A9-E50E24DCCA9E")
    private let rxUUID = CBUUID(string: "6E400002-B5A3-F393-E0A9-E50E24DCCA9E")  // 앱 -> 보드
    private let txUUID = CBUUID(string: "6E400003-B5A3-F393-E0A9-E50E24DCCA9E")  // 보드 -> 앱
    // HID 서비스. 명령 서비스로 못 찾을 때 연결된 마우스를 찾는 데 쓴다.
    private let hidService = CBUUID(string: "1812")

    private var central: CBCentralManager?
    private var peripheral: CBPeripheral?
    private var rx: CBCharacteristic?
    private var wantConnect = false

    private func log(_ text: String) { Probe.shared.log("[board] " + text) }

    func connect() {
        wantConnect = true
        if central == nil {
            central = CBCentralManager(delegate: self, queue: nil)
        } else {
            attempt()
        }
    }

    func send(_ line: String) {
        guard let peripheral, let rx else {
            log("send \(line): not connected")
            return
        }
        peripheral.writeValue((line + "\n").data(using: .ascii)!, for: rx, type: .withResponse)
    }

    private func attempt() {
        guard let central, central.state == .poweredOn, wantConnect else { return }
        var found = central.retrieveConnectedPeripherals(withServices: [service])
        log("connected peripherals with command service: \(found.map { $0.name ?? "?" })")
        if found.isEmpty {
            found = central.retrieveConnectedPeripherals(withServices: [hidService])
            log("connected HID peripherals: \(found.map { $0.name ?? "?" })")
        }
        guard let target = found.first(where: { $0.name == "FGA Bridge" }) ?? found.first else {
            // 연결된 기기에서 못 찾으면 광고를 검색한다 (보드가 아직 마우스로 붙지 않은 경우).
            log("scanning")
            central.scanForPeripherals(withServices: [service, hidService])
            return
        }
        peripheral = target
        target.delegate = self
        central.connect(target)
    }

    func centralManagerDidUpdateState(_ central: CBCentralManager) {
        log("bluetooth state=\(central.state.rawValue)")
        attempt()
    }

    func centralManager(_ central: CBCentralManager, didDiscover peripheral: CBPeripheral,
                        advertisementData: [String: Any], rssi RSSI: NSNumber) {
        log("discovered \(peripheral.name ?? "?")")
        guard peripheral.name == "FGA Bridge" else { return }
        central.stopScan()
        self.peripheral = peripheral
        peripheral.delegate = self
        central.connect(peripheral)
    }

    func centralManager(_ central: CBCentralManager, didConnect peripheral: CBPeripheral) {
        log("connected to \(peripheral.name ?? "?")")
        peripheral.discoverServices([service])
    }

    func centralManager(_ central: CBCentralManager, didFailToConnect peripheral: CBPeripheral, error: Error?) {
        log("connect failed: \(String(describing: error))")
    }

    func centralManager(_ central: CBCentralManager, didDisconnectPeripheral peripheral: CBPeripheral, error: Error?) {
        log("disconnected: \(String(describing: error))")
        rx = nil
        // 보드가 재시작되면 다시 붙을 때까지 기다렸다가 잡는다.
        if wantConnect { central.connect(peripheral) }
    }

    func peripheral(_ peripheral: CBPeripheral, didDiscoverServices error: Error?) {
        guard let found = peripheral.services?.first(where: { $0.uuid == service }) else {
            log("command service not found: \(String(describing: error))")
            return
        }
        peripheral.discoverCharacteristics([rxUUID, txUUID], for: found)
    }

    func peripheral(_ peripheral: CBPeripheral, didDiscoverCharacteristicsFor service: CBService, error: Error?) {
        for c in service.characteristics ?? [] {
            if c.uuid == rxUUID { rx = c }
            if c.uuid == txUUID { peripheral.setNotifyValue(true, for: c) }
        }
        log(rx == nil ? "command characteristic not found" : "ready")
    }

    func peripheral(_ peripheral: CBPeripheral, didUpdateValueFor characteristic: CBCharacteristic, error: Error?) {
        guard let data = characteristic.value, let text = String(data: data, encoding: .utf8) else { return }
        log("reply: \(text)")
    }

    func peripheral(_ peripheral: CBPeripheral, didWriteValueFor characteristic: CBCharacteristic, error: Error?) {
        if let error { log("write failed: \(error)") }
    }
}
