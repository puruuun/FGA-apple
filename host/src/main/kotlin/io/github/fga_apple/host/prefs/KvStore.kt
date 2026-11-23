package io.github.fga_apple.host.prefs

import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlin.properties.ReadWriteProperty
import kotlin.reflect.KProperty

/**
 * Android의 SharedPreferences 자리를 대신하는 키-값 저장소.
 *
 * FGA-Modified의 `prefs` 모듈과 **같은 키 이름**을 쓴다. 그래서 Android 앱에서 내보낸 전투 설정
 * (SharedPreferences 전체를 JSON으로 쓴 것)을 그대로 읽을 수 있다. Android는 숫자를 문자열로
 * 저장하는 키와 숫자로 저장하는 키가 섞여 있으므로, 읽을 때는 양쪽을 모두 받아들인다.
 * 실행 중 바뀐 값은 메모리에만 남고 파일로 다시 쓰지 않는다.
 */
class KvStore(initial: JsonObject = JsonObject(emptyMap())) {
    private val values: MutableMap<String, JsonElement> = initial.toMutableMap()

    private fun primitive(key: String): JsonPrimitive? =
        (values[key] as? JsonPrimitive)?.takeIf { it !is JsonNull }

    fun getString(key: String, default: String): String = primitive(key)?.content ?: default

    fun getBool(key: String, default: Boolean): Boolean =
        primitive(key)?.content?.lowercase()?.toBooleanStrictOrNull() ?: default

    fun getInt(key: String, default: Int): Int =
        primitive(key)?.content?.trim()?.let { it.toIntOrNull() ?: it.toDoubleOrNull()?.toInt() } ?: default

    fun getStringSet(key: String): Set<String> =
        (values[key] as? JsonArray)?.mapNotNull { (it as? JsonPrimitive)?.content }?.toSet() ?: emptySet()

    fun put(key: String, value: String) { values[key] = JsonPrimitive(value) }
    fun put(key: String, value: Boolean) { values[key] = JsonPrimitive(value) }
    fun put(key: String, value: Int) { values[key] = JsonPrimitive(value) }
    fun putStringSet(key: String, value: Set<String>) { values[key] = JsonArray(value.map { JsonPrimitive(it) }) }

    fun putAll(map: Map<String, *>) {
        for ((k, v) in map) {
            when (v) {
                is String -> put(k, v)
                is Boolean -> put(k, v)
                is Int -> put(k, v)
                is Number -> put(k, v.toInt())
                is Collection<*> -> putStringSet(k, v.filterIsInstance<String>().toSet())
            }
        }
    }

    fun export(): Map<String, Any?> = values.mapValues { (_, v) ->
        when (v) {
            is JsonArray -> v.mapNotNull { (it as? JsonPrimitive)?.content }.toSet()
            is JsonPrimitive -> v.content.toBooleanStrictOrNull() ?: v.content.toIntOrNull() ?: v.content
            else -> null
        }
    }

    // 위임 속성. `var x by kv.bool("key")`처럼 쓴다.
    fun string(key: String, default: String = "") = prop({ getString(key, default) }, { put(key, it) })
    fun bool(key: String, default: Boolean = false) = prop({ getBool(key, default) }, { put(key, it) })
    fun int(key: String, default: Int = 0) = prop({ getInt(key, default) }, { put(key, it) })

    inline fun <reified E : Enum<E>> enum(key: String, default: E) = prop(
        { runCatching { enumValueOf<E>(getString(key, default.name)) }.getOrDefault(default) },
        { put(key, it.name) }
    )

    /** 문자열 하나로 저장되는 값. 저장된 것이 없으면 [default], 해석에 실패하면 [parse]가 정한 대로. */
    fun <T> serialized(key: String, default: T, parse: (String) -> T, format: (T) -> String) = prop(
        { if (values[key] is JsonPrimitive && values[key] !is JsonNull) parse(getString(key, "")) else default },
        { put(key, format(it)) }
    )

    fun <T> prop(get: () -> T, set: (T) -> Unit) = object : ReadWriteProperty<Any?, T> {
        override fun getValue(thisRef: Any?, property: KProperty<*>): T = get()
        override fun setValue(thisRef: Any?, property: KProperty<*>, value: T) = set(value)
    }
}
