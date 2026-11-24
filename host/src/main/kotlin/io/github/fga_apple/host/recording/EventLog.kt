package io.github.fga_apple.host.recording

import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import java.io.IOException
import java.io.Writer

class RecordingException(message: String, cause: Throwable? = null) : RuntimeException(message, cause)

fun interface RecordingClock {
    fun nowMs(): Long
}

class MonotonicClock : RecordingClock {
    private val origin = System.nanoTime()
    override fun nowMs(): Long = (System.nanoTime() - origin) / 1_000_000
}

class EventLog(
    private val out: Writer,
    private val clock: RecordingClock,
    private val onFailure: (Throwable) -> Unit = {}
) : AutoCloseable {
    fun now(): Long = clock.nowMs()

    fun screenshot(t: Long) = write(t, "screenshot") {}

    fun tap(t: Long, x: Int, y: Int, times: Int) = write(t, "tap") {
        put("x", x)
        put("y", y)
        put("times", times)
    }

    fun swipe(t: Long, x1: Int, y1: Int, x2: Int, y2: Int) = write(t, "swipe") { line(x1, y1, x2, y2) }

    fun scroll(t: Long, x1: Int, y1: Int, x2: Int, y2: Int) = write(t, "scroll") { line(x1, y1, x2, y2) }

    fun pinch(
        t: Long,
        ax1: Int, ay1: Int, ax2: Int, ay2: Int,
        bx1: Int, by1: Int, bx2: Int, by2: Int,
        durationMs: Long, performed: Boolean
    ) = write(t, "pinch") {
        put("ax1", ax1)
        put("ay1", ay1)
        put("ax2", ax2)
        put("ay2", ay2)
        put("bx1", bx1)
        put("by1", by1)
        put("bx2", bx2)
        put("by2", by2)
        put("durationMs", durationMs)
        put("performed", performed)
    }

    fun notify(t: Long, text: String) = write(t, "notify") { put("text", text) }

    fun log(t: Long, text: String) = write(t, "log") { put("text", text) }

    fun exit(t: Long, reason: String, runs: Int) = write(t, "exit") {
        put("reason", reason)
        put("runs", runs)
    }

    private fun kotlinx.serialization.json.JsonObjectBuilder.line(x1: Int, y1: Int, x2: Int, y2: Int) {
        put("x1", x1)
        put("y1", y1)
        put("x2", x2)
        put("y2", y2)
    }

    @Synchronized
    private fun write(t: Long, type: String, fields: kotlinx.serialization.json.JsonObjectBuilder.() -> Unit) {
        val line = buildJsonObject {
            put("t", t)
            put("type", JsonPrimitive(type))
            fields()
        }
        try {
            out.write(line.toString())
            out.write("\n")
            out.flush()
        } catch (e: IOException) {
            val failure = RecordingException("events.jsonl에 쓰지 못했습니다: ${e.message}", e)
            onFailure(failure)
            throw failure
        }
    }

    @Synchronized
    override fun close() {
        out.close()
    }
}

fun recordingMeta(
    screenWidth: Int,
    screenHeight: Int,
    gameServer: String,
    battleConfig: JsonObject,
    fgaCommit: String?
): JsonObject = buildJsonObject {
    put("formatVersion", 1)
    put("screenWidth", screenWidth)
    put("screenHeight", screenHeight)
    put("gameServer", gameServer)
    put("battleConfig", battleConfig)
    put("fgaCommit", fgaCommit)
}
