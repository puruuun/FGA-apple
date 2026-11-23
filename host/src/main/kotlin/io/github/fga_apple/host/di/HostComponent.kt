package io.github.fga_apple.host.di

import dagger.Binds
import dagger.BindsInstance
import dagger.Component
import dagger.Module
import dagger.Provides
import io.github.fate_grand_automata.scripts.FgoAutomataApi
import io.github.fate_grand_automata.scripts.FgoGameAreaManager
import io.github.fate_grand_automata.scripts.IFgoAutomataApi
import io.github.fate_grand_automata.scripts.IImageLoader
import io.github.fate_grand_automata.scripts.entrypoints.AutoBattle
import io.github.fate_grand_automata.scripts.entrypoints.AutoDetect
import io.github.fate_grand_automata.scripts.locations.IScriptAreaTransforms
import io.github.fate_grand_automata.scripts.locations.ScriptAreaTransforms
import io.github.fate_grand_automata.scripts.modules.RealSupportScreen
import io.github.fate_grand_automata.scripts.modules.SupportScreen
import io.github.fate_grand_automata.scripts.prefs.IAddedFeatureTuning
import io.github.fate_grand_automata.scripts.prefs.IBattleConfig
import io.github.fate_grand_automata.scripts.prefs.IPreferences
import io.github.fate_grand_automata.scripts.prefs.ISupportPreferences
import io.github.fate_grand_automata.scripts.prefs.ISupportPreferencesCommon
import io.github.fga_apple.host.capture.HostScreenshotService
import io.github.fga_apple.host.imaging.HostImageLoader
import io.github.fga_apple.host.input.BoardGestures
import io.github.fga_apple.host.platform.HostPlatform
import io.github.lib_automata.AutomataApi
import io.github.lib_automata.Clicker
import io.github.lib_automata.ExitManager
import io.github.lib_automata.GameAreaManager
import io.github.lib_automata.GestureService
import io.github.lib_automata.Highlighter
import io.github.lib_automata.ImageMatcher
import io.github.lib_automata.PlatformImpl
import io.github.lib_automata.RealClicker
import io.github.lib_automata.RealHighlighter
import io.github.lib_automata.RealImageMatcher
import io.github.lib_automata.RealScale
import io.github.lib_automata.RealSwiper
import io.github.lib_automata.RealTransformer
import io.github.lib_automata.RealWaiter
import io.github.lib_automata.Scale
import io.github.lib_automata.ScreenshotManager
import io.github.lib_automata.ScreenshotService
import io.github.lib_automata.StandardAutomataApi
import io.github.lib_automata.Swiper
import io.github.lib_automata.Transformer
import io.github.lib_automata.Waiter
import io.github.lib_automata.dagger.ScriptScope
import javax.inject.Singleton

/**
 * 스크립트 한 번 실행에 필요한 부품 묶음. Android 앱에서 Hilt의 ScriptComponent와
 * `app/di/script` 폴더의 모듈들이 하던 일을 일반 Dagger로 한다.
 */
@Singleton
@ScriptScope
@Component(modules = [LibAutomataModule::class, ScriptsModule::class, PreferencesModule::class, HostModule::class])
interface HostComponent {
    fun battle(): AutoBattle
    fun autoDetect(): AutoDetect
    fun api(): IFgoAutomataApi
    fun gameArea(): GameAreaManager
    fun scale(): Scale
    fun exitManager(): ExitManager
    fun screenshotManager(): ScreenshotManager
    fun gestures(): GestureService
    fun screenshotService(): ScreenshotService

    @Component.Builder
    interface Builder {
        @BindsInstance fun preferences(prefs: IPreferences): Builder
        @BindsInstance fun config(config: HostConfig): Builder
        fun build(): HostComponent
    }
}

// app/di/script/LibAutomataModule.kt와 같은 연결.
@Module
abstract class LibAutomataModule {
    companion object {
        @ScriptScope
        @Provides
        fun exitManager() = ExitManager()
    }

