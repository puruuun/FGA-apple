package io.github.fga_apple.host.recording

import io.github.lib_automata.GestureService
import io.github.lib_automata.Location
import io.github.lib_automata.Pattern
import io.github.lib_automata.ScreenshotService
import java.io.IOException
import java.io.StringWriter
import java.io.Writer
import java.lang.reflect.Proxy

class FakeClock(var now: Long = 0) : RecordingClock {
    override fun nowMs() = now
}

class FakeGestures(
    private val calls: MutableList<String>,
    var pinchResult: Boolean = false,
    private val duringCall: () -> Unit = {}
) : GestureService {
    var closed = false

    override fun swipe(start: Location, end: Location) {
        calls += "swipe $start $end"
        duringCall()
    }

    override fun scroll(start: Location, end: Location) {
        calls += "scroll $start $end"
        duringCall()
    }

    override fun pinch(startA: Location, endA: Location, startB: Location, endB: Location, durationMs: Long): Boolean {
        calls += "pinch $startA $endA $startB $endB $durationMs"
        duringCall()
        return pinchResult
    }

    override fun click(location: Location, times: Int) {
        calls += "click $location $times"
        duringCall()
    }

    override fun close() {
        closed = true
    }
}

class FakeScreenshots(private val calls: MutableList<String>) : ScreenshotService {
    val pattern: Pattern = Proxy.newProxyInstance(
        Pattern::class.java.classLoader, arrayOf(Pattern::class.java)
    ) { _, method, _ -> throw UnsupportedOperationException(method.name) } as Pattern

    override fun takeScreenshot(): Pattern {
        calls += "screenshot"
        return pattern
    }

    override fun startRecording(): AutoCloseable? = null

    override fun close() {}
}

class FakeFrames(private val frames: List<ByteArray>) : RawFrameSource {
    private var index = 0
    val fetches get() = index

    override fun fetch(): ByteArray = frames[minOf(index++, frames.lastIndex)]
}

class FailingWriter : Writer() {
    override fun write(cbuf: CharArray, off: Int, len: Int) = throw IOException("disk full")
    override fun flush() {}
    override fun close() {}
}

class LineSink {
    val writer = StringWriter()
    fun lines(): List<String> = writer.toString().split("\n").filter { it.isNotEmpty() }
}
