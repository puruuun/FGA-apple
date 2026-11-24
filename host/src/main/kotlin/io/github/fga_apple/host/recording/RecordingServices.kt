package io.github.fga_apple.host.recording

import io.github.fate_grand_automata.scripts.IScriptMessages
import io.github.fate_grand_automata.scripts.ScriptLog
import io.github.fate_grand_automata.scripts.ScriptNotify
import io.github.fga_apple.host.platform.logLine
import io.github.fga_apple.host.platform.notifyLine
import io.github.lib_automata.GestureService
import io.github.lib_automata.Location
import io.github.lib_automata.Pattern
import io.github.lib_automata.ScreenshotService

class RecordingGestureService(
    private val inner: GestureService,
    private val events: EventLog
) : GestureService {
    override fun click(location: Location, times: Int) {
        val t = events.now()
        inner.click(location, times)
        events.tap(t, location.x, location.y, times)
    }

    override fun swipe(start: Location, end: Location) {
        val t = events.now()
        inner.swipe(start, end)
        events.swipe(t, start.x, start.y, end.x, end.y)
    }

    override fun scroll(start: Location, end: Location) {
        val t = events.now()
        inner.scroll(start, end)
        events.scroll(t, start.x, start.y, end.x, end.y)
    }

    override fun pinch(startA: Location, endA: Location, startB: Location, endB: Location, durationMs: Long): Boolean {
        val t = events.now()
        val performed = inner.pinch(startA, endA, startB, endB, durationMs)
        events.pinch(
            t,
            startA.x, startA.y, endA.x, endA.y,
            startB.x, startB.y, endB.x, endB.y,
            durationMs, performed
        )
        return performed
    }

    override fun close() = inner.close()
}

class RecordingScreenshotService(
    private val inner: ScreenshotService,
    private val events: EventLog
) : ScreenshotService {
    override fun takeScreenshot(): Pattern {
        val t = events.now()
        val shot = inner.takeScreenshot()
        events.screenshot(t)
        return shot
    }

    override fun startRecording(): AutoCloseable? = inner.startRecording()

    override fun close() = inner.close()
}

class RecordingMessages(
    private val inner: IScriptMessages,
    private val events: EventLog
) : IScriptMessages {
    override val supportCeDiagnosticsEnabled: Boolean get() = inner.supportCeDiagnosticsEnabled

    override fun notify(action: ScriptNotify) {
        val t = events.now()
        inner.notify(action)
        events.notify(t, notifyLine(action))
    }

    override fun log(item: ScriptLog) {
        val t = events.now()
        inner.log(item)
        events.log(t, logLine(item))
    }
}
