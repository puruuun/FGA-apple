package io.github.fga_apple.host.input

import io.github.fate_grand_automata.scripts.prefs.IPreferences
import io.github.fga_apple.host.di.HostConfig
import io.github.lib_automata.GestureService
import io.github.lib_automata.Location
import io.github.lib_automata.Waiter
import io.github.lib_automata.dagger.ScriptScope
import javax.inject.Inject

private const val COORD_MAX = 32767

/**
 * 스크립트의 탭·스와이프를 ESP32 명령으로 바꿔 보낸다.
 * 들어오는 좌표는 iPhone 화면 픽셀이고, 펌웨어 좌표(0..32767)로 바꾼다.
 */
@ScriptScope
class BoardGestures @Inject constructor(
    private val config: HostConfig,
    private val prefs: IPreferences,
    private val wait: Waiter
) : GestureService {
    private val board: Board? = if (config.dryRun) null else Board(config.serialPort).also {
        val stat = it.require("STAT")
        check("connected=1" in stat) { "보드가 iPhone에 연결돼 있지 않습니다: $stat" }
        // 띠 회피 설정은 보드 전원을 켤 때마다 지워지므로 매번 보낸다.
        it.require("AVOID ${config.avoidLo} ${config.avoidHi}")
    }

    private fun x(px: Int) = (px.toLong() * COORD_MAX / config.screenWidth).toInt().coerceIn(0, COORD_MAX)
    private fun y(px: Int) = (px.toLong() * COORD_MAX / config.screenHeight).toInt().coerceIn(0, COORD_MAX)

    private fun send(line: String, timeoutMs: Long = 5000) {
        if (board == null) println("[dry-run] $line") else board.require(line, timeoutMs)
    }

    override fun click(location: Location, times: Int) {
        val hold = prefs.gestures.clickDuration.inWholeMilliseconds
        repeat(times) { send("TAP ${x(location.x)} ${y(location.y)} $hold") }
        wait(prefs.gestures.clickWaitTime)
    }

    private fun drag(start: Location, end: Location, durationMs: Long) {
        send("SWIPE ${x(start.x)} ${y(start.y)} ${x(end.x)} ${y(end.y)} $durationMs", durationMs + 5000)
        wait(prefs.gestures.swipeWaitTime)
    }

    override fun swipe(start: Location, end: Location) =
        drag(start, end, prefs.gestures.swipeDuration.inWholeMilliseconds)

    // 펌웨어의 스와이프는 끝으로 갈수록 감속해서 관성 스크롤이 생기지 않는다(README의 실측).
    // Android 구현과 같이 스크롤은 600 ms 이상으로 한다.
    override fun scroll(start: Location, end: Location) =
        drag(start, end, maxOf(600L, prefs.gestures.swipeDuration.inWholeMilliseconds))

    // 블루투스 마우스는 포인터가 하나라 두 손가락 동작을 낼 수 없다.
    override fun pinch(startA: Location, endA: Location, startB: Location, endB: Location, durationMs: Long) = false

    override fun close() {
        board?.close()
    }
}
