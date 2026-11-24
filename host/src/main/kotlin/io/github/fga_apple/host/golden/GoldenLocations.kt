package io.github.fga_apple.host.golden

import io.github.fate_grand_automata.scripts.enums.GameServer
import io.github.fate_grand_automata.scripts.locations.IScriptAreaTransforms
import io.github.fate_grand_automata.scripts.locations.Locations
import io.github.fga_apple.host.di.DaggerHostComponent
import io.github.fga_apple.host.di.HostConfig
import io.github.fga_apple.host.prefs.HostPreferences
import io.github.lib_automata.Location
import io.github.lib_automata.Region
import io.github.lib_automata.Transformer
import java.io.File
import java.util.jar.JarFile
import kotlin.reflect.KClass
import kotlin.reflect.full.isSubclassOf
import kotlin.reflect.jvm.isAccessible

object GoldenLocations {
    val screens = listOf(
        2622 to 1206, 2796 to 1290, 2556 to 1179, 2340 to 1080, 2400 to 1080, 1920 to 1080,
        2560 to 1440, 3360 to 1440, 3840 to 1600, 2048 to 1536, 2732 to 2048
    )

    fun serverName(server: GameServer) = server.serialize().replace(' ', '_')

    fun fileName(server: GameServer, width: Int, height: Int) = "${serverName(server)}_${width}x$height.json"

    fun export(server: GameServer, width: Int, height: Int): Map<String, Any?> {
        val prefs = HostPreferences.empty().also { it.gameServer = server }
        val component = DaggerHostComponent.builder()
            .preferences(prefs)
            .config(HostConfig(frameUrl = "http://127.0.0.1:1", screenWidth = width, screenHeight = height, dryRun = true))
            .build()
        try {
            val transformer = component.transformer()
            val scale = component.scale()
            val locations = component.api().locations

            val walker = LocationWalker(
                owns = { it.packageName == Locations::class.java.packageName },
                inheritedFrom = IScriptAreaTransforms::class
            )
            walker.visit(locations)
            val reached = walker.visitedTypes()

            val unreachable = sortedSetOf<String>()
            for (klass in packageClasses(Locations::class.java)) {
                if (klass in reached || klass == Locations::class.java) continue
                if (!klass.isAnnotationPresent(Metadata::class.java) || klass.isInterface || klass.isEnum || klass.name.endsWith("Kt") ||
                    java.lang.reflect.Modifier.isAbstract(klass.modifiers)
                ) continue
                val holder = instantiate(klass.kotlin, locations)
                if (holder == null) unreachable.add(klass.simpleName)
                else walker.visit(holder, "sibling.${klass.simpleName}", false)
            }

            val result = walker.result()
            return mapOf(
                "server" to serverName(server),
                "screen" to mapOf("width" to width, "height" to height),
                "isWide" to locations.isWide,
                "gameArea" to region(component.gameArea().gameArea),
                "scriptToScreen" to scale.scriptToScreen,
                "screenToImage" to scale.screenToImage,
                "scriptToImage" to scale.scriptToImage,
                "values" to result.leaves.mapValues { (_, leaf) -> render(leaf, transformer) },
                "errors" to result.leaves.filterValues { it is ErrorLeaf }.keys.toList(),
                "notEnumerated" to result.notEnumerated,
                "omittedInvalidArguments" to result.omittedArguments,
                "unreachableHolders" to unreachable.toList()
            )
        } finally {
            component.gestures().close()
        }
    }

    private fun loc(l: Location) = mapOf("x" to l.x, "y" to l.y)

    private fun region(r: Region) = mapOf("x" to r.x, "y" to r.y, "width" to r.width, "height" to r.height)

    private fun <T> attempt(into: MutableMap<String, Any?>, key: String, block: () -> T): T? =
        try {
            block()
        } catch (t: Throwable) {
            into["${key}Error"] = t::class.java.name
            null
        }

    private fun render(leaf: Leaf, t: Transformer): Any? = when (leaf) {
        is NullLeaf -> null
        is IntLeaf -> leaf.value
        is ErrorLeaf -> mapOf("error" to leaf.exceptionClass)
        is LocationLeaf -> mutableMapOf<String, Any?>("script" to loc(leaf.value)).also { out ->
            attempt(out, "screen") { t.toScreen(leaf.value) }?.let { out["screen"] = loc(it) }
        }
        is RegionLeaf -> mutableMapOf<String, Any?>("script" to region(leaf.value)).also { out ->
            attempt(out, "screen") { t.toScreen(leaf.value) }?.let { out["screen"] = region(it) }
            attempt(out, "image") { t.toImage(leaf.value) }?.let { out["image"] = region(it) }
        }
    }

    private fun instantiate(klass: KClass<*>, anchor: Any): Any? {
        klass.objectInstance?.let { return it }
        val constructor = klass.constructors.singleOrNull() ?: return null
        val parameters = constructor.parameters
        if (parameters.isEmpty()) return null
        val supplied = parameters.all { p -> (p.type.classifier as? KClass<*>)?.let { anchor::class.isSubclassOf(it) } == true }
        if (!supplied) return null
        return try {
            constructor.isAccessible = true
            constructor.call(*parameters.map { anchor }.toTypedArray())
        } catch (t: Throwable) {
            null
        }
    }

    private fun packageClasses(anchor: Class<*>): List<Class<*>> {
        val directory = anchor.packageName.replace('.', '/')
        val source = File(anchor.protectionDomain.codeSource.location.toURI())
        val names = if (source.isDirectory) {
            File(source, directory).listFiles { f -> f.name.endsWith(".class") }.orEmpty()
                .map { "${anchor.packageName}.${it.name.removeSuffix(".class")}" }
        } else {
            JarFile(source).use { jar ->
                jar.entries().asSequence().map { it.name }
                    .filter { it.startsWith("$directory/") && it.endsWith(".class") && it.indexOf('/', directory.length + 1) < 0 }
                    .map { it.removeSuffix(".class").replace('/', '.') }
                    .toList()
            }
        }
        return names.filter { '$' !in it }.sorted().map { Class.forName(it, false, anchor.classLoader) }
    }
}
