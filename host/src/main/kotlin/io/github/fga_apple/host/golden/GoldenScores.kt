package io.github.fga_apple.host.golden

import io.github.fga_apple.host.imaging.CvPattern
import io.github.fga_apple.host.imaging.OpenCv
import io.github.lib_automata.Region
import org.opencv.core.Mat
import org.opencv.core.Size
import org.opencv.imgcodecs.Imgcodecs
import org.opencv.imgproc.Imgproc
import java.io.File
import kotlin.system.exitProcess

object GoldenScores {
    const val SIMILARITY = 0.8
    const val FURTHER_MATCHES = 3
    const val METHOD = "TM_CCOEFF_NORMED"
    private const val NO_THRESHOLD = -2.0

    class Hit(val x: Int, val y: Int, val score: Double)

    fun hits(frame: CvPattern, template: Template, region: Box): Pair<Hit, List<Hit>> {
        require(region.width >= template.width && region.height >= template.height) {
            "region smaller than ${template.name}"
        }
        val cropped = frame.crop(Region(region.x, region.y, region.width, region.height)) as CvPattern
        val best = cropped.findMatches(template.pattern, NO_THRESHOLD).first().toHit(region)
        val further = cropped.findMatches(template.pattern, SIMILARITY)
            .take(FURTHER_MATCHES + 1).toList().drop(1).map { it.toHit(region) }
        check(best.score.isFinite() && further.all { it.score.isFinite() }) { "non-finite score for ${template.name}" }
        return best to further
    }

    private fun io.github.lib_automata.Match.toHit(region: Box) =
        Hit(region.x + this.region.x, region.y + this.region.y, score)

    fun frameJson(name: String, source: String, seed: Long?, frame: Mat, cases: List<CaseSpec>): String {
        val pattern = CvPattern(frame, ownsMat = false, tag = name)
        val items = cases.map { spec ->
            val (best, further) = hits(pattern, spec.template, spec.region)
            caseJson(spec, best, further)
        }
        val out = StringBuilder()
        out.append("{\n")
        out.append("  \"cases\": [\n")
        out.append(items.joinToString(",\n"))
        out.append("\n  ],\n")
        out.append("  \"frame\": ${quote(name)},\n")
        out.append("  \"height\": ${frame.rows()},\n")
        out.append("  \"method\": ${quote(METHOD)},\n")
        out.append("  \"seed\": ${seed ?: "null"},\n")
        out.append("  \"similarity\": $SIMILARITY,\n")
        out.append("  \"source\": ${quote(source)},\n")
        out.append("  \"width\": ${frame.cols()}\n")
        out.append("}\n")
        return out.toString()
    }

    private fun caseJson(spec: CaseSpec, best: Hit, further: List<Hit>): String {
        val params = spec.params.toSortedMap().entries.joinToString(", ") { "${quote(it.key)}: ${it.value}" }
        val pasted = spec.pasted?.let { boxJson(it) } ?: "null"
        val furtherJson = if (further.isEmpty()) "[]" else further.joinToString(",\n", "[\n", "\n      ]") {
            "        ${hitJson(it)}"
        }
        return buildString {
            append("    {\n")
            append("      \"best\": ${hitJson(best)},\n")
            append("      \"further\": $furtherJson,\n")
            append("      \"kind\": ${quote(spec.kind)},\n")
            append("      \"masked\": ${spec.template.masked},\n")
            append("      \"params\": {$params},\n")
            append("      \"pasted\": $pasted,\n")
            append("      \"region\": ${boxJson(spec.region)},\n")
            append("      \"template\": ${quote(spec.template.name)},\n")
            append("      \"variant\": ${quote(spec.variant.id)}\n")
            append("    }")
        }
    }

    private fun hitJson(hit: Hit) = "{\"score\": ${hit.score}, \"x\": ${hit.x}, \"y\": ${hit.y}}"

    private fun boxJson(box: Box) =
        "{\"height\": ${box.height}, \"width\": ${box.width}, \"x\": ${box.x}, \"y\": ${box.y}}"

    private fun quote(text: String) = "\"" + text.replace("\\", "\\\\").replace("\"", "\\\"") + "\""

    fun writeSynthetic(root: File, assetsDir: File) {
        val loaded = Templates.load(assetsDir)
        loaded.skipped.forEach { println("skipped $it") }
        val frames = File(root, "core/testdata/synthetic/frames").apply { mkdirs() }
        val scores = File(root, "core/testdata/golden/scores").apply { mkdirs() }
        for (index in 0 until FrameBuilder.FRAME_COUNT) {
            val frame = FrameBuilder.build(index, loaded.usable)
            File(frames, "${frame.name}.png").writeBytes(FrameBuilder.encodePng(frame.mat))
            val json = frameJson(frame.name, "synthetic", frame.seed, frame.mat, frame.cases)
            File(scores, "${frame.name}.json").writeBytes(json.toByteArray(Charsets.UTF_8))
            println("${frame.name}: ${frame.cases.size} cases")
        }
    }

    fun writeRecorded(framesDir: File, root: File, assetsDir: File) {
        val loaded = Templates.load(assetsDir)
        loaded.skipped.forEach { println("skipped $it") }
        val out = File(root, "data/golden/scores").apply { mkdirs() }
        val files = framesDir.listFiles { f -> f.isFile && f.extension.lowercase() in setOf("png", "jpg", "jpeg") }
            ?.sortedBy { it.name }.orEmpty()
        check(files.isNotEmpty()) { "no png or jpg frames in ${framesDir.path}" }
        for (file in files) {
            val gray = Imgcodecs.imread(file.path, Imgcodecs.IMREAD_GRAYSCALE)
            check(!gray.empty()) { "cannot read ${file.path}" }
            val frame = if (gray.rows() == FrameBuilder.HEIGHT) gray else Mat().also {
                val width = Math.round(gray.cols() * FrameBuilder.HEIGHT.toDouble() / gray.rows()).toInt()
                Imgproc.resize(gray, it, Size(width.toDouble(), FrameBuilder.HEIGHT.toDouble()), 0.0, 0.0, Imgproc.INTER_AREA)
            }
            val whole = Box(0, 0, frame.cols(), frame.rows())
            val cases = loaded.usable
                .filter { it.width <= frame.cols() && it.height <= frame.rows() }
                .map { CaseSpec(it, Variant.Recorded, "whole_frame", null, whole, emptyMap()) }
            val stem = file.nameWithoutExtension
            File(out, "$stem.json").writeBytes(frameJson(stem, "recorded", null, frame, cases).toByteArray(Charsets.UTF_8))
            println("$stem: ${cases.size} cases")
        }
    }
}

fun main(args: Array<String>) {
    OpenCv.load()
    val root = File(".")
    val assets = File(root, "fga/app/src/main/assets")
    val index = args.indexOf("--frames")
    if (index >= 0) {
        val dir = args.getOrNull(index + 1) ?: run {
            System.err.println("--frames needs a directory")
            exitProcess(2)
        }
        GoldenScores.writeRecorded(File(dir), root, assets)
    } else {
        GoldenScores.writeSynthetic(root, assets)
    }
}
