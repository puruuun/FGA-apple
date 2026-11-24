package io.github.fga_apple.host.golden

import io.github.fate_grand_automata.scripts.enums.GameServers
import java.io.File

fun main(args: Array<String>) {
    val root = File(args.firstOrNull() ?: "core/testdata")
    val goldenDir = File(root, "golden")
    val locationsDir = File(goldenDir, "locations").also { it.mkdirs() }
    val prefsDir = File(goldenDir, "prefs").also { it.mkdirs() }

    val written = mutableSetOf<File>()
    fun write(file: File, content: String) {
        file.writeText(content, Charsets.UTF_8)
        written += file
    }

    var errorCount = 0
    for (server in GameServers.values) {
        for ((width, height) in GoldenLocations.screens) {
            val file = File(locationsDir, GoldenLocations.fileName(server, width, height))
            try {
                val document = GoldenLocations.export(server, width, height)
                val values = document["values"] as Map<*, *>
                val errors = document["errors"] as List<*>
                errorCount += errors.size
                println("${file.name}: ${values.size} values, ${errors.size} errors, " +
                    "${(document["notEnumerated"] as List<*>).size} not enumerated, " +
                    "${(document["omittedInvalidArguments"] as List<*>).size} omitted")
                write(file, GoldenJson.write(document))
            } catch (t: Throwable) {
                println("${file.name}: FAILED ${t::class.java.name}: ${t.message}")
                errorCount++
            }
        }
    }

    val inputs = File(root, "fga").listFiles { f -> f.isFile && (f.name.endsWith(".fga") || f.name.endsWith(".fga.json")) }
        .orEmpty().sortedBy { it.name }
    for (input in inputs) {
        val name = input.name.removeSuffix(".json").removeSuffix(".fga")
        val file = File(prefsDir, "$name.json")
        write(file, GoldenJson.write(GoldenPrefs.export(input.readText(Charsets.UTF_8))))
        println("${file.name}: written")
    }

    for (directory in listOf(locationsDir, prefsDir)) {
        directory.listFiles { f -> f.isFile && f.name.endsWith(".json") && f !in written }
            .orEmpty().forEach { it.delete() }
    }
    println("errors: $errorCount")
}
