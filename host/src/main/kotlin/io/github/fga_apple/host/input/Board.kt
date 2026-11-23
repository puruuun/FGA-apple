package io.github.fga_apple.host.input

import com.fazecast.jSerialComm.SerialPort

/** ESP32 펌웨어(`firmware/`)와 USB 시리얼로 명령을 주고받는다. */
class Board(portName: String) : AutoCloseable {
    private val port: SerialPort = SerialPort.getCommPort(portName).apply {
        setComPortParameters(115200, 8, SerialPort.ONE_STOP_BIT, SerialPort.NO_PARITY)
        setComPortTimeouts(SerialPort.TIMEOUT_READ_SEMI_BLOCKING, 200, 0)
        // ESP32-C3의 USB 단자는 RTS를 리셋으로 받는다. 열기 전에 꺼 두지 않으면 보드가 재시작된다.
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

    /** 명령 한 줄을 보내고 `OK…` 또는 `ERR…` 응답 줄을 돌려준다. */
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
                // '#'로 시작하는 줄은 보드의 상태 알림이므로 건너뛴다.
                if (text.startsWith("OK") || text.startsWith("ERR")) return text
            } else sb.append(c)
        }
        return "ERR timeout"
    }

    /** 실패 응답이면 예외를 던진다. */
    fun require(line: String, timeoutMs: Long = 5000): String =
        command(line, timeoutMs).also { check(it.startsWith("OK")) { "보드 명령 실패: $line -> $it" } }

    override fun close() {
        port.closePort()
    }
}
