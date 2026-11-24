package io.github.fga_apple.host.golden

import org.opencv.core.Core
import org.opencv.core.Mat
import org.opencv.core.Rect
import java.io.File
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class FrameBuilderTest {
    private companion object {
        val loaded: Templates.Loaded by lazy { Templates.load(File("fga/app/src/main/assets")) }
    }

    private fun png(frame: SyntheticFrame) = FrameBuilder.encodePng(frame.mat)

    @Test
    fun sameSeedGivesIdenticalPngBytes() {
        val first = FrameBuilder.build(3, loaded.usable)
        val second = FrameBuilder.build(3, loaded.usable)
        assertEquals(first.seed, second.seed)
        assertContentEquals(png(first), png(second))
    }

    @Test
    fun differentSeedsGiveDifferentFrames() {
        val first = FrameBuilder.build(3, loaded.usable, seed = 1L)
        val second = FrameBuilder.build(3, loaded.usable, seed = 2L)
        assertFalse(png(first).contentEquals(png(second)))
    }

    @Test
    fun frameIndexSelectsTheSeed() {
        val seeds = (0 until FrameBuilder.FRAME_COUNT).map { FrameBuilder.seedFor(it) }
        assertEquals(seeds.size, seeds.toSet().size)
        assertEquals(seeds, (0 until FrameBuilder.FRAME_COUNT).map { FrameBuilder.seedFor(it) })
    }

    @Test
    fun framesCoverTemplatesIncludingEveryMaskedOne() {
        val frames = (0 until FrameBuilder.FRAME_COUNT).map { FrameBuilder.build(it, loaded.usable) }
        val covered = frames.flatMap { f -> f.cases.map { it.template.name } }.toSet()
        assertTrue(frames.size >= 12)
        assertTrue(covered.size >= 40, "covered ${covered.size}")
        val masked = loaded.usable.filter { it.masked }.map { it.name }
        assertTrue(masked.isNotEmpty())
        assertTrue(covered.containsAll(masked))
        Variant.values().filter { it != Variant.Recorded }.forEach { variant ->
            assertTrue(frames.any { f -> f.cases.any { it.variant == variant } }, variant.id)
        }
        masked.forEach { name ->
            Variant.values().filter { it != Variant.Recorded }.forEach { variant ->
                assertTrue(
                    frames.any { f -> f.cases.any { it.template.name == name && it.variant == variant } },
                    "$name $variant"
                )
            }
        }
    }

    @Test
    fun frameDiffersFromItsNoiseOnlyWhereTemplatePixelsWerePasted() {
        for (index in 0 until FrameBuilder.FRAME_COUNT) {
            val frame = FrameBuilder.build(index, loaded.usable)
            val noise = FrameBuilder.noise(Xorshift(frame.seed))
            val diff = Mat()
            Core.absdiff(frame.mat, noise, diff)
            val allowed = Mat.zeros(diff.rows(), diff.cols(), diff.type())
            frame.cases.filter { it.pasted != null }.distinctBy { it.pasted }.forEach { spec ->
                val box = assertNotNull(spec.pasted)
                val target = allowed.submat(Rect(box.x, box.y, box.width, box.height))
                if (spec.template.mask != null) spec.template.mask.copyTo(target)
                else target.setTo(org.opencv.core.Scalar(255.0))
                target.release()
            }
            val outside = Mat()
            Core.bitwise_not(allowed, outside)
            val stray = Mat()
            Core.bitwise_and(diff, diff, stray, outside)
            assertEquals(0, Core.countNonZero(stray), "frame $index has pixels that are neither noise nor template")
        }
    }

    @Test
    fun pastedAsIsMatchesItsOwnPositionWithTheFullScore() {
        val frame = FrameBuilder.build(0, loaded.usable)
        val pattern = io.github.fga_apple.host.imaging.CvPattern(frame.mat, ownsMat = false, tag = frame.name)
        val asIs = frame.cases.filter { it.variant == Variant.AsIs && it.kind == "local" }
        assertTrue(asIs.isNotEmpty())
        asIs.forEach { spec ->
            val (best, _) = GoldenScores.hits(pattern, spec.template, spec.region)
            val pasted = assertNotNull(spec.pasted)
            assertEquals(pasted.x, best.x, spec.template.name)
            assertEquals(pasted.y, best.y, spec.template.name)
            assertTrue(best.score > 0.999, "${spec.template.name} ${best.score}")
        }
    }

    @Test
    fun absentCasesAreScoredInPureNoise() {
        val frame = FrameBuilder.build(4, loaded.usable)
        val noise = FrameBuilder.noise(Xorshift(frame.seed))
        val absent = frame.cases.filter { it.variant == Variant.Absent }
        assertTrue(absent.isNotEmpty())
        absent.forEach { spec ->
            val rect = Rect(spec.region.x, spec.region.y, spec.region.width, spec.region.height)
            val diff = Mat()
            Core.absdiff(frame.mat.submat(rect), noise.submat(rect), diff)
            assertEquals(0, Core.countNonZero(diff), spec.template.name)
        }
    }
}
