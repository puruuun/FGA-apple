package io.github.fga_apple.host.platform

import io.github.fate_grand_automata.scripts.prefs.IPreferences
import io.github.fga_apple.host.di.HostConfig
import io.github.fga_apple.host.imaging.CvPattern
import io.github.lib_automata.HighlightColor
import io.github.lib_automata.Location
import io.github.lib_automata.Pattern
import io.github.lib_automata.PlatformImpl
import io.github.lib_automata.PlatformPrefs
import io.github.lib_automata.Region
import io.github.lib_automata.Size
import javax.inject.Inject
import kotlin.time.Duration

class HostPlatform @Inject constructor(
    private val preferences: IPreferences,
    private val config: HostConfig
) : PlatformImpl {
    // iPhone의 FGO는 화면 전체를 쓴다. Android처럼 노치만큼 잘라 내지 않는다.
    override val windowRegion get() = Region(Location(), Size(config.screenWidth, config.screenHeight))

    override val canLongSwipe = true

    override val prefs: PlatformPrefs get() = preferences.platformPrefs

    override fun getResizableBlankPattern(): Pattern = CvPattern()

    // iPhone 화면 위에 표시를 그릴 방법이 없다. Android에서도 디버그 모드의 시각 보조일 뿐이다.
    override fun highlight(region: Region, duration: Duration, color: HighlightColor) {}
}
