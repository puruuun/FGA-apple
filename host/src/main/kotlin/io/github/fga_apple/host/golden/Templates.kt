package io.github.fga_apple.host.golden

import io.github.fate_grand_automata.scripts.Images
import io.github.fga_apple.host.imaging.CvPattern
import io.github.fga_apple.host.imaging.OpenCv
import org.opencv.core.Core
import org.opencv.core.Mat
import org.opencv.core.MatOfByte
import org.opencv.core.MatOfDouble
import org.opencv.core.Scalar
import org.opencv.imgcodecs.Imgcodecs
import java.io.File

class Template(
    val name: String,
    val masked: Boolean,
    val gray: Mat,
    val mask: Mat?,
    val pattern: CvPattern
) {
    val width get() = gray.cols()
    val height get() = gray.rows()
}

object Templates {
    const val SERVER = "En"
    const val MIN_STD_DEV = 4.0

    class Loaded(val usable: List<Template>, val skipped: List<String>)

    fun load(assetsDir: File): Loaded {
        OpenCv.load()
        val usable = mutableListOf<Template>()
        val skipped = mutableListOf<String>()
        val entries = Images.values().distinctBy { it.path }.sortedBy { it.path }
        for (image in entries) {
            val file = File(File(assetsDir, SERVER), image.path)
            if (!file.isFile) {
                skipped += "${image.path}: no file in $SERVER"
                continue
            }
            val pattern = file.inputStream().use { CvPattern(it, false, image.path, image.masked) }
            val mask = if (image.masked) alphaMask(file.readBytes()) else null
            val template = Template(image.path, image.masked, pattern.mat.clone(), mask, pattern)
            val spread = stdDev(template)
            if (spread < MIN_STD_DEV) {
                skipped += "${image.path}: flat template (std dev ${"%.2f".format(spread)})"
                continue
            }
            usable += template
        }
        return Loaded(usable, skipped)
    }

    private fun alphaMask(bytes: ByteArray): Mat {
        val rgba = Imgcodecs.imdecode(MatOfByte(*bytes), Imgcodecs.IMREAD_UNCHANGED)
        check(rgba.channels() == 4) { "masked template needs an alpha channel" }
        val alpha = Mat()
        val mask = Mat()
        Core.extractChannel(rgba, alpha, 3)
        Core.compare(alpha, Scalar(0.0), mask, Core.CMP_GT)
        alpha.release()
        rgba.release()
        return mask
    }

    private fun stdDev(template: Template): Double {
        val mean = MatOfDouble()
        val std = MatOfDouble()
        if (template.mask != null) Core.meanStdDev(template.gray, mean, std, template.mask)
        else Core.meanStdDev(template.gray, mean, std)
        return std.toArray()[0]
    }
}
