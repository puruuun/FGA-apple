package io.github.fga_apple.host.recording

import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Assertions.assertThrows
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.io.File

class FrameStoreTest {
    @TempDir
    lateinit var dir: File

    private val a = byteArrayOf(-1, -40, 1, 2, 3)
    private val b = byteArrayOf(-1, -40, 9, 9, 9)

    private fun names() = dir.list()!!.sorted()

    @Test
    fun framesAreStoredByteForByteUnderPaddedMilliseconds() {
        val clock = FakeClock(1234)
        val store = FrameStore(dir, FakeFrames(listOf(a)), clock)

        assertTrue(store.captureOnce())

        assertEquals(listOf("00001234.jpg"), names())
        assertArrayEquals(a, File(dir, "00001234.jpg").readBytes())
    }

    @Test
    fun aFrameEqualToTheLastStoredOneIsSkipped() {
        val clock = FakeClock()
        val store = FrameStore(dir, FakeFrames(listOf(a, a.copyOf(), b, b.copyOf(), a)), clock)

        val stored = (0 until 5).map {
            clock.now = it * 200L
            store.captureOnce()
        }

        assertEquals(listOf(true, false, true, false, true), stored)
        assertEquals(listOf("00000000.jpg", "00000400.jpg", "00000800.jpg"), names())
        assertArrayEquals(b, File(dir, "00000400.jpg").readBytes())
        assertArrayEquals(a, File(dir, "00000800.jpg").readBytes())
    }

    @Test
    fun aFailedFrameWritePropagates() {
        val missing = File(dir, "not-created")
        val store = FrameStore(missing, FakeFrames(listOf(a)), FakeClock())

        val thrown = assertThrows(RecordingException::class.java) { store.captureOnce() }

        assertTrue(thrown.message!!.contains("00000000.jpg"))
    }

    @Test
    fun aFailedFrameFetchPropagates() {
        val source = RawFrameSource { throw IllegalStateException("frame server answered 503") }
        val store = FrameStore(dir, source, FakeClock())

        val thrown = assertThrows(RecordingException::class.java) { store.captureOnce() }

        assertTrue(thrown.message!!.contains("503"))
        assertEquals(emptyList<String>(), names())
    }

    @Test
    fun recorderThreadReportsAFrameFailure() {
        val source = RawFrameSource { throw IllegalStateException("frame server down") }
        val recorder = Recorder.open(dir, source, recordingMeta(1, 1, "Kr", kotlinx.serialization.json.JsonObject(emptyMap()), null), intervalMs = 5)

        recorder.start()
        val deadline = System.nanoTime() + 5_000_000_000L
        while (recorder.failure == null && System.nanoTime() < deadline) Thread.sleep(10)
        recorder.close()

        assertNotNull(recorder.failure)
        assertTrue(recorder.failure!!.message!!.contains("frame server down"))
    }

    @Test
    fun recorderThreadStoresFramesAndKeepsGoing() {
        val frames = FakeFrames(listOf(a, a, b))
        val recorder = Recorder.open(dir, frames, recordingMeta(1, 1, "Kr", kotlinx.serialization.json.JsonObject(emptyMap()), null), intervalMs = 5)

        recorder.start()
        val deadline = System.nanoTime() + 5_000_000_000L
        while (frames.fetches < 6 && System.nanoTime() < deadline) Thread.sleep(10)
        recorder.close()

        assertNull(recorder.failure)
        assertEquals(2, File(dir, "frames").list()!!.size)
        assertTrue(File(dir, "meta.json").isFile)
        assertTrue(File(dir, "events.jsonl").isFile)
    }

    @Test
    fun anExistingRecordingIsNotOverwritten() {
        File(dir, "events.jsonl").writeText("")
        val meta = recordingMeta(1, 1, "Kr", kotlinx.serialization.json.JsonObject(emptyMap()), null)

        assertThrows(RecordingException::class.java) { Recorder.open(dir, FakeFrames(listOf(a)), meta) }
    }
}
