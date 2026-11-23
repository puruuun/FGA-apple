// ios/FGAProbe의 매칭 코드를 Mac에서 돌려 보는 점검 도구.
// 검증 앱이 저장한 프레임(세로 버퍼 그대로의 JPEG)을 읽어, 두 회전 방향으로 템플릿 점수를 출력한다.
//
//   swiftc -O tools/matching_check.swift ios/FGAProbe/FGAProbe/Matching.swift \
//          ios/FGAProbe/FGAProbe/Templates.swift -o /tmp/matching_check
//   /tmp/matching_check last.jpg

import CoreGraphics
import CoreVideo
import Foundation
import ImageIO

@main
struct Check {
    static func main() {
        let path = CommandLine.arguments.dropFirst().first ?? "last.jpg"
        guard let source = CGImageSourceCreateWithURL(URL(fileURLWithPath: path) as CFURL, nil),
              let image = CGImageSourceCreateImageAtIndex(source, 0, nil) else {
            print("cannot read \(path)")
            exit(1)
        }
        var buffer: CVPixelBuffer?
        CVPixelBufferCreate(nil, image.width, image.height, kCVPixelFormatType_32BGRA, nil, &buffer)
        guard let pixelBuffer = buffer else { exit(1) }
        CVPixelBufferLockBaseAddress(pixelBuffer, [])
        defer { CVPixelBufferUnlockBaseAddress(pixelBuffer, []) }
        let context = CGContext(
            data: CVPixelBufferGetBaseAddress(pixelBuffer), width: image.width, height: image.height,
            bitsPerComponent: 8, bytesPerRow: CVPixelBufferGetBytesPerRow(pixelBuffer),
            space: CGColorSpace(name: CGColorSpace.sRGB)!,
            bitmapInfo: CGImageAlphaInfo.premultipliedFirst.rawValue | CGBitmapInfo.byteOrder32Little.rawValue)!
        context.draw(image, in: CGRect(x: 0, y: 0, width: image.width, height: image.height))

        guard let frame = GrayFrame(pixelBuffer) else { exit(1) }
        print("buffer \(frame.bufferWidth)x\(frame.bufferHeight), upright \(frame.uprightWidth)x\(frame.uprightHeight)")
        let regions: [(String, (x: Int, y: Int, w: Int, h: Int), Template)] = [
            ("menu", (1265, 600, 300, 120), Templates.menu),
            ("support", (85, 0, 100, 200), Templates.supportScreen),
        ]
        for rotation in [BufferRotation.counterClockwise, .clockwise] {
            for (name, r, template) in regions {
                let began = CFAbsoluteTimeGetCurrent()
                let region = frame.region(x: r.x, y: r.y, width: r.w, height: r.h, rotation: rotation)
                let match = bestMatch(region: region.pixels, regionWidth: region.width,
                                      regionHeight: region.height, template: template)
                let ms = (CFAbsoluteTimeGetCurrent() - began) * 1000
                print(String(format: "%@ %@ score=%.3f at %d,%d (%.1f ms)", rotation.rawValue, name,
                             match?.score ?? -1, (match?.x ?? 0) + r.x, (match?.y ?? 0) + r.y, ms))
            }
        }
    }
}
