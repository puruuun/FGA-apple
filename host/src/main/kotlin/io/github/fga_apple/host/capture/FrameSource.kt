package io.github.fga_apple.host.capture

import io.github.fga_apple.host.imaging.CvPattern
import java.net.HttpURLConnection
import java.net.URI

/** 화면 한 장과, 그 화면이 만들어진 뒤 지난 시간. */
class Frame(val image: CvPattern, val ageMs: Long)

/** Mac의 `mac-capture/frame_server.py`에서 최신 iPhone 화면을 받아 온다. */
class MacFrameSource(private val baseUrl: String) {
    fun grab(): Frame {
        val conn = URI("$baseUrl/frame").toURL().openConnection() as HttpURLConnection
        conn.connectTimeout = 3000
        conn.readTimeout = 5000
        try {
            check(conn.responseCode == 200) { "frame server answered ${conn.responseCode}" }
            val age = conn.getHeaderField("X-Frame-Age-Ms")?.toLongOrNull() ?: -1
            val bytes = conn.inputStream.use { it.readBytes() }
            return Frame(CvPattern.decodeColor(bytes, tag = "frame"), age)
        } finally {
            conn.disconnect()
        }
    }
}
