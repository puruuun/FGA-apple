package io.github.fga_apple.host.capture

import io.github.fga_apple.host.di.HostConfig
import io.github.fga_apple.host.imaging.CvPattern
import io.github.lib_automata.ColorManager
import io.github.lib_automata.Pattern
import io.github.lib_automata.Scale
import io.github.lib_automata.ScreenshotService
import io.github.lib_automata.Size
import org.opencv.core.Mat
import org.opencv.imgproc.Imgproc
import javax.inject.Inject
import kotlin.math.roundToInt

class HostScreenshotService @Inject constructor(
    config: HostConfig,
    private val scale: Scale,
    private val colorManager: ColorManager
) : ScreenshotService {
    private val source = MacFrameSource(config.frameUrl)
    private val expected = Size(config.screenWidth, config.screenHeight)

    private var current: CvPattern? = null

    @Volatile
    var lastFrameAgeMs: Long = -1
        private set

    @Synchronized
    override fun takeScreenshot(): Pattern {
        val frame = source.grab()
        lastFrameAgeMs = frame.ageMs
        frame.image.use { raw ->
            check(raw.width == expected.width && raw.height == expected.height) {
                "받은 화면이 ${raw.width}x${raw.height}입니다. 설정은 ${expected.width}x${expected.height}입니다. " +
                    "iPhone이 FGO 가로 화면인지 확인하세요."
            }
            val ratio = scale.screenToImage ?: 1.0
            val target = Size((raw.width * ratio).roundToInt(), (raw.height * ratio).roundToInt())

            val converted = if (colorManager.isColor) raw.copy() as CvPattern else {
                val gray = Mat()
                Imgproc.cvtColor(raw.mat, gray, Imgproc.COLOR_BGR2GRAY)
                CvPattern(gray, tag = "screenshot")
            }
            val result = converted.use { it.resize(target) as CvPattern }
            current?.close()
            current = result
            return result
        }
    }

    override fun startRecording(): AutoCloseable? = null

    @Synchronized
    override fun close() {
        current?.close()
        current = null
    }
}

private inline fun <T> CvPattern.use(block: (CvPattern) -> T): T =
    try {
        block(this)
    } finally {
        close()
    }