    @ScriptScope @Binds abstract fun swiper(swiper: RealSwiper): Swiper
    @ScriptScope @Binds abstract fun waiter(waiter: RealWaiter): Waiter
    @ScriptScope @Binds abstract fun highlighter(highlighter: RealHighlighter): Highlighter
    @ScriptScope @Binds abstract fun clicker(clicker: RealClicker): Clicker
    @ScriptScope @Binds abstract fun scale(scale: RealScale): Scale
    @ScriptScope @Binds abstract fun transformer(transformer: RealTransformer): Transformer
    @ScriptScope @Binds abstract fun imageMatcher(imageMatcher: RealImageMatcher): ImageMatcher
    @ScriptScope @Binds abstract fun api(api: StandardAutomataApi): AutomataApi
}

// app/di/script/ScriptsModule.kt에서 화면·게임 로직 쪽 연결.
@Module
abstract class ScriptsModule {
    companion object {
        @ScriptScope
        @Provides
        fun provideGameAreaManager(platformImpl: PlatformImpl): GameAreaManager =
            FgoGameAreaManager(
                gameSizeWithBorders = platformImpl.windowRegion.size,
                offset = { platformImpl.windowRegion.location }
            )
    }

    @ScriptScope @Binds abstract fun api(api: FgoAutomataApi): IFgoAutomataApi
    @ScriptScope @Binds abstract fun scriptAreaTransforms(transforms: ScriptAreaTransforms): IScriptAreaTransforms
    @ScriptScope @Binds abstract fun supportScreen(screen: RealSupportScreen): SupportScreen
}

// app/di/script/PreferencesModule.kt와 같은 연결.
@Module
class PreferencesModule {
    @ScriptScope @Provides
    fun provideAddedFeatureTuning(): IAddedFeatureTuning = IAddedFeatureTuning.Default

    @ScriptScope @Provides
    fun provideBattleConfig(prefs: IPreferences): IBattleConfig = prefs.selectedBattleConfig

    @ScriptScope @Provides
    fun provideSupportPrefs(battleConfig: IBattleConfig): ISupportPreferences = battleConfig.support

    @ScriptScope @Provides
    fun provideCommonSupportPrefs(prefs: IPreferences): ISupportPreferencesCommon = prefs.support
}

// 이 저장소가 직접 구현하는 플랫폼 부품.
@Module
abstract class HostModule {
    @ScriptScope @Binds abstract fun platform(platform: HostPlatform): PlatformImpl
    @ScriptScope @Binds abstract fun gestures(gestures: BoardGestures): GestureService
    @ScriptScope @Binds abstract fun screenshots(service: HostScreenshotService): ScreenshotService
    @ScriptScope @Binds abstract fun imageLoader(loader: HostImageLoader): IImageLoader
    @ScriptScope @Binds abstract fun messages(messages: io.github.fga_apple.host.platform.ConsoleMessages): io.github.fate_grand_automata.scripts.IScriptMessages
    @ScriptScope @Binds abstract fun ocr(ocr: io.github.fga_apple.host.platform.UnsupportedOcr): io.github.lib_automata.OcrService
    @ScriptScope @Binds abstract fun storage(storage: io.github.fga_apple.host.platform.HostStorage): io.github.fate_grand_automata.IStorageProvider

    companion object {
        // Android의 로그 수집(게임 통신 캡처)과 판독 진단 저장은 이쪽에 없다. FGA가 제공하는 "없음" 구현을 쓴다.
        @ScriptScope @Provides
        fun captureFeed(): io.github.fate_grand_automata.scripts.capture.ICaptureFeed =
            io.github.fate_grand_automata.scripts.capture.ICaptureFeed.None

        @ScriptScope @Provides
        fun readerDiagnostics(): io.github.fate_grand_automata.scripts.diagnostics.IReaderDiagnostics =
            io.github.fate_grand_automata.scripts.diagnostics.NoReaderDiagnostics
    }
}
