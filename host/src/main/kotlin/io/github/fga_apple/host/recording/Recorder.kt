package io.github.fga_apple.host.recording

import kotlinx.serialization.json.JsonObject
import java.io.File
import java.io.IOException
import java.util.concurrent.atomic.AtomicReference

class Recorder private constructor(
    val events: EventLog,
    private val frames: FrameStore,
    private val intervalMs: Long,
    private val clock: RecordingClock,
    private val failureRef: AtomicReference<Throwable?>
) : AutoCloseable {
    val failure: Throwable? get() = failureRef.get()

    @Volatile
    private var stopped = false

    private val thread = Thread(::loop, "recorder-frames").apply { isDaemon = true }

    private fun loop() {
        while (!stopped) {
            val begin = clock.nowMs()
            try {
                frames.captureOnce()
            } catch (e: Throwable) {
                failureRef.compareAndSet(null, e)
                return
            }
            val wait = intervalMs - (clock.nowMs() - begin)
            if (wait > 0) {
                try {
                    Thread.sleep(wait)
                } catch (_: InterruptedException) {
                    return
                }
            }
        }
    }

    fun start() = thread.start()

    override fun close() {
        stopped = true
        thread.interrupt()
        thread.join(5000)
        events.close()
    }

    companion object {
        const val FRAME_INTERVAL_MS = 200L

        fun open(
            dir: File,
            source: RawFrameSource,
            meta: JsonObject,
            clock: RecordingClock = MonotonicClock(),
            intervalMs: Long = FRAME_INTERVAL_MS
        ): Recorder {
            val eventsFile = File(dir, "events.jsonl")
            val framesDir = File(dir, "frames")
            try {
                if (eventsFile.exists()) throw IOException("${eventsFile.path} already exists")
                if (!framesDir.isDirectory && !framesDir.mkdirs()) throw IOException("cannot create ${framesDir.path}")
                File(dir, "meta.json").writeText(meta.toString())
            } catch (e: IOException) {
                throw RecordingException("녹화 폴더를 준비하지 못했습니다 (${dir.path}): ${e.message}", e)
            }
            val failureRef = AtomicReference<Throwable?>()
            val writer = try {
                eventsFile.bufferedWriter(Charsets.UTF_8)
            } catch (e: IOException) {
                throw RecordingException("events.jsonl을 열지 못했습니다: ${e.message}", e)
            }
            val events = EventLog(writer, clock) { failureRef.compareAndSet(null, it) }
            return Recorder(events, FrameStore(framesDir, source, clock), intervalMs, clock, failureRef)
        }
    }
}
