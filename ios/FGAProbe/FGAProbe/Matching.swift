// 전체 경로 시험용 템플릿 매칭.
//
// FGA가 쓰는 OpenCV의 TM_CCOEFF_NORMED와 같은 식을 Accelerate로 직접 계산한다.
// 시험에서 찾는 템플릿은 두 개뿐이고 찾는 영역도 작아서, OpenCV를 앱에 넣지 않고도 충분하다.
// 실제 앱에서는 FGA와 점수를 정확히 맞추기 위해 OpenCV를 쓸 예정이다.

import Accelerate
import CoreVideo
import Foundation

struct Template {
    let name: String
    let width: Int
    let height: Int
    /// 평균을 뺀 픽셀값.
    let zeroMean: [Float]
    /// zeroMean의 크기(제곱합의 제곱근).
    let norm: Float

    init(name: String, width: Int, height: Int, base64: String) {
        self.name = name
        self.width = width
        self.height = height
        let bytes = [UInt8](Data(base64Encoded: base64) ?? Data())
        precondition(bytes.count == width * height, "template \(name) has the wrong size")
        var pixels = [Float](repeating: 0, count: bytes.count)
        vDSP_vfltu8(bytes, 1, &pixels, 1, vDSP_Length(bytes.count))
        var mean: Float = 0
        vDSP_meanv(pixels, 1, &mean, vDSP_Length(pixels.count))
        var negMean = -mean
        vDSP_vsadd(pixels, 1, &negMean, &pixels, 1, vDSP_Length(pixels.count))
        var sumSquares: Float = 0
        vDSP_svesq(pixels, 1, &sumSquares, vDSP_Length(pixels.count))
        zeroMean = pixels
        norm = sumSquares.squareRoot()
    }
}

struct MatchResult {
    let score: Float
    let x: Int
    let y: Int
}

/// 가로 화면이 세로 버퍼 안에 어느 쪽으로 누워 있는지. 기기를 든 방향에 따라 달라진다.
enum BufferRotation: String {
    /// 바로 선 화면을 반시계 방향으로 90도 돌린 모습으로 들어 있다.
    case counterClockwise
    /// 바로 선 화면을 시계 방향으로 90도 돌린 모습으로 들어 있다.
    case clockwise
}

/// 잠근 픽셀 버퍼에서 흑백 값을 읽는다. 호출하는 쪽이 버퍼를 잠가 둔 동안만 유효하다.
struct GrayFrame {
    let base: UnsafePointer<UInt8>
    let rowBytes: Int
    let bufferWidth: Int
    let bufferHeight: Int
    /// 1이면 밝기 평면(YCbCr의 Y), 4면 BGRA.
    let bytesPerPixel: Int

    /// 바로 선 가로 화면의 크기.
    var uprightWidth: Int { bufferHeight }
    var uprightHeight: Int { bufferWidth }

    init?(_ pixelBuffer: CVPixelBuffer) {
        let planar = CVPixelBufferGetPlaneCount(pixelBuffer) > 0
        guard let address = planar ? CVPixelBufferGetBaseAddressOfPlane(pixelBuffer, 0)
                                   : CVPixelBufferGetBaseAddress(pixelBuffer) else { return nil }
        base = UnsafePointer(address.assumingMemoryBound(to: UInt8.self))
        rowBytes = planar ? CVPixelBufferGetBytesPerRowOfPlane(pixelBuffer, 0) : CVPixelBufferGetBytesPerRow(pixelBuffer)
        bufferWidth = CVPixelBufferGetWidth(pixelBuffer)
        bufferHeight = CVPixelBufferGetHeight(pixelBuffer)
        bytesPerPixel = planar ? 1 : 4
    }

    /// 바로 선 화면 좌표의 사각형을 흑백 실수 배열(행 우선)로 꺼낸다. 화면 밖으로 나가는 부분은 잘라 낸다.
    func region(x: Int, y: Int, width: Int, height: Int, rotation: BufferRotation) -> (pixels: [Float], width: Int, height: Int) {
        let x0 = max(0, x), y0 = max(0, y)
        let w = min(uprightWidth, x + width) - x0
        let h = min(uprightHeight, y + height) - y0
        guard w > 0, h > 0 else { return ([], 0, 0) }
        var out = [Float](repeating: 0, count: w * h)
        for row in 0..<h {
            let uy = y0 + row
            for col in 0..<w {
                let ux = x0 + col
                let bx: Int
                let by: Int
                switch rotation {
                case .counterClockwise:
                    bx = uy
                    by = uprightWidth - 1 - ux
                case .clockwise:
                    bx = uprightHeight - 1 - uy
                    by = ux
                }
                let p = base + by * rowBytes + bx * bytesPerPixel
                let value: Float
                if bytesPerPixel == 1 {
                    value = Float(p[0])
                } else {
                    // BGRA. OpenCV의 흑백 변환과 같은 가중치.
                    value = 0.114 * Float(p[0]) + 0.587 * Float(p[1]) + 0.299 * Float(p[2])
                }
                out[row * w + col] = value
            }
        }
        return (out, w, h)
    }
}

/// 영역 안에서 템플릿과 가장 비슷한 위치와 그 점수(-1...1)를 찾는다. TM_CCOEFF_NORMED와 같은 식이다.
func bestMatch(region: [Float], regionWidth: Int, regionHeight: Int, template: Template) -> MatchResult? {
    let tw = template.width, th = template.height
    guard regionWidth >= tw, regionHeight >= th, template.norm > 0 else { return nil }

    // 창 안의 합과 제곱합을 빨리 구하기 위한 누적 합 표.
    let iw = regionWidth + 1
    var sum = [Double](repeating: 0, count: iw * (regionHeight + 1))
    var sumSq = [Double](repeating: 0, count: iw * (regionHeight + 1))
    for y in 0..<regionHeight {
        var rowSum = 0.0, rowSq = 0.0
        for x in 0..<regionWidth {
            let v = Double(region[y * regionWidth + x])
            rowSum += v
            rowSq += v * v
            sum[(y + 1) * iw + x + 1] = sum[y * iw + x + 1] + rowSum
            sumSq[(y + 1) * iw + x + 1] = sumSq[y * iw + x + 1] + rowSq
        }
    }
    func window(_ table: [Double], _ x: Int, _ y: Int) -> Double {
        table[(y + th) * iw + x + tw] - table[y * iw + x + tw] - table[(y + th) * iw + x] + table[y * iw + x]
    }

    let count = Double(tw * th)
    var best = MatchResult(score: -2, x: 0, y: 0)
    region.withUnsafeBufferPointer { r in
        template.zeroMean.withUnsafeBufferPointer { t in
            for oy in 0...(regionHeight - th) {
                for ox in 0...(regionWidth - tw) {
                    // 템플릿의 평균을 뺐으므로, 분자는 템플릿과 창의 내적만으로 구해진다.
                    var numerator: Float = 0
                    for row in 0..<th {
                        var dot: Float = 0
                        vDSP_dotpr(r.baseAddress! + (oy + row) * regionWidth + ox, 1,
                                   t.baseAddress! + row * tw, 1, &dot, vDSP_Length(tw))
                        numerator += dot
                    }
                    let s = window(sum, ox, oy)
                    let variance = window(sumSq, ox, oy) - s * s / count
                    guard variance > 1e-3 else { continue }
                    let score = numerator / (template.norm * Float(variance.squareRoot()))
                    if score > best.score { best = MatchResult(score: score, x: ox, y: oy) }
                }
            }
        }
    }
    return best.score > -2 ? best : nil
}
