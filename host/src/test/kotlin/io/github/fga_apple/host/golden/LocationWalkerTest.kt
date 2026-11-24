package io.github.fga_apple.host.golden

import io.github.lib_automata.Location
import io.github.lib_automata.Region
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertTrue

enum class Side { Left, Right }

sealed class Mark {
    data object One : Mark()
    sealed class Group : Mark() {
        data object Two : Group()
        data object Three : Group()
    }
}

class Inner {
    val corner = Location(5, 6)
    val box = Region(1, 2, 3, 4)
}

class Holder {
    val original = Location(1, 2)
    val added = Region(10, 20, 30, 40)
    val inner = Inner()
    val regions = listOf(Region(0, 0, 1, 1), Region(2, 2, 1, 1))
    val byKey = mapOf("b" to Location(2, 2), "a" to Location(1, 1))
    val broken: Region get() = throw IllegalStateException("no")
    val text = "ignored"
    val self: Holder get() = this

    fun at(side: Side) = Location(if (side == Side.Left) -1 else 1, 0)
    fun mark(mark: Mark) = Location(mark.hashCode() * 0 + (if (mark == Mark.One) 1 else 2), 0)
    fun slot(slot: Int) = Location(listOf(7, 8, 9)[slot], 0)
    fun unknown(region: Region) = region
}

class LocationWalkerTest {
    private fun walk(holder: Any = Holder()): WalkResult {
        val walker = LocationWalker(owns = { it.packageName == Holder::class.java.packageName && it.simpleName != "WalkResult" })
        walker.visit(holder)
        return walker.result()
    }

    @Test
    fun findsEveryLocationAndRegionProperty() {
        val leaves = walk().leaves
        assertEquals(LocationLeaf(Location(1, 2)), leaves["original"])
        assertEquals(RegionLeaf(Region(10, 20, 30, 40)), leaves["added"])
    }

    @Test
    fun followsNestedHoldersCollectionsAndMaps() {
        val leaves = walk().leaves
        assertEquals(LocationLeaf(Location(5, 6)), leaves["inner.corner"])
        assertEquals(RegionLeaf(Region(1, 2, 3, 4)), leaves["inner.box"])
        assertEquals(RegionLeaf(Region(2, 2, 1, 1)), leaves["regions[1]"])
        assertEquals(LocationLeaf(Location(1, 1)), leaves["byKey[a]"])
    }

    @Test
    fun cyclesAreVisitedOnce() {
        assertFalse(walk().leaves.keys.any { it.startsWith("self.") })
    }

    @Test
    fun throwingGetterIsRecordedAndOthersSurvive() {
        val leaves = walk().leaves
        assertEquals(ErrorLeaf("java.lang.IllegalStateException"), leaves["broken"])
        assertIs<LocationLeaf>(leaves["original"])
    }

    @Test
    fun functionsAreEnumeratedOverEnumsAndSealedTypes() {
        val leaves = walk().leaves
        assertEquals(LocationLeaf(Location(-1, 0)), leaves["at(Side.Left)"])
        assertEquals(LocationLeaf(Location(1, 0)), leaves["at(Side.Right)"])
        assertEquals(setOf("mark(Mark.One)", "mark(Mark.Group.Two)", "mark(Mark.Group.Three)"),
            leaves.keys.filter { it.startsWith("mark(") }.toSet())
    }

    @Test
    fun integerArgumentsKeepOnlyTheValidRange() {
        val result = walk()
        assertEquals(setOf("slot(0)", "slot(1)", "slot(2)"), result.leaves.keys.filter { it.startsWith("slot(") }.toSet())
        assertEquals(listOf("slot(3)", "slot(4)"), result.omittedArguments)
    }

    @Test
    fun argumentsThatCannotBeEnumeratedAreReported() {
        val result = walk()
        assertTrue(result.notEnumerated.any { it.startsWith("unknown(region:") })
        assertFalse(result.leaves.keys.any { it.startsWith("unknown(") })
    }

    @Test
    fun jsonIsSortedAndStable() {
        val document = mapOf("b" to mapOf("y" to 1, "x" to 2.5), "a" to listOf(1, 2))
        val first = GoldenJson.write(document)
        assertEquals("{\n  \"a\": [1, 2],\n  \"b\": {\"x\": 2.5, \"y\": 1}\n}\n", first)
        assertEquals(first, GoldenJson.write(document))
    }
}
