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

/**
 * Mac에서 받은 iPhone 화면을 FGA가 기대하는 형태로 만든다.
 *
 * Android 구현은 화면을 처음부터 비교 해상도(높이 720)로 캡처해서 넘긴다. 여기서는 원본 해상도로
 * 받은 뒤 같은 배율로 줄여 같은 형태를 만든다. [ColorManager]가 컬러를 요구하지 않으면 흑백으로 준다.
 */
class HostScreenshotService @Inject constructor(
    config: HostConfig,
    private val scale: Scale,
    private val colorManager: ColorManager
) : ScreenshotService {
    private val source = MacFrameSource(config.frameUrl)
    private val expected = Size(config.screenWidth, config.screenHeight)

    // 돌려준 이미지는 다음 호출 전까지만 유효하다 (Android 구현과 같은 규칙).
    private var current: CvPattern? = null

    /** 마지막으로 받은 프레임이 만들어진 뒤 지난 시간(ms). */
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
