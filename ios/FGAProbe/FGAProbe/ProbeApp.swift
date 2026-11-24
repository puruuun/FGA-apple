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
                Button("포인터 이동") { BoardLink.shared.send("MOVE 8192 8192") }.buttonStyle(.bordered)
                Button(probe.pinging ? "주기 PING 끄기" : "주기 PING") { probe.pinging.toggle() }.buttonStyle(.bordered)
            }
            HStack {
                Stepper("메모리 채우기 \(probe.ballastMB)MB", value: $probe.ballastMB, in: 0...600, step: 50)
            }
            HStack {
                Toggle("연산 부하", isOn: $probe.cpuLoad)
                Toggle("5분마다 +50MB", isOn: $probe.autoRamp)
            }
            Toggle("연결 시험 (퀘스트 목록 ↔ 서포트 화면 \(probe.chainCycles)/\(Probe.chainMaxCycles)회)", isOn: $probe.chainTest)
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
    @Published var pinging = false
    @Published var status = "대기 중"
    @Published var tail = ""

    @Published var ballastMB = 0 { didSet { frameQueue.async { self.resizeBallast(to: self.ballastMB) } } }
    @Published var cpuLoad = false
    @Published var autoRamp = false

    static let chainMaxCycles = 5
    @Published var chainTest = false {
        didSet {
            guard chainTest != oldValue else { return }
            if chainTest {
                chainCycles = 0
                BoardLink.shared.send("AVOID 15040 16480")
            }
            frameQueue.async {
                self.chainRotation = nil
                self.chainNextAction = .distantPast
                self.log("chain test \(self.chainTest ? "on" : "off")")
            }
        }
    }
    @Published var chainCycles = 0

    // Values for iPhone 17 (2622x1206) taken with `host regions`.
    private let menuRegion = (x: 1265, y: 600, w: 300, h: 120)
    private let supportRegion = (x: 85, y: 0, w: 100, h: 200)
    private let questTap = "TAP 27955 10025"
    private let backTap = "TAP 3700 2030"

    private var chainRotation: BufferRotation?
    private var chainNextAction = Date.distantPast
    private var chainLastCheck = Date.distantPast
    private var chainLoggedFormat = false

    private var chainAwaitingReply = false
    private var chainSentAt = Date.distantPast

    private func sendChainTap(_ command: String) {
        chainAwaitingReply = true
        chainSentAt = Date()
        BoardLink.shared.send(command) { [weak self] reply, ms in
            guard let self else { return }
            self.frameQueue.async {
                self.chainAwaitingReply = false
                self.chainNextAction = Date().addingTimeInterval(2)
                self.log("chain: \(command) -> \(reply) after \(ms)ms")
            }
        }
    }

    private func runChain(on pixelBuffer: CVPixelBuffer) {
        let (enabled, state, cycles): (Bool, UIApplication.State, Int) = DispatchQueue.main.sync {
            (self.chainTest, UIApplication.shared.applicationState, self.chainCycles)
        }
        guard enabled, state == .background else { return }

        let began = CFAbsoluteTimeGetCurrent()
        CVPixelBufferLockBaseAddress(pixelBuffer, .readOnly)
        defer { CVPixelBufferUnlockBaseAddress(pixelBuffer, .readOnly) }
        guard let frame = GrayFrame(pixelBuffer) else { return }
        if !chainLoggedFormat {
            chainLoggedFormat = true
            let f = CVPixelBufferGetPixelFormatType(pixelBuffer)
            let fourCC = String(bytes: [UInt8(f >> 24 & 255), UInt8(f >> 16 & 255), UInt8(f >> 8 & 255), UInt8(f & 255)], encoding: .ascii) ?? "?"
            log("chain: pixel format \(fourCC), buffer \(frame.bufferWidth)x\(frame.bufferHeight), bytesPerPixel \(frame.bytesPerPixel)")
        }

        func score(_ r: (x: Int, y: Int, w: Int, h: Int), _ template: Template, _ rotation: BufferRotation) -> Float {
            let region = frame.region(x: r.x, y: r.y, width: r.w, height: r.h, rotation: rotation)
            return bestMatch(region: region.pixels, regionWidth: region.width, regionHeight: region.height,
                             template: template)?.score ?? -1
        }

        var best: (rotation: BufferRotation, menu: Float, support: Float)?
        for rotation in chainRotation.map({ [$0] }) ?? [.counterClockwise, .clockwise] {
            let menu = score(menuRegion, Templates.menu, rotation)
            let support = score(supportRegion, Templates.supportScreen, rotation)
            if best == nil || max(menu, support) > max(best!.menu, best!.support) {
                best = (rotation, menu, support)
            }
        }
        guard let found = best else { return }
        let elapsedMs = (CFAbsoluteTimeGetCurrent() - began) * 1000
        if chainRotation == nil, max(found.menu, found.support) >= 0.8 {
            chainRotation = found.rotation
            log("chain: rotation is \(found.rotation.rawValue)")
        }

        let now = Date()
        var action = "-"
        if chainAwaitingReply {
            action = "await reply"
            if now.timeIntervalSince(chainSentAt) > 6 {
                chainAwaitingReply = false
                log("chain: no reply from the board in 6s, stopping")
                DispatchQueue.main.async { self.chainTest = false }
            }
        } else if now >= chainNextAction {
            if found.menu >= 0.8 {
                action = "tap quest"
                sendChainTap(questTap)
            } else if found.support >= 0.85 {
                action = "tap back"
                sendChainTap(backTap)
                let done = cycles + 1
                DispatchQueue.main.async {
                    self.chainCycles = done
                    if done >= Probe.chainMaxCycles { self.chainTest = false }
                }
            }
        } else {
            action = "wait"
        }
        log(String(format: "chain: menu=%.3f support=%.3f match=%.0fms action=%@", found.menu, found.support, elapsedMs, action))
    }

    private var ballast: [UnsafeMutableRawPointer] = []
    // iOS compresses the memory of a backgrounded app, so zero-filled ballast adds no real load; fill with random bytes.
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

    private var framesThisSecond = 0
    private var totalFrames = 0
    private var lastSize = CGSize.zero
    private var lastSave = Date.distantPast
    private var startedAt = Date()

    private var docs: URL { FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0] }
    private var logURL: URL { docs.appendingPathComponent("probe.log") }
    private var jpgURL: URL { docs.appendingPathComponent("last.jpg") }

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

    func start() {
        let picker = SCContentSharingPicker.shared
        log("start: picker available=\(picker.isAvailable)")
        if !observingPicker {
            observingPicker = true
            picker.add(self)
        }
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

    private var observingPicker = false

    private func begin(with filter: SCContentFilter) {
        guard stream == nil else {
            log("begin ignored: a stream already exists")
            return
        }
        let config = SCStreamConfiguration()
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
                if error != nil { self.stream = nil }
                self.status = error == nil ? "캡처 중. 이제 FGO로 넘어가세요" : "시작 실패: \(error!.localizedDescription)"
                if error == nil { self.startTimer() }
            }
        }
    }

    private func startTimer() {
        self.timer?.cancel()
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

    func contentSharingPicker(_ picker: SCContentSharingPicker, didCancelFor stream: SCStream?) {
        log("picker cancelled")
        DispatchQueue.main.async { if !self.running { self.status = "선택이 취소됨" } }
    }

    func contentSharingPicker(_ picker: SCContentSharingPicker, didUpdateWith filter: SCContentFilter, for stream: SCStream?) {
        log("picker selected style=\(filter.style.rawValue)")
        begin(with: filter)
    }

    func contentSharingPickerStartDidFailWithError(_ error: Error) {
        log("picker failed: \(error)")
        DispatchQueue.main.async { self.status = "선택 창 실패: \(error.localizedDescription)" }
    }

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
        if now.timeIntervalSince(chainLastCheck) >= 0.5 {
            chainLastCheck = now
            runChain(on: pixelBuffer)
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
            self.timer?.cancel()
            self.timer = nil
            self.stream = nil
            self.running = false
            self.status = "스트림이 멈춤: \(error.localizedDescription)"
        }
    }
}

