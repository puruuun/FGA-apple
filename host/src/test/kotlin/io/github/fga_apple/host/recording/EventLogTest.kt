package io.github.fga_apple.host.recording

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Assertions.assertThrows
import org.junit.jupiter.api.Test

class EventLogTest {
    private val sink = LineSink()
    private val log = EventLog(sink.writer, FakeClock())

    @Test
    fun everyEventTypeHasTheDocumentedLine() {
        log.screenshot(1)
        log.tap(2, 3, 4, 5)
        log.swipe(6, 7, 8, 9, 10)
        log.scroll(11, 12, 13, 14, 15)
        log.pinch(16, 1, 2, 3, 4, 5, 6, 7, 8, 300, true)
        log.notify(17, "n")
        log.log(18, "l")
        log.exit(19, "done", 2)

        assertEquals(
            listOf(
                """{"t":1,"type":"screenshot"}""",
                """{"t":2,"type":"tap","x":3,"y":4,"times":5}""",
                """{"t":6,"type":"swipe","x1":7,"y1":8,"x2":9,"y2":10}""",
                """{"t":11,"type":"scroll","x1":12,"y1":13,"x2":14,"y2":15}""",
                """{"t":16,"type":"pinch","ax1":1,"ay1":2,"ax2":3,"ay2":4,"bx1":5,"by1":6,"bx2":7,"by2":8,"durationMs":300,"performed":true}""",
                """{"t":17,"type":"notify","text":"n"}""",
                """{"t":18,"type":"log","text":"l"}""",
                """{"t":19,"type":"exit","reason":"done","runs":2}"""
            ),
            sink.lines()
        )
    }

    @Test
    fun textWithNewlinesAndKoreanStaysOnOneLineAndRoundTrips() {
        val text = "[log] 줄바꿈\n\"따옴표\"\t끝"
        log.log(5, text)

        assertEquals(1, sink.lines().size)
        val parsed = Json.parseToJsonElement(sink.lines().single()).jsonObject
        assertEquals(text, parsed.getValue("text").jsonPrimitive.content)
    }

    @Test
    fun aFailedWriteThrowsAndIsReported() {
        var reported: Throwable? = null
        val failing = EventLog(FailingWriter(), FakeClock()) { reported = it }

        val thrown = assertThrows(RecordingException::class.java) { failing.tap(1, 2, 3, 1) }

        assertNotNull(reported)
        assertEquals(thrown, reported)
        assertEquals(true, thrown.message!!.contains("events.jsonl"))
    }

    @Test
    fun metaHasTheDocumentedKeys() {
        val battle: JsonObject = Json.parseToJsonElement("""{"support_mode":"First"}""").jsonObject
        val meta = recordingMeta(2622, 1206, "Kr", battle, "abc123")

        assertEquals(
            """{"formatVersion":1,"screenWidth":2622,"screenHeight":1206,"gameServer":"Kr",""" +
                """"battleConfig":{"support_mode":"First"},"fgaCommit":"abc123"}""",
            meta.toString()
        )
        assertEquals(
            """null""",
            recordingMeta(1, 2, "En", JsonObject(emptyMap()), null).getValue("fgaCommit").toString()
        )
    }
}
