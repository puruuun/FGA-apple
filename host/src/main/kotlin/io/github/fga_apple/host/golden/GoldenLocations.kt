package io.github.fga_apple.host.golden

import io.github.fate_grand_automata.scripts.Images
import io.github.fate_grand_automata.scripts.ScriptLog
import io.github.fate_grand_automata.scripts.enums.GameServer
import io.github.fate_grand_automata.scripts.locations.IScriptAreaTransforms
import io.github.fate_grand_automata.scripts.locations.Locations
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

    val masterOffsetPaths = listOf(
        "battle.master.locate(Skill.Master.A)",
        "battle.master.locate(Skill.Master.B)",
        "battle.master.locate(Skill.Master.C)",
        "battle.master.masterSkillOpenClick",
        "battle.master.stageCountRegion"
    )

    val battleMenu = Images.BattleMenu

    fun build(server: GameServer, width: Int, height: Int, scene: GoldenScene, assetsDir: File? = null): GoldenComponent {
        val prefs = HostPreferences.empty().also { it.gameServer = server }
        val defaults = HostConfig()
        return DaggerGoldenComponent.builder()
            .preferences(prefs)
            .config(
                HostConfig(
                    screenWidth = width,
                    screenHeight = height,
                    assetsDir = assetsDir ?: defaults.assetsDir,
                    dryRun = true
                )
            )
            .scene(scene)
            .build()
    }

    fun foundScene(blank: GoldenComponent): GoldenScene {
        val master = blank.api().locations.battle.master
        val search = master.battleMenuSearchRegion
        val usualCenterX = master.masterSkillOpenClick.x
        val template = blank.images()[battleMenu]
        val size = blank.transformer().fromImage(Region(0, 0, template.width, template.height))
        return GoldenScene(
            Pasted(
                battleMenu,
                Location(usualCenterX - size.width / 2, search.y + (search.height - size.height) / 2)
            )
        )
    }

    private class Walked(val result: WalkResult, val unreachable: List<String>, val locations: Locations)

    private fun walk(component: GoldenComponent): Walked {
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
        return Walked(walker.result(), unreachable.toList(), locations)
    }

    private fun outcome(component: GoldenComponent, walked: Walked): Map<String, Any?> {
        val transformer = component.transformer()
        return mapOf(
            "defaultOffsetLogged" to component.messages().logged.contains(ScriptLog.DefaultMasterOffset),
            "values" to masterOffsetPaths.associateWith { path ->
                render(walked.result.leaves[path] ?: error("missing $path"), transformer)
            }
        )
    }

    fun export(server: GameServer, width: Int, height: Int, assetsDir: File? = null): Map<String, Any?> {
        val blank = build(server, width, height, GoldenScene.Blank, assetsDir)
        var found: GoldenComponent? = null
        try {
            val transformer = blank.transformer()
            val scale = blank.scale()
            val walked = walk(blank)
            val result = walked.result
            val locations = walked.locations

            val foundComponent = build(server, width, height, foundScene(blank), assetsDir).also { found = it }
            val outcomes = mapOf(
                "buttonNotFound" to outcome(blank, walked),
                "buttonFound" to outcome(foundComponent, walk(foundComponent))
            )

            return mapOf(
                "server" to serverName(server),
                "screen" to mapOf("width" to width, "height" to height),
                "isWide" to locations.isWide,
                "gameArea" to region(blank.gameArea().gameArea),
                "scriptToScreen" to scale.scriptToScreen,
                "screenToImage" to scale.screenToImage,
                "scriptToImage" to scale.scriptToImage,
                "values" to result.leaves.mapValues { (_, leaf) -> render(leaf, transformer) },
                "errors" to result.leaves.filterValues { it is ErrorLeaf }.keys.toList(),
                "masterOutcomes" to outcomes,
                "notEnumerated" to result.notEnumerated,
                "omittedInvalidArguments" to result.omittedArguments,
                "unreachableHolders" to walked.unreachable
            )
        } finally {
            for (component in listOfNotNull(blank, found)) {
                component.gestures().close()
                component.screenshots().close()
            }
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
