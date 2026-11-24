import Accelerate
import CoreVideo
import Foundation

struct Template {
    let name: String
    let width: Int
    let height: Int
    let zeroMean: [Float]
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

enum BufferRotation: String {
    case counterClockwise
    case clockwise
}

struct GrayFrame {
    let base: UnsafePointer<UInt8>
    let rowBytes: Int
    let bufferWidth: Int
    let bufferHeight: Int
    let bytesPerPixel: Int

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
                    value = 0.114 * Float(p[0]) + 0.587 * Float(p[1]) + 0.299 * Float(p[2])
                }
                out[row * w + col] = value
            }
        }
        return (out, w, h)
    }
}

func bestMatch(region: [Float], regionWidth: Int, regionHeight: Int, template: Template) -> MatchResult? {
    let tw = template.width, th = template.height
    guard regionWidth >= tw, regionHeight >= th, template.norm > 0 else { return nil }

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
