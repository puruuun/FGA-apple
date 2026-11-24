package io.github.fga_apple.host.golden

import io.github.fga_apple.host.imaging.CvPattern
import io.github.fga_apple.host.imaging.OpenCv
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.boolean
import kotlinx.serialization.json.double
import kotlinx.serialization.json.int
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import org.opencv.core.Core
import org.opencv.core.Mat
import org.opencv.core.MatOfByte
import org.opencv.imgcodecs.Imgcodecs
import java.io.File
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class GoldenTemplatesTest {
    private companion object {
        val root: File = generateSequence(File("").absoluteFile) { it.parentFile }
            .first { File(it, "core/testdata/golden/scores").isDirectory && File(it, "fga/app/src/main/assets/En").isDirectory }
        val templatesDir = File(root, "core/testdata/synthetic/templates")
        val framesDir = File(root, "core/testdata/synthetic/frames")

        val loaded: Templates.Loaded by lazy { Templates.load(File(root, "fga/app/src/main/assets")) }

        val scoreFiles: List<File> by lazy {
            File(root, "core/testdata/golden/scores").listFiles { f -> f.extension == "json" }!!.sortedBy { it.name }
        }

        fun cases(file: File): List<JsonObject> =
            Json.parseToJsonElement(file.readText()).jsonObject.getValue("cases").jsonArray.map { it.jsonObject }

        val named: Map<String, Boolean> by lazy {
            val out = sortedMapOf<String, Boolean>()
            scoreFiles.flatMap { cases(it) }.forEach {
                out[it.getValue("template").jsonPrimitive.content] = it.getValue("masked").jsonPrimitive.boolean
            }
            out
        }

        fun decode(file: File): Mat {
            OpenCv.load()
            return Imgcodecs.imdecode(MatOfByte(*file.readBytes()), Imgcodecs.IMREAD_UNCHANGED)
        }

        fun sameMat(a: Mat, b: Mat): Boolean {
            if (a.rows() != b.rows() || a.cols() != b.cols() || a.type() != b.type()) return false
            val diff = Mat()
            Core.absdiff(a, b, diff)
            return Core.countNonZero(diff) == 0
        }

        fun fromFiles(name: String, masked: Boolean): Template {
            val gray = decode(File(templatesDir, name))
            val mask = if (masked) decode(File(templatesDir, GoldenScores.maskPath(name))) else null
            return Template(name, masked, gray, mask, CvPattern(gray, ownsMat = false, tag = name, mask = mask))
        }
    }

    @Test
    fun everyNamedTemplateHasItsFileAndEveryMaskedOneItsMaskAndNothingElseExists() {
        assertTrue(named.isNotEmpty())
        val expected = named.flatMap { (name, masked) ->
            if (masked) listOf(name, name.removeSuffix(".png") + ".mask.png") else listOf(name)
        }.sorted()
        val actual = templatesDir.walkTopDown().filter { it.isFile }
            .map { it.relativeTo(templatesDir).invariantSeparatorsPath }.sorted().toList()
        assertEquals(expected, actual)
    }

    @Test
    fun writtenTemplatesAndMasksDecodeToExactlyTheMatsTheScoresWereComputedWith() {
        val byName = loaded.usable.associateBy { it.name }
        named.forEach { (name, masked) ->
            val template = byName.getValue(name)
            assertEquals(masked, template.masked, name)
            assertTrue(sameMat(template.gray, decode(File(templatesDir, name))), name)
            if (masked) {
                assertTrue(sameMat(template.mask!!, decode(File(templatesDir, GoldenScores.maskPath(name)))), "$name mask")
            }
        }
    }

    @Test
    fun writtenFilesAreTheBytesTheGeneratorEncodes() {
        val byName = loaded.usable.associateBy { it.name }
        val out = File(templatesDir.parentFile.parentFile.parentFile.parentFile, "build/golden-templates-test")
        try {
            GoldenScores.writeTemplates(out, named.keys.map { byName.getValue(it) })
            named.forEach { (name, masked) ->
                assertContentEquals(File(templatesDir, name).readBytes(), File(out, name).readBytes(), name)
                if (masked) {
                    val mask = GoldenScores.maskPath(name)
                    assertContentEquals(File(templatesDir, mask).readBytes(), File(out, mask).readBytes(), mask)
                }
            }
        } finally {
            out.deleteRecursively()
        }
    }

    @Test
    fun scoringWithTheWrittenFilesReproducesTheCommittedBestOfEveryAsIsCase() {
        var checked = 0
        var maskedChecked = 0
        scoreFiles.forEach { file ->
            val json = Json.parseToJsonElement(file.readText()).jsonObject
            val name = json.getValue("frame").jsonPrimitive.content
            val frame = decode(File(framesDir, "$name.png"))
            val pattern = CvPattern(frame, ownsMat = false, tag = name)
            cases(file).filter { it.getValue("variant").jsonPrimitive.content == "as_is" }.forEach { case ->
                val template = fromFiles(case.getValue("template").jsonPrimitive.content, case.getValue("masked").jsonPrimitive.boolean)
                val r = case.getValue("region").jsonObject
                val region = Box(r.getValue("x").jsonPrimitive.int, r.getValue("y").jsonPrimitive.int, r.getValue("width").jsonPrimitive.int, r.getValue("height").jsonPrimitive.int)
                val (best, _) = GoldenScores.hits(pattern, template, region)
                val expected = case.getValue("best").jsonObject
                val label = "$name ${template.name} ${case.getValue("kind").jsonPrimitive.content}"
                assertEquals(expected.getValue("x").jsonPrimitive.int, best.x, label)
                assertEquals(expected.getValue("y").jsonPrimitive.int, best.y, label)
                assertEquals(expected.getValue("score").jsonPrimitive.double, best.score, 1e-9, label)
                checked++
                if (template.masked) maskedChecked++
            }
        }
        assertTrue(checked >= 40, "checked $checked")
        assertTrue(maskedChecked > 0)
    }
}
