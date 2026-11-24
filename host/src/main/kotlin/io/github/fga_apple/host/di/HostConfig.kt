package io.github.fga_apple.host.di

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import java.io.File

data class HostConfig(
    val frameUrl: String = "http://localhost:8766",
    val serialPort: String = "COM3",
    val screenWidth: Int = 2622,
    val screenHeight: Int = 1206,
    val avoidLo: Int = 15040,
    val avoidHi: Int = 16480,
    val assetsDir: File = File("fga/app/src/main/assets"),
    val dataDir: File = File("data"),
    val dryRun: Boolean = false
) {
    companion object {
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
