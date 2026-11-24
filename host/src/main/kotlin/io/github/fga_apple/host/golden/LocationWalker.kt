package io.github.fga_apple.host.golden

import io.github.lib_automata.Location
import io.github.lib_automata.Region
import java.lang.reflect.InvocationTargetException
import java.util.Collections
import java.util.IdentityHashMap
import kotlin.reflect.KClass
import kotlin.reflect.KParameter
import kotlin.reflect.KType
import kotlin.reflect.KVisibility
import kotlin.reflect.full.extensionReceiverParameter
import kotlin.reflect.full.instanceParameter
import kotlin.reflect.full.memberFunctions
import kotlin.reflect.full.memberProperties
import kotlin.reflect.jvm.isAccessible
import kotlin.reflect.jvm.javaField

sealed interface Leaf
data class LocationLeaf(val value: Location) : Leaf
data class RegionLeaf(val value: Region) : Leaf
data class IntLeaf(val value: Int) : Leaf
data object NullLeaf : Leaf
data class ErrorLeaf(val exceptionClass: String) : Leaf

class WalkResult(
    val leaves: Map<String, Leaf>,
    val notEnumerated: List<String>,
    val omittedArguments: List<String>
)

fun defaultIntRange(parameterName: String?): IntRange? = when (parameterName) {
    "slot", "index" -> 0..4
    "column" -> 0..6
    "columns" -> 5..7
    else -> null
}

class LocationWalker(
    private val owns: (Class<*>) -> Boolean,
    private val inheritedFrom: KClass<*>? = null,
    private val intRange: (String?) -> IntRange? = ::defaultIntRange
) {
    private val leaves = sortedMapOf<String, Leaf>()
    private val notEnumerated = sortedSetOf<String>()
    private val omitted = sortedSetOf<String>()
    private val visited = Collections.newSetFromMap(IdentityHashMap<Any, Boolean>())

    private val inheritedNames: Set<String> =
        inheritedFrom?.memberProperties?.map { it.name }?.toSet().orEmpty()

    fun visit(holder: Any, path: String = "", root: Boolean = true) {
        if (!visited.add(holder)) return
        val klass = holder::class
        val skip = if (root) emptySet() else inheritedNames

        for (property in klass.memberProperties.sortedBy { it.name }) {
            if (property.visibility != KVisibility.PUBLIC || property.name in skip) continue
            val childPath = join(path, property.name)
            val value = try {
                property.isAccessible = true
                if (property.isConst) property.javaField!!.get(null) else property.getter.call(holder)
            } catch (t: Throwable) {
                leaves[childPath] = ErrorLeaf(exceptionName(t))
                continue
            }
            record(childPath, value, property.returnType.isMarkedNullable && relevant(property.returnType))
        }

        for (function in klass.memberFunctions.sortedBy { it.name }) {
            if (function.visibility != KVisibility.PUBLIC) continue
            if (function.extensionReceiverParameter != null || function.instanceParameter == null) continue
            if (function.name in objectFunctions || function.name.startsWith("component")) continue
            if (!relevant(function.returnType)) continue

            val parameters = function.parameters.filter { it.kind == KParameter.Kind.VALUE }
            val candidates = parameters.map { candidates(it) }
            val signature = "${join(path, function.name)}(" +
                parameters.joinToString(", ") { "${it.name}: ${typeText(it.type)}" } + ")"
            if (candidates.any { it == null }) {
                notEnumerated.add(signature)
                continue
            }

            for (combination in product(candidates.map { it!! })) {
                val label = "${join(path, function.name)}(" + combination.joinToString(", ") { it.label } + ")"
                val result = try {
                    function.isAccessible = true
                    function.call(holder, *combination.map { it.value }.toTypedArray())
                } catch (t: Throwable) {
                    if (combination.any { it.ranged }) omitted.add(label)
                    else leaves[label] = ErrorLeaf(exceptionName(t))
                    continue
                }
                record(label, result, function.returnType.isMarkedNullable)
            }
        }
    }

    fun visitedTypes(): Set<Class<*>> = visited.map { it.javaClass }.toSet()

    fun result() =WalkResult(leaves.toMap(), notEnumerated.toList(), omitted.toList())

    private fun record(path: String, value: Any?, nullRelevant: Boolean) {
        when (value) {
            null -> if (nullRelevant) leaves[path] = NullLeaf
            is Location -> leaves[path] = LocationLeaf(value)
            is Region -> leaves[path] = RegionLeaf(value)
            is Int -> leaves[path] = IntLeaf(value)
            is Map<*, *> -> value.entries.sortedBy { it.key.toString() }
                .forEach { record("$path[${it.key}]", it.value, true) }
            is Iterable<*> -> value.forEachIndexed { index, item -> record("$path[$index]", item, true) }
            else -> if (owns(value.javaClass)) visit(value, path, false)
        }
    }

    private class Argument(val value: Any?, val label: String, val ranged: Boolean = false)

    private fun candidates(parameter: KParameter): List<Argument>? {
        val klass = parameter.type.classifier as? KClass<*> ?: return null
        return when (klass) {
            Int::class -> intRange(parameter.name)?.map { Argument(it, it.toString(), true) }
            Boolean::class -> listOf(false, true).map { Argument(it, it.toString()) }
            else -> values(klass)?.map { Argument(it, label(it)) }
        }
    }

    private fun values(klass: KClass<*>): List<Any>? {
        klass.objectInstance?.let { return listOf(it) }
        if (klass.java.isEnum) return klass.java.enumConstants.toList()
        if (klass.isSealed) {
            val all = mutableListOf<Any>()
            for (sub in klass.sealedSubclasses) all.addAll(values(sub) ?: return null)
            return all.distinct()
        }
        return null
    }

    private fun label(value: Any): String = when (value) {
        is Enum<*> -> "${shortName(value.declaringJavaClass)}.${value.name}"
        else -> shortName(value.javaClass)
    }

    private fun shortName(type: Class<*>): String =
        (type.canonicalName ?: type.name).split('.').dropWhile { it.first().isLowerCase() }.joinToString(".")

    private fun typeText(type: KType): String {
        val klass = type.classifier as? KClass<*>
        val name = klass?.qualifiedName?.split('.')?.dropWhile { it.first().isLowerCase() }?.joinToString(".")
        return (name ?: type.toString()) + if (type.isMarkedNullable) "?" else ""
    }

    private fun relevant(type: KType): Boolean {
        val klass = type.classifier as? KClass<*>
        if (klass == Location::class || klass == Region::class || klass == Int::class) return true
        return type.arguments.any { argument -> argument.type?.let { relevant(it) } == true }
    }

    private fun product(lists: List<List<Argument>>): List<List<Argument>> =
        lists.fold(listOf(emptyList())) { acc, list -> acc.flatMap { prefix -> list.map { prefix + it } } }

    private fun join(path: String, name: String) = if (path.isEmpty()) name else "$path.$name"

    private companion object {
        val objectFunctions = setOf("equals", "hashCode", "toString", "copy")

        fun exceptionName(t: Throwable): String {
            var current: Throwable = t
            while (current is InvocationTargetException && current.cause != null) current = current.cause!!
            return current::class.java.name
        }
    }
}
