package io.github.fga_apple.host.golden

import dagger.Binds
import dagger.BindsInstance
import dagger.Component
import dagger.Module
import dagger.Provides
import io.github.fate_grand_automata.IStorageProvider
import io.github.fate_grand_automata.scripts.IFgoAutomataApi
import io.github.fate_grand_automata.scripts.IImageLoader
import io.github.fate_grand_automata.scripts.IScriptMessages
import io.github.fate_grand_automata.scripts.Images
import io.github.fate_grand_automata.scripts.ScriptLog
import io.github.fate_grand_automata.scripts.ScriptNotify
import io.github.fate_grand_automata.scripts.capture.ICaptureFeed
import io.github.fate_grand_automata.scripts.diagnostics.IReaderDiagnostics
import io.github.fate_grand_automata.scripts.diagnostics.NoReaderDiagnostics
import io.github.fate_grand_automata.scripts.prefs.IPreferences
import io.github.fga_apple.host.di.HostConfig
import io.github.fga_apple.host.di.LibAutomataModule
import io.github.fga_apple.host.di.PreferencesModule
import io.github.fga_apple.host.di.ScriptsModule
import io.github.fga_apple.host.imaging.CvPattern
import io.github.fga_apple.host.imaging.HostImageLoader
import io.github.fga_apple.host.input.BoardGestures
import io.github.fga_apple.host.platform.HostPlatform
import io.github.fga_apple.host.platform.HostStorage
import io.github.fga_apple.host.platform.UnsupportedOcr
import io.github.lib_automata.ColorManager
import io.github.lib_automata.GameAreaManager
import io.github.lib_automata.GestureService
import io.github.lib_automata.Location
import io.github.lib_automata.OcrService
import io.github.lib_automata.Pattern
import io.github.lib_automata.PlatformImpl
import io.github.lib_automata.Scale
import io.github.lib_automata.ScreenshotService
import io.github.lib_automata.Transformer
import io.github.lib_automata.dagger.ScriptScope
import org.opencv.core.CvType
import org.opencv.core.Mat
import org.opencv.core.Rect
import javax.inject.Inject
import javax.inject.Singleton
import kotlin.math.roundToInt

data class Pasted(val image: Images, val topLeft: Location)

data class GoldenScene(val pasted: Pasted? = null) {
    companion object {
        val Blank = GoldenScene()
    }
}

class GoldenScreenshotService @Inject constructor(
    private val config: HostConfig,
    private val scene: GoldenScene,
    private val scale: Scale,
    private val gameArea: GameAreaManager,
    private val colorManager: ColorManager,
    private val images: IImageLoader
) : ScreenshotService {
    private var current: CvPattern? = null

    @Synchronized
    override fun takeScreenshot(): Pattern {
        val ratio = scale.screenToImage ?: 1.0
        val width = (config.screenWidth * ratio).roundToInt()
        val height = (config.screenHeight * ratio).roundToInt()
        val frame = Mat.zeros(height, width, if (colorManager.isColor) CvType.CV_8UC3 else CvType.CV_8UC1)
        scene.pasted?.let { pasted ->
            val template = (images[pasted.image] as CvPattern).mat
            val x = (gameArea.gameArea.x * ratio + pasted.topLeft.x * scale.scriptToImage).roundToInt()
            val y = (gameArea.gameArea.y * ratio + pasted.topLeft.y * scale.scriptToImage).roundToInt()
            val target = Rect(x, y, template.width(), template.height())
            require(target.x >= 0 && target.y >= 0 && target.x + target.width <= width && target.y + target.height <= height) {
                "${pasted.image} at ${pasted.topLeft} does not fit in the ${width}x$height frame"
            }
            frame.submat(target).use { template.copyTo(it) }
        }
        val result = CvPattern(frame, tag = "golden-screenshot")
        current?.close()
        current = result
        return result
    }

    override fun startRecording(): AutoCloseable? = null

    @Synchronized
    override fun close() {
        current?.close()
        current = null
    }

    private inline fun <T> Mat.use(block: (Mat) -> T): T =
        try {
            block(this)
        } finally {
            release()
        }
}

@Singleton
class GoldenMessages @Inject constructor() : IScriptMessages {
    val logged = mutableListOf<ScriptLog>()

    override fun notify(action: ScriptNotify) {}

    override fun log(item: ScriptLog) {
        logged += item
    }
}

@Module
abstract class GoldenHostModule {
    @ScriptScope @Binds abstract fun platform(platform: HostPlatform): PlatformImpl
    @ScriptScope @Binds abstract fun gestures(gestures: BoardGestures): GestureService
    @ScriptScope @Binds abstract fun screenshots(service: GoldenScreenshotService): ScreenshotService
    @ScriptScope @Binds abstract fun imageLoader(loader: HostImageLoader): IImageLoader
    @ScriptScope @Binds abstract fun messages(messages: GoldenMessages): IScriptMessages
    @ScriptScope @Binds abstract fun ocr(ocr: UnsupportedOcr): OcrService
    @ScriptScope @Binds abstract fun storage(storage: HostStorage): IStorageProvider

    companion object {
        @ScriptScope @Provides
        fun captureFeed(): ICaptureFeed = ICaptureFeed.None

        @ScriptScope @Provides
        fun readerDiagnostics(): IReaderDiagnostics = NoReaderDiagnostics
    }
}

@Singleton
@ScriptScope
@Component(modules = [LibAutomataModule::class, ScriptsModule::class, PreferencesModule::class, GoldenHostModule::class])
interface GoldenComponent {
    fun api(): IFgoAutomataApi
    fun gameArea(): GameAreaManager
    fun scale(): Scale
    fun transformer(): Transformer
    fun gestures(): GestureService
    fun messages(): GoldenMessages
    fun images(): IImageLoader
    fun screenshots(): ScreenshotService

    @Component.Builder
    interface Builder {
        @BindsInstance fun preferences(prefs: IPreferences): Builder
        @BindsInstance fun config(config: HostConfig): Builder
        @BindsInstance fun scene(scene: GoldenScene): Builder
        fun build(): GoldenComponent
    }
}
