package io.github.fga_apple.host.golden

import io.github.fate_grand_automata.scripts.prefs.IBattleConfig
import io.github.fga_apple.host.prefs.HostPreferences
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonObject
import java.lang.reflect.InvocationTargetException
import kotlin.reflect.KClass
import kotlin.reflect.KVisibility
import kotlin.reflect.full.memberProperties
import kotlin.reflect.jvm.isAccessible

object GoldenPrefs {
    private const val PREFS_PACKAGE = "io.github.fate_grand_automata.scripts.prefs"
    private const val MAX_DEPTH = 8

    fun export(fgaText: String): Map<String, Any?> {
        val root = Json.parseToJsonElement(fgaText).jsonObject
        val prefs = HostPreferences(JsonObject(mapOf("battleConfig" to root)))
        return dump(prefs.selectedBattleConfig, IBattleConfig::class)
    }

    private fun dump(target: Any, type: KClass<*>): Map<String, Any?> {
        val out = sortedMapOf<String, Any?>()
        for (property in type.memberProperties.filter { it.visibility == KVisibility.PUBLIC }) {
            val value = try {
                property.isAccessible = true
                property.getter.call(target)
            } catch (t: Throwable) {
                out[property.name] = mapOf("error" to exceptionName(t))
                continue
            }
            val declared = property.returnType.classifier as? KClass<*>
            out[property.name] =
                if (value != null && declared != null && declared.java.isInterface && declared.java.packageName == PREFS_PACKAGE)
                    dump(value, declared)
                else convert(value, 0)
        }
        return out
    }

    private fun convert(value: Any?, depth: Int): Any? = when {
        value == null -> null
        value is String || value is Boolean || value is Int || value is Long -> value
        value is Double -> value
        value is Enum<*> -> value.name
        depth > MAX_DEPTH -> value.toString()
        value is Set<*> -> value.map { convert(it, depth + 1) }.sortedBy { GoldenJson.write(it) }
        value is Iterable<*> -> value.map { convert(it, depth + 1) }
        value is Map<*, *> -> value.entries.associate { it.key.toString() to convert(it.value, depth + 1) }
        else -> {
            val properties = value::class.memberProperties.filter { it.visibility == KVisibility.PUBLIC }
            if (properties.isEmpty()) value.toString()
            else properties.sortedBy { it.name }.associate { property ->
                property.name to try {
                    property.isAccessible = true
                    convert(property.getter.call(value), depth + 1)
                } catch (t: Throwable) {
                    mapOf("error" to exceptionName(t))
                }
            }
        }
    }

    private fun exceptionName(t: Throwable): String {
        var current: Throwable = t
        while (current is InvocationTargetException && current.cause != null) current = current.cause!!
        return current::class.java.name
    }
}