import CoreBluetooth

final class BoardLink: NSObject, CBCentralManagerDelegate, CBPeripheralDelegate {
    static let shared = BoardLink()

    private let service = CBUUID(string: "6E400001-B5A3-F393-E0A9-E50E24DCCA9E")
    private let rxUUID = CBUUID(string: "6E400002-B5A3-F393-E0A9-E50E24DCCA9E")
    private let txUUID = CBUUID(string: "6E400003-B5A3-F393-E0A9-E50E24DCCA9E")
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

    private var pending: [(line: String, sentAt: CFAbsoluteTime, completion: ((String, Int) -> Void)?)] = []

    func send(_ line: String, completion: ((String, Int) -> Void)? = nil) {
        DispatchQueue.main.async {
            guard let peripheral = self.peripheral, let rx = self.rx else {
                self.log("send \(line): not connected")
                completion?("ERR not connected", 0)
                return
            }
            self.pending.append((line, CFAbsoluteTimeGetCurrent(), completion))
            peripheral.writeValue((line + "\n").data(using: .ascii)!, for: rx, type: .withoutResponse)
        }
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
        guard !pending.isEmpty else {
            log("reply: \(text)")
            return
        }
        let sent = pending.removeFirst()
        let ms = Int((CFAbsoluteTimeGetCurrent() - sent.sentAt) * 1000)
        if sent.completion == nil { log("\(sent.line) -> \(text) after \(ms)ms") }
        sent.completion?(text, ms)
    }

    func peripheral(_ peripheral: CBPeripheral, didWriteValueFor characteristic: CBCharacteristic, error: Error?) {
        if let error { log("write failed: \(error)") }
    }
}
