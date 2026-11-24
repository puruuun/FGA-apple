package io.github.fga_apple.host.capture

import io.github.fga_apple.host.imaging.CvPattern
import java.net.HttpURLConnection
import java.net.URI

class Frame(val image: CvPattern, val ageMs: Long)

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
