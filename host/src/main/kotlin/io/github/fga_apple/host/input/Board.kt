package io.github.fga_apple.host.input

import com.fazecast.jSerialComm.SerialPort

class Board(portName: String) : AutoCloseable {
    private val port: SerialPort = SerialPort.getCommPort(portName).apply {
        setComPortParameters(115200, 8, SerialPort.ONE_STOP_BIT, SerialPort.NO_PARITY)
        setComPortTimeouts(SerialPort.TIMEOUT_READ_SEMI_BLOCKING, 200, 0)
        // The ESP32-C3 USB port treats RTS as reset, so RTS must be off before the port opens.
        clearRTS()
        setDTR()
        check(openPort()) { "시리얼 포트 $portName 을 열 수 없습니다" }
    }

    init {
        Thread.sleep(300)
        drain()
    }

    private fun drain() {
        val buf = ByteArray(256)
        while (port.bytesAvailable() > 0) port.readBytes(buf, buf.size)
    }

    @Synchronized
    fun command(line: String, timeoutMs: Long = 5000): String {
        val bytes = (line + "\n").toByteArray(Charsets.US_ASCII)
        port.writeBytes(bytes, bytes.size)

        val deadline = System.currentTimeMillis() + timeoutMs
        val sb = StringBuilder()
        val one = ByteArray(1)
        while (System.currentTimeMillis() < deadline) {
            if (port.readBytes(one, 1) != 1) continue
            val c = one[0].toInt().toChar()
            if (c == '\n' || c == '\r') {
                val text = sb.toString().trim()
                sb.setLength(0)
                if (text.startsWith("OK") || text.startsWith("ERR")) return text
            } else sb.append(c)
        }
        return "ERR timeout"
    }

    fun require(line: String, timeoutMs: Long = 5000): String =
        command(line, timeoutMs).also { check(it.startsWith("OK")) { "보드 명령 실패: $line -> $it" } }

    override fun close() {
        port.closePort()
    }
}
