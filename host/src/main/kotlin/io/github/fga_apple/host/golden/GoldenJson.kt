package io.github.fga_apple.host.golden

object GoldenJson {
    fun write(value: Any?): String = StringBuilder().also { append(it, value, 0) }.append('\n').toString()

    private fun scalar(value: Any?) = value == null || value is String || value is Number || value is Boolean

    private fun inline(value: Any?): Boolean = when (value) {
        is Map<*, *> -> value.values.all { scalar(it) }
        is List<*> -> value.all { scalar(it) }
        else -> false
    }

    private fun append(out: StringBuilder, value: Any?, depth: Int) {
        when (value) {
            null -> out.append("null")
            is String -> quote(out, value)
            is Boolean -> out.append(value)
            is Int, is Long -> out.append(value.toString())
            is Double -> {
                require(value.isFinite()) { "not finite: $value" }
                out.append(value.toString())
            }
            is Float -> append(out, value.toDouble(), depth)
            is Map<*, *> -> {
                val entries = value.entries.sortedBy { it.key.toString() }
                if (entries.isEmpty()) {
                    out.append("{}")
                } else if (inline(value)) {
                    out.append("{")
                    entries.forEachIndexed { i, e ->
                        if (i > 0) out.append(", ")
                        quote(out, e.key.toString())
                        out.append(": ")
                        append(out, e.value, depth)
                    }
                    out.append("}")
                } else {
                    out.append("{\n")
                    entries.forEachIndexed { i, e ->
                        indent(out, depth + 1)
                        quote(out, e.key.toString())
                        out.append(": ")
                        append(out, e.value, depth + 1)
                        out.append(if (i < entries.size - 1) ",\n" else "\n")
                    }
                    indent(out, depth)
                    out.append("}")
                }
            }
            is List<*> -> {
                if (value.isEmpty()) {
                    out.append("[]")
                } else if (inline(value)) {
                    out.append("[")
                    value.forEachIndexed { i, e ->
                        if (i > 0) out.append(", ")
                        append(out, e, depth)
                    }
                    out.append("]")
                } else {
                    out.append("[\n")
                    value.forEachIndexed { i, e ->
                        indent(out, depth + 1)
                        append(out, e, depth + 1)
                        out.append(if (i < value.size - 1) ",\n" else "\n")
                    }
                    indent(out, depth)
                    out.append("]")
                }
            }
            else -> error("unsupported: ${value::class}")
        }
    }

    private fun indent(out: StringBuilder, depth: Int) {
        repeat(depth) { out.append("  ") }
    }

    private fun quote(out: StringBuilder, text: String) {
        out.append('"')
        for (c in text) {
            when {
                c == '"' -> out.append("\\\"")
                c == '\\' -> out.append("\\\\")
                c == '\n' -> out.append("\\n")
                c == '\r' -> out.append("\\r")
                c == '\t' -> out.append("\\t")
                c < ' ' -> out.append("\\u%04x".format(c.code))
                else -> out.append(c)
            }
        }
        out.append('"')
    }
}
