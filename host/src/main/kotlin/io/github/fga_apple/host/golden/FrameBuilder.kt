package io.github.fga_apple.host.golden

import org.opencv.core.Core
import org.opencv.core.CvType
import org.opencv.core.Mat
import org.opencv.core.MatOfByte
import org.opencv.core.MatOfInt
import org.opencv.core.Rect
import org.opencv.core.Scalar
import org.opencv.core.Size
import org.opencv.imgcodecs.Imgcodecs
import org.opencv.imgproc.Imgproc

enum class Variant(val id: String) {
    AsIs("as_is"),
    Brightness("brightness"),
    HalfPixel("half_pixel"),
    PartlyOutside("partly_outside"),
    Absent("absent"),
    Recorded("recorded")
}

data class Box(val x: Int, val y: Int, val width: Int, val height: Int) {
    val right get() = x + width
    val bottom get() = y + height
}

data class CaseSpec(
    val template: Template,
    val variant: Variant,
    val kind: String,
    val pasted: Box?,
    val region: Box,
    val params: Map<String, Double>
)

class SyntheticFrame(val name: String, val seed: Long, val mat: Mat, val cases: List<CaseSpec>)

object FrameBuilder {
    const val WIDTH = 1565
    const val HEIGHT = 720
    const val FRAME_COUNT = 14
    const val NOISE_LEVELS = 16
    const val NOISE_STEP = 255 / (NOISE_LEVELS - 1)
    const val COLUMNS = 6
    const val CELL_WIDTH = 260
    const val CELL_HEIGHT = 240
    const val PER_FRAME_UNMASKED = 10
    const val ABSENT_PER_FRAME = 2
    const val REGION_MARGIN = 20
    const val SEED_BASE = 0x46474120L
    private val PASTED_VARIANTS = listOf(Variant.AsIs, Variant.Brightness, Variant.HalfPixel, Variant.PartlyOutside)
    private val MASKED_VARIANTS = PASTED_VARIANTS + Variant.Absent
    private val CONTRASTS = listOf(0.7, 0.85, 1.15, 1.3)
    private val BRIGHTNESS = listOf(-40.0, -20.0, 20.0, 40.0)
    private val WHOLE_FRAME_SLOTS = setOf(0, 3, 6)

    fun seedFor(index: Int): Long = SEED_BASE * 1000003L + index.toLong() * 7919L + 1L

    fun frameName(index: Int) = "frame_%02d".format(index)

    fun noise(rng: Xorshift): Mat {
        val bytes = ByteArray(WIDTH * HEIGHT) { (rng.nextInt(NOISE_LEVELS) * NOISE_STEP).toByte() }
        val mat = Mat(HEIGHT, WIDTH, CvType.CV_8UC1)
        mat.put(0, 0, bytes)
        return mat
    }

    fun build(index: Int, templates: List<Template>, seed: Long = seedFor(index)): SyntheticFrame {
        val rng = Xorshift(seed)
        val frame = noise(rng)
        val masked = templates.filter { it.masked }
        val plain = templates.filter { !it.masked }
        val cases = mutableListOf<CaseSpec>()
        var cell = 0

        fun place(template: Template, variant: Variant, slot: Int) {
            val cellX = (cell % COLUMNS) * CELL_WIDTH
            val cellY = (cell / COLUMNS) * CELL_HEIGHT
            cell++
            if (variant == Variant.Absent) {
                val region = Box(cellX, cellY, CELL_WIDTH, CELL_HEIGHT)
                cases += CaseSpec(template, variant, "noise", null, region, emptyMap())
                return
            }
            val x = cellX + rng.nextInt(CELL_WIDTH - template.width + 1)
            val y = cellY + rng.nextInt(CELL_HEIGHT - template.height + 1)
            val params = mutableMapOf<String, Double>()
            val source = when (variant) {
                Variant.Brightness -> {
                    val contrast = CONTRASTS[rng.nextInt(CONTRASTS.size)]
                    val brightness = BRIGHTNESS[rng.nextInt(BRIGHTNESS.size)]
                    params["brightness"] = brightness
                    params["contrast"] = contrast
                    Mat().also { template.gray.convertTo(it, -1, contrast, brightness) }
                }
                Variant.HalfPixel -> {
                    params["shift_x"] = 0.5
                    params["shift_y"] = 0.5
                    shifted(template.gray)
                }
                else -> template.gray.clone()
            }
            paste(frame, source, template.mask, x, y)
            source.release()
            val pasted = Box(x, y, template.width, template.height)
            val local = if (variant == Variant.PartlyOutside) partlyOutsideRegion(pasted) else marginRegion(pasted)
            cases += CaseSpec(template, variant, "local", pasted, local, params)
            if (slot in WHOLE_FRAME_SLOTS) {
                cases += CaseSpec(template, variant, "whole_frame", pasted, Box(0, 0, WIDTH, HEIGHT), params)
            }
        }

        masked.forEachIndexed { i, template ->
            place(template, MASKED_VARIANTS[(index + i) % MASKED_VARIANTS.size], 0)
        }
        for (i in 0 until PER_FRAME_UNMASKED) {
            val template = plain[(index * PER_FRAME_UNMASKED + i) % plain.size]
            place(template, PASTED_VARIANTS[(index + i) % PASTED_VARIANTS.size], masked.size + i)
        }
        for (j in 0 until ABSENT_PER_FRAME) {
            val template = plain[((index + 1) * PER_FRAME_UNMASKED + j) % plain.size]
            place(template, Variant.Absent, -1)
        }
        return SyntheticFrame(frameName(index), seed, frame, cases)
    }

    fun encodePng(mat: Mat): ByteArray {
        val buffer = MatOfByte()
        val flags = MatOfInt(Imgcodecs.IMWRITE_PNG_COMPRESSION, 9)
        check(Imgcodecs.imencode(".png", mat, buffer, flags)) { "cannot encode PNG" }
        return buffer.toArray().also { buffer.release() }
    }

    private fun shifted(source: Mat): Mat {
        val transform = Mat(2, 3, CvType.CV_64FC1)
        transform.put(0, 0, 1.0, 0.0, 0.5, 0.0, 1.0, 0.5)
        val out = Mat()
        Imgproc.warpAffine(
            source, out, transform, Size(source.cols().toDouble(), source.rows().toDouble()),
            Imgproc.INTER_LINEAR, Core.BORDER_REPLICATE, Scalar(0.0)
        )
        transform.release()
        return out
    }

    private fun paste(frame: Mat, source: Mat, mask: Mat?, x: Int, y: Int) {
        val target = frame.submat(Rect(x, y, source.cols(), source.rows()))
        if (mask != null) source.copyTo(target, mask) else source.copyTo(target)
        target.release()
    }

    private fun marginRegion(pasted: Box): Box {
        val x0 = maxOf(0, pasted.x - REGION_MARGIN)
        val y0 = maxOf(0, pasted.y - REGION_MARGIN)
        val x1 = minOf(WIDTH, pasted.right + REGION_MARGIN)
        val y1 = minOf(HEIGHT, pasted.bottom + REGION_MARGIN)
        return Box(x0, y0, x1 - x0, y1 - y0)
    }

    private fun partlyOutsideRegion(pasted: Box): Box {
        val width = pasted.width + 10
        val height = pasted.height + 30
        val y = (pasted.y - 15).coerceIn(0, HEIGHT - height)
        val rightEdge = pasted.x + pasted.width / 2
        val x = if (rightEdge + width <= WIDTH) rightEdge else rightEdge - width
        return Box(x, y, width, height)
    }
}
