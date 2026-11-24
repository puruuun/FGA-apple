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
    override val windowRegion get() = Region(Location(), Size(config.screenWidth, config.screenHeight))

    override val canLongSwipe = true

    override val prefs: PlatformPrefs get() = preferences.platformPrefs

    override fun getResizableBlankPattern(): Pattern = CvPattern()

    override fun highlight(region: Region, duration: Duration, color: HighlightColor) {}
}
