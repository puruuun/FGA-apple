// USB로 연결된 iPhone 화면을 받아 최신 프레임을 JPEG 파일로 계속 덮어쓴다.
// 출력: /tmp/fga-frame.jpg   로그: /tmp/fga-capture.log
// 카메라 권한이 필요하므로 .app 번들로 묶어 `open`으로 실행한다.

import AVFoundation
import CoreImage
import CoreMediaIO
import Foundation

let outPath = "/tmp/fga-frame.jpg"
let logPath = "/tmp/fga-capture.log"
let minInterval: TimeInterval = 0.25

func log(_ message: String) {
    let line = "\(Date()) \(message)\n"
    guard let data = line.data(using: .utf8) else { return }
    if let handle = FileHandle(forWritingAtPath: logPath) {
        handle.seekToEndOfFile()
        handle.write(data)
        handle.closeFile()
    } else {
        FileManager.default.createFile(atPath: logPath, contents: data)
    }
}

// iOS 기기는 이 속성을 켜야 캡처 장치 목록에 나타난다.
func enableScreenCaptureDevices() {
    var address = CMIOObjectPropertyAddress(
        mSelector: CMIOObjectPropertySelector(kCMIOHardwarePropertyAllowScreenCaptureDevices),
        mScope: CMIOObjectPropertyScope(kCMIOObjectPropertyScopeGlobal),
        mElement: CMIOObjectPropertyElement(kCMIOObjectPropertyElementMain))
    var allow: UInt32 = 1
    let status = CMIOObjectSetPropertyData(
        CMIOObjectID(kCMIOObjectSystemObject), &address, 0, nil,
        UInt32(MemoryLayout<UInt32>.size), &allow)
    log("allow screen capture devices status=\(status)")
}

final class Grabber: NSObject, AVCaptureVideoDataOutputSampleBufferDelegate {
    let session = AVCaptureSession()
    let context = CIContext()
    let colorSpace = CGColorSpace(name: CGColorSpace.sRGB)!
    var lastWrite = Date.distantPast
    var frames = 0

    func start(device: AVCaptureDevice) throws {
        let input = try AVCaptureDeviceInput(device: device)
        session.beginConfiguration()
        guard session.canAddInput(input) else {
            throw NSError(domain: "fga", code: 1,
                          userInfo: [NSLocalizedDescriptionKey: "cannot add input"])
        }
        session.addInput(input)
        let output = AVCaptureVideoDataOutput()
        output.videoSettings = [
            kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_32BGRA
        ]
        output.alwaysDiscardsLateVideoFrames = true
        output.setSampleBufferDelegate(self, queue: DispatchQueue(label: "fga.frames"))
        guard session.canAddOutput(output) else {
            throw NSError(domain: "fga", code: 2,
                          userInfo: [NSLocalizedDescriptionKey: "cannot add output"])
        }
        session.addOutput(output)
        session.commitConfiguration()

        let center = NotificationCenter.default
        center.addObserver(forName: AVCaptureSession.runtimeErrorNotification, object: session,
                           queue: nil) { note in
            log("runtime error: \(String(describing: note.userInfo))")
        }
        center.addObserver(forName: AVCaptureSession.didStartRunningNotification, object: session,
                           queue: nil) { _ in log("session did start") }
        center.addObserver(forName: AVCaptureSession.didStopRunningNotification, object: session,
                           queue: nil) { _ in log("session did stop") }

        log("format: \(device.activeFormat)")
        log("video connection: \(String(describing: output.connection(with: .video)))")
        session.startRunning()
        log("isRunning=\(session.isRunning)")
        DispatchQueue.main.asyncAfter(deadline: .now() + 5) {
            log("after 5s isRunning=\(self.session.isRunning) frames=\(self.frames) dropped=\(self.dropped)")
        }
    }

    var dropped = 0
    func captureOutput(_ output: AVCaptureOutput, didDrop sampleBuffer: CMSampleBuffer,
                       from connection: AVCaptureConnection) {
        dropped += 1
    }

    func captureOutput(_ output: AVCaptureOutput, didOutput sampleBuffer: CMSampleBuffer,
                       from connection: AVCaptureConnection) {
        let now = Date()
        guard now.timeIntervalSince(lastWrite) >= minInterval,
              let pixelBuffer = CMSampleBufferGetImageBuffer(sampleBuffer) else { return }
        lastWrite = now
        let image = CIImage(cvPixelBuffer: pixelBuffer)
        let tmpPath = outPath + ".tmp"
        do {
            try context.writeJPEGRepresentation(
                of: image, to: URL(fileURLWithPath: tmpPath), colorSpace: colorSpace, options: [:])
            if rename(tmpPath, outPath) != 0 { log("rename failed errno=\(errno)") }
        } catch {
            log("write error: \(error)")
        }
        frames += 1
        if frames == 1 {
            log("first frame \(CVPixelBufferGetWidth(pixelBuffer))x\(CVPixelBufferGetHeight(pixelBuffer))")
        }
    }
}

let grabber = Grabber()
let wantedName = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : nil

func findAndStart(attempt: Int) {
    let discovery = AVCaptureDevice.DiscoverySession(
        deviceTypes: [.external], mediaType: .muxed, position: .unspecified)
    let devices = discovery.devices
    if attempt % 4 == 0 {
        log("devices: \(devices.map { "\($0.localizedName) [\($0.modelID)]" })")
    }
    let chosen = devices.first { wantedName == nil || $0.localizedName == wantedName } ?? nil
    if let device = chosen {
        do {
            try grabber.start(device: device)
            log("capturing from \(device.localizedName)")
        } catch {
            log("start error: \(error)")
            exit(3)
        }
        return
    }
    if attempt >= 60 {
        log("no iOS device found")
        exit(4)
    }
    DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) { findAndStart(attempt: attempt + 1) }
}

log("start pid=\(ProcessInfo.processInfo.processIdentifier)")
enableScreenCaptureDevices()
AVCaptureDevice.requestAccess(for: .video) { granted in
    log("camera access granted=\(granted)")
    guard granted else { exit(2) }
    DispatchQueue.main.async { findAndStart(attempt: 0) }
}
RunLoop.main.run()
