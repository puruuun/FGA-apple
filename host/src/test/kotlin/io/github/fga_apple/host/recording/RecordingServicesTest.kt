package io.github.fga_apple.host.recording

import io.github.fate_grand_automata.scripts.IScriptMessages
import io.github.fate_grand_automata.scripts.ScriptLog
import io.github.fate_grand_automata.scripts.ScriptNotify
import io.github.fga_apple.host.platform.logLine
import io.github.fga_apple.host.platform.notifyLine
import io.github.lib_automata.Location
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertSame
import org.junit.jupiter.api.Assertions.assertThrows
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

class RecordingServicesTest {
    private val clock = FakeClock()
    private val sink = LineSink()
    private val events = EventLog(sink.writer, clock)
    private val calls = mutableListOf<String>()

    @Test
    fun gestureDecoratorForwardsEveryCallUnchangedAndInOrder() {
        val inner = FakeGestures(calls)
        val gestures = RecordingGestureService(inner, events)
        val a = Location(10, 20)
        val b = Location(30, 40)

        gestures.click(a, 3)
        gestures.swipe(a, b)
        gestures.scroll(b, a)
        gestures.pinch(a, b, b, a, 450)
        gestures.click(b)

        assertEquals(
            listOf(
                "click Location(x=10, y=20) 3",
                "swipe Location(x=10, y=20) Location(x=30, y=40)",
                "scroll Location(x=30, y=40) Location(x=10, y=20)",
                "pinch Location(x=10, y=20) Location(x=30, y=40) Location(x=30, y=40) Location(x=10, y=20) 450",
                "click Location(x=30, y=40) 1"
            ),
            calls
        )
        assertEquals(5, sink.lines().size)
    }

    @Test
    fun pinchResultIsReturnedUnchanged() {
        val inner = FakeGestures(calls)
        val gestures = RecordingGestureService(inner, events)
        val p = Location(1, 2)

        inner.pinchResult = false
        assertFalse(gestures.pinch(p, p, p, p, 1))
        inner.pinchResult = true
        assertTrue(gestures.pinch(p, p, p, p, 1))
        assertTrue(sink.lines().last().contains("\"performed\":true"))
    }

    @Test
    fun gestureEventsCarryStartTimeAndCoordinates() {
        val gestures = RecordingGestureService(FakeGestures(calls, duringCall = { clock.now += 300 }), events)
        clock.now = 1500
        gestures.click(Location(100, 200), 2)
        clock.now = 2500
        gestures.swipe(Location(1, 2), Location(3, 4))
        clock.now = 3500
        gestures.scroll(Location(5, 6), Location(7, 8))

        assertEquals(
            listOf(
                """{"t":1500,"type":"tap","x":100,"y":200,"times":2}""",
                """{"t":2500,"type":"swipe","x1":1,"y1":2,"x2":3,"y2":4}""",
                """{"t":3500,"type":"scroll","x1":5,"y1":6,"x2":7,"y2":8}"""
            ),
            sink.lines()
        )
    }

    @Test
    fun gestureCloseReachesTheWrappedService() {
        val inner = FakeGestures(calls)
        RecordingGestureService(inner, events).close()
        assertTrue(inner.closed)
    }

    @Test
    fun gestureIsSentBeforeItIsLogged() {
        val inner = FakeGestures(calls)
        val failing = RecordingGestureService(inner, EventLog(FailingWriter(), clock))

        assertThrows(RecordingException::class.java) { failing.click(Location(5, 5), 1) }

        assertEquals(listOf("click Location(x=5, y=5) 1"), calls)
    }

    @Test
    fun screenshotDecoratorReturnsTheSameFrameAndLogsTheRequest() {
        val inner = FakeScreenshots(calls)
        val shots = RecordingScreenshotService(inner, events)
        clock.now = 700

        val shot = shots.takeScreenshot()

        assertSame(inner.pattern, shot)
        assertEquals(listOf("""{"t":700,"type":"screenshot"}"""), sink.lines())
    }

    @Test
    fun messagesDecoratorForwardsAndLogsTheConsoleText() {
        val seen = mutableListOf<String>()
        val inner = object : IScriptMessages {
            override fun notify(action: ScriptNotify) {
                seen += "notify ${action.javaClass.simpleName}"
            }

            override fun log(item: ScriptLog) {
                seen += "log ${item.javaClass.simpleName}"
            }
        }
        val messages = RecordingMessages(inner, events)
        clock.now = 40

        messages.notify(ScriptNotify.CEGet)
        clock.now = 90
        messages.log(ScriptLog.CaptureMode(true))

        assertEquals(listOf("notify CEGet", "log CaptureMode"), seen)
        assertEquals(
            listOf(
                """{"t":40,"type":"notify","text":"[notify] CEGet"}""",
                """{"t":90,"type":"log","text":"[log] CaptureMode(enabled=true)"}"""
            ),
            sink.lines()
        )
        assertEquals("[notify] CEGet", notifyLine(ScriptNotify.CEGet))
        assertEquals("[log] CaptureMode(enabled=true)", logLine(ScriptLog.CaptureMode(true)))
    }
}
