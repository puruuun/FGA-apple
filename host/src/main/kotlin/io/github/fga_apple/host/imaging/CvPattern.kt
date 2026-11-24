package io.github.fga_apple.host.imaging

import io.github.lib_automata.ImageFeatureMatch
import io.github.lib_automata.Match
import io.github.lib_automata.MeanColor
import io.github.lib_automata.Pattern
import io.github.lib_automata.Region
import io.github.lib_automata.Size
import org.opencv.core.Core
import org.opencv.core.CvType
import org.opencv.core.Mat
import org.opencv.core.MatOfByte
import org.opencv.core.MatOfDMatch
import org.opencv.core.MatOfKeyPoint
import org.opencv.core.MatOfPoint
import org.opencv.core.Point
import org.opencv.core.Rect
import org.opencv.core.Scalar
import org.opencv.features2d.BFMatcher
import org.opencv.features2d.SIFT
import org.opencv.imgcodecs.Imgcodecs
import org.opencv.imgproc.Imgproc
import java.io.InputStream
import java.io.OutputStream
import kotlin.math.roundToInt
import org.opencv.core.Size as CvSize

internal inline fun <T> Mat.use(block: (Mat) -> T): T =
    try {
        block(this)
    } finally {
        release()
    }

class CvPattern(
    internal var mat: Mat = Mat(),
    private val ownsMat: Boolean = true,
    override var tag: String = "",
    private val mask: Mat? = null
) : Pattern {
    private class MaskedMats(val mat: Mat, val mask: Mat?)

    companion object {
        init {
            OpenCv.load()
        }

        private fun decode(bytes: ByteArray, flags: Int): Mat =
            MatOfByte(*bytes).use { Imgcodecs.imdecode(it, flags) }

        private fun makeMat(stream: InputStream, isColor: Boolean): Mat =
            decode(stream.readBytes(), if (isColor) Imgcodecs.IMREAD_COLOR else Imgcodecs.IMREAD_GRAYSCALE)

        private fun makeMaskedMats(stream: InputStream, isColor: Boolean, filePath: String): MaskedMats {
            val rgba = decode(stream.readBytes(), Imgcodecs.IMREAD_UNCHANGED)
            val colorMat = Mat()
            val alpha = Mat()
            val mask = Mat()
            try {
                if (rgba.channels() != 4) {
                    error("$filePath must have an alpha channel")
                }
                Imgproc.cvtColor(rgba, colorMat, if (isColor) Imgproc.COLOR_BGRA2BGR else Imgproc.COLOR_BGRA2GRAY)
                Core.extractChannel(rgba, alpha, 3)
                Core.compare(alpha, Scalar(0.0), mask, Core.CMP_GT)
                return MaskedMats(colorMat, mask)
            } catch (t: Throwable) {
                colorMat.release()
                mask.release()
                throw t
            } finally {
                alpha.release()
                rgba.release()
            }
        }

        fun decodeColor(bytes: ByteArray, tag: String = ""): CvPattern =
            CvPattern(decode(bytes, Imgcodecs.IMREAD_COLOR), tag = tag)
    }

    private constructor(mats: MaskedMats, tag: String) : this(mats.mat, tag = tag, mask = mats.mask)

    constructor(stream: InputStream, isColor: Boolean, tag: String = "") : this(makeMat(stream, isColor), tag = tag)

    constructor(stream: InputStream, isColor: Boolean, tag: String = "", masked: Boolean) : this(
        if (masked) makeMaskedMats(stream, isColor, tag) else MaskedMats(makeMat(stream, isColor), null),
        tag = tag
    )

    override fun toString() = tag.ifBlank { super.toString() }

    override fun close() {
        if (ownsMat) {
            mat.release()
        }
        mask?.release()
    }

    private fun resize(source: Mat, target: Mat, size: Size) {
        Imgproc.resize(
            source, target,
            CvSize(size.width.toDouble(), size.height.toDouble()),
            0.0, 0.0, Imgproc.INTER_AREA
        )
    }

    override fun resize(size: Size): Pattern {
        check(mask == null) { "masked template: resize not supported" }
        val resizedMat = Mat().apply { resize(mat, this, size) }
        return CvPattern(resizedMat, tag = tag)
    }

    override fun resize(target: Pattern, size: Size) {
        check(mask == null && (target as? CvPattern)?.mask == null) { "masked template: resize not supported" }
        if (target is CvPattern) {
            resize(mat, target.mat, size)
        }
        target.tag = tag
    }

    private fun sanitizeMaskedResult(result: Mat) {
        val ok = Mat()
        val bad = Mat()
        try {
            Core.inRange(result, Scalar(-1.01), Scalar(1.01), ok)
            Core.bitwise_not(ok, bad)
            result.setTo(Scalar(0.0), bad)
        } finally {
            bad.release()
            ok.release()
        }
    }

    private fun match(template: Pattern): Mat? {
        if (template !is CvPattern) return null
        if (template.width > width || template.height > height) return null
        val result = Mat()
        val templateMask = template.mask
        if (templateMask != null) {
            Imgproc.matchTemplate(mat, template.mat, result, Imgproc.TM_CCOEFF_NORMED, templateMask)
            sanitizeMaskedResult(result)
        } else {
            Imgproc.matchTemplate(mat, template.mat, result, Imgproc.TM_CCOEFF_NORMED)
        }
        return result
    }

    override fun findMatches(template: Pattern, similarity: Double) = sequence {
        val result = match(template)

        result?.use {
            while (true) {
                val minMaxLocResult = Core.minMaxLoc(it)
                val score = minMaxLocResult.maxVal

                if (score >= similarity) {
                    val loc = minMaxLocResult.maxLoc
                    val region = Region(loc.x.roundToInt(), loc.y.roundToInt(), template.width, template.height)
                    yield(Match(region, score))

                    result.floodFill(loc, 0.3, 0.0)
                } else {
                    break
                }
            }
        }
    }

    override val width get() = mat.width()
    override val height get() = mat.height()

    override fun crop(region: Region): Pattern {
        check(mask == null) { "masked template: crop not supported" }
        val clippedRegion = Region(0, 0, width, height).clip(region)
        val rect = Rect(clippedRegion.x, clippedRegion.y, clippedRegion.width, clippedRegion.height)
        return CvPattern(Mat(mat, rect), tag = tag)
    }

    override fun save(stream: OutputStream) {
        val encoded = MatOfByte()
        try {
            check(Imgcodecs.imencode(".png", mat, encoded)) { "cannot encode $tag as PNG" }
            stream.write(encoded.toArray())
        } finally {
            encoded.release()
        }
    }

    override fun copy(): Pattern {
        check(mask == null) { "masked template: copy not supported" }
        return CvPattern(mat.clone(), tag = tag)
    }

    override fun stackedWith(other: Pattern): Pattern {
        check(mask == null && (other as? CvPattern)?.mask == null) { "masked template: stackedWith not supported" }
        require(other is CvPattern && other.mat.type() == mat.type()) { "stackedWith needs the same image type" }
        val wide = maxOf(width, other.width)
        fun padded(source: Mat): Mat = if (source.width() == wide) source.clone() else Mat().also {
            Core.copyMakeBorder(source, it, 0, 0, 0, wide - source.width(), Core.BORDER_CONSTANT, Scalar.all(0.0))
        }
        val top = padded(mat)
        val bottom = padded(other.mat)
        val result = Mat()
        Core.vconcat(listOf(top, bottom), result)
        top.release()
        bottom.release()
        return CvPattern(result, tag = "$tag+${other.tag}")
    }

    override fun threshold(value: Double): Pattern {
        check(mask == null) { "masked template: threshold not supported" }
        val result = Mat()
        Imgproc.threshold(mat, result, value * 255, 255.0, Imgproc.THRESH_BINARY)
        return CvPattern(result, tag = "$tag[threshold=$value]")
    }

    override fun featureMatches(scene: Pattern, sceneRegion: Region): List<ImageFeatureMatch> {
        if (scene !is CvPattern || mat.channels() != 1 || scene.mat.channels() != 1) return emptyList()
        require(sceneRegion.x >= 0 && sceneRegion.y >= 0 && sceneRegion.right <= scene.width && sceneRegion.bottom <= scene.height)
        val sift = SIFT.create(5000, 3, 0.025)
        val matcher = BFMatcher.create(Core.NORM_L2, false)
        val mapKeys = MatOfKeyPoint()
        val sceneKeys = MatOfKeyPoint()
        val mapDescriptors = Mat()
        val sceneDescriptors = Mat()
        val sceneMask = Mat.zeros(scene.height, scene.width, CvType.CV_8UC1)
        val emptyMask = Mat()
        val matches = mutableListOf<MatOfDMatch>()
        try {
            Imgproc.rectangle(
                sceneMask,
                Point(sceneRegion.x.toDouble(), sceneRegion.y.toDouble()),
                Point((sceneRegion.right - 1).toDouble(), (sceneRegion.bottom - 1).toDouble()),
                Scalar(255.0), Imgproc.FILLED
            )
            sift.detectAndCompute(mat, emptyMask, mapKeys, mapDescriptors)
            sift.detectAndCompute(scene.mat, sceneMask, sceneKeys, sceneDescriptors)
            if (mapDescriptors.empty() || sceneDescriptors.rows() < 2) return emptyList()
            matcher.knnMatch(mapDescriptors, sceneDescriptors, matches, 2)
            val mapPoints = mapKeys.toArray()
            val scenePoints = sceneKeys.toArray()
            return matches.mapNotNull { pair ->
                val a = pair.toArray()
                a.firstOrNull()?.takeIf { a.size == 2 && it.distance < 0.70 * a[1].distance }
            }.sortedBy { it.distance }.distinctBy { it.trainIdx }.map { m ->
                val a = mapPoints[m.queryIdx].pt
                val b = scenePoints[m.trainIdx].pt
                ImageFeatureMatch(a.x, a.y, b.x, b.y)
            }
        } finally {
            matches.forEach { it.release() }
            mapKeys.release(); sceneKeys.release(); mapDescriptors.release(); sceneDescriptors.release()
            sceneMask.release(); emptyMask.release(); sift.clear(); matcher.clear()
        }
    }

    override fun isWhite(): Boolean {
        return Core.minMaxLoc(mat).minVal >= 200
    }

    override fun isBlack(): Boolean {
        return Core.minMaxLoc(mat).maxVal <= 55
    }

    override fun meanColor(): MeanColor? {
        if (mat.empty() || mat.channels() < 3) return null
        val mean = Core.mean(mat)
        return MeanColor(red = mean.`val`[2], green = mean.`val`[1], blue = mean.`val`[0])
    }

    override fun grayPixels(): ByteArray? {
        if (mat.empty() || mat.channels() != 1) return null
        return mat.clone().use { m -> ByteArray(width * height).also { m.get(0, 0, it) } }
    }

    override fun rgbPixels(): ByteArray? {
        if (mat.empty() || mat.channels() != 3) return null
        return Mat().use { rgb ->
            Imgproc.cvtColor(mat, rgb, Imgproc.COLOR_BGR2RGB)
            ByteArray(width * height * 3).also { rgb.get(0, 0, it) }
        }
    }

    override fun floodFill(x: Double, y: Double, maxDiff: Double, newValue: Double): Pattern {
        mat.floodFill(Point(x, y), maxDiff, newValue)
        return this
    }

    override fun fillText(): Pattern {
        check(mask == null) { "masked template: fillText not supported" }
        val fillMask = Mat.zeros(CvSize(width + 2.0, height + 2.0), mat.type())

        for (point in getPointInHoles(mat)) {
            mat.floodFillMask(point, 255.0, fillMask)
        }

        Core.bitwise_not(fillMask, fillMask)
        return CvPattern(fillMask)
    }

    private fun Mat.floodFill(startingPoint: Point, maxDiff: Double, newValue: Double) {
        Mat().use { m ->
            Imgproc.floodFill(
                this, m, startingPoint, Scalar(newValue),
                Rect(),
                Scalar(maxDiff), Scalar(maxDiff),
                Imgproc.FLOODFILL_FIXED_RANGE
            )
        }
    }

    private fun Mat.floodFillMask(startingPoint: Point, newValue: Double, mask: Mat) {
        Imgproc.floodFill(
            this, mask, startingPoint, Scalar(newValue),
            Rect(), Scalar(0.0), Scalar(0.0),
            4 + (newValue.toInt() shl 8) + Imgproc.FLOODFILL_MASK_ONLY
        )
    }

    private fun findTopHolePoint(holePoints: MatOfPoint): Point {
        var topPoint = Point(0.0, Double.MAX_VALUE)
        for (point in holePoints.toList()) {
            if (point.y < topPoint.y) {
                topPoint = point
            }
        }
        return Point(topPoint.x, topPoint.y + 1.0)
    }

    private fun getPointInHoles(img: Mat): List<Point> {
        val work = Mat()
        Core.bitwise_not(img, work)
        val contours = mutableListOf<MatOfPoint>()
        val hierarchy = Mat()
        Imgproc.findContours(work, contours, hierarchy, Imgproc.RETR_TREE, Imgproc.CHAIN_APPROX_SIMPLE)
        val holePoints = mutableListOf<Point>()
        contours.forEachIndexed { index, matOfPoint ->
            val parent = hierarchy[0, index][3].toInt()
            if ((parent >= 0) && (hierarchy[0, parent][3] < 0)) {
                holePoints.add(findTopHolePoint(matOfPoint))
            }
        }
        work.release()
        hierarchy.release()
        return holePoints
    }
}
