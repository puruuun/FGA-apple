package io.github.fga_apple.host.di

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import java.io.File

/** 기기와 연결에 관한 설정. 게임 설정(IPreferences)과는 별개다. */
data class HostConfig(
    /** Mac의 `mac-capture/frame_server.py` 주소. */
    val frameUrl: String = "http://localhost:8766",
    /** ESP32가 붙은 시리얼 포트. */
    val serialPort: String = "COM3",
    /** iPhone 화면 크기(픽셀, 가로 기준). 받은 프레임과 다르면 실행을 멈춘다. */
    val screenWidth: Int = 2622,
    val screenHeight: Int = 1206,
    /** 누를 수 없는 세로 띠(펌웨어 좌표 0..32767). README의 실측값. 0,0이면 끈다. */
    val avoidLo: Int = 15040,
    val avoidHi: Int = 16480,
    /** FGA-Modified의 `app/src/main/assets` 폴더. */
    val assetsDir: File = File("fga/app/src/main/assets"),
    /** 서포트 이미지 등 사용자가 넣는 파일을 두는 폴더. */
    val dataDir: File = File("data"),
    /** true면 터치를 보내지 않고 무엇을 보냈을지만 출력한다. */
    val dryRun: Boolean = false
) {
    companion object {
        /**
         * 사용자 환경에 따라 달라지는 값을 `data/host.json`에서 읽는다. 이 파일은 Git에 넣지 않는다.
         * 없는 값은 기본값을 쓴다. 예시는 `configs/host.example.json`.
         */
        fun load(file: File = File("data/host.json")): HostConfig {
            val base = HostConfig()
            if (!file.isFile) return base
            val o: JsonObject = Json.parseToJsonElement(file.readText()).jsonObject
            fun str(key: String, default: String) = o[key]?.jsonPrimitive?.content ?: default
            fun int(key: String, default: Int) = o[key]?.jsonPrimitive?.intOrNull ?: default
            return base.copy(
                frameUrl = str("frameUrl", base.frameUrl),
                serialPort = str("serialPort", base.serialPort),
                screenWidth = int("screenWidth", base.screenWidth),
                screenHeight = int("screenHeight", base.screenHeight),
                avoidLo = int("avoidLo", base.avoidLo),
                avoidHi = int("avoidHi", base.avoidHi)
            )
        }
    }
}
