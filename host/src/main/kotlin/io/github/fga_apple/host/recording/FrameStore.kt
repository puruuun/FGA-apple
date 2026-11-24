package io.github.fga_apple.host.recording

import java.io.File
import java.io.IOException
import java.net.HttpURLConnection
import java.net.URI

fun interface RawFrameSource {
    fun fetch(): ByteArray
}

class HttpFrameSource(private val baseUrl: String) : RawFrameSource {
    override fun fetch(): ByteArray {
        val conn = URI("$baseUrl/frame").toURL().openConnection() as HttpURLConnection
        conn.connectTimeout = 3000
        conn.readTimeout = 5000
        try {
            check(conn.responseCode == 200) { "frame server answered ${conn.responseCode}" }
            return conn.inputStream.use { it.readBytes() }
        } finally {
            conn.disconnect()
        }
    }
}

class FrameStore(
    private val dir: File,
    private val source: RawFrameSource,
    private val clock: RecordingClock
) {
    private var last: ByteArray? = null

    fun captureOnce(): Boolean {
        val bytes = try {
            source.fetch()
        } catch (e: Exception) {
            throw RecordingException("녹화용 화면을 frame server에서 받지 못했습니다: ${e.message}", e)
        }
        val t = clock.nowMs()
        if (last?.contentEquals(bytes) == true) return false
        val file = File(dir, "%08d.jpg".format(t))
        try {
            if (file.exists()) throw IOException("${file.name} already exists")
            file.writeBytes(bytes)
        } catch (e: IOException) {
            throw RecordingException("화면 파일을 쓰지 못했습니다 (${file.path}): ${e.message}", e)
        }
        last = bytes
        return true
    }
}
