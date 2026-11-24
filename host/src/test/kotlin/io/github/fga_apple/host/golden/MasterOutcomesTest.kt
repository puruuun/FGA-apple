package io.github.fga_apple.host.golden

import io.github.fate_grand_automata.scripts.ScriptLog
import io.github.fate_grand_automata.scripts.enums.GameServer
import io.github.fate_grand_automata.scripts.enums.GameServers
import io.github.lib_automata.Location
import java.io.File
import kotlin.math.abs
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class MasterOutcomesTest {
    private val assets: File = generateSequence(File(System.getProperty("user.dir")).absoluteFile) { it.parentFile }
        .map { File(it, "fga/app/src/main/assets") }
        .first { it.isDirectory }

    private val shift = 25

    private class Outcome(val openClick: Location, val defaultOffsetLogged: Boolean)

    private fun measure(server: GameServer, scene: (GoldenComponent) -> GoldenScene): Outcome {
        val blank = GoldenLocations.build(server, 1920, 1080, GoldenScene.Blank, assets)
        val component = GoldenLocations.build(server, 1920, 1080, scene(blank), assets)
        try {
            val click = component.api().locations.battle.master.masterSkillOpenClick
            return Outcome(click, component.messages().logged.contains(ScriptLog.DefaultMasterOffset))
        } finally {
            listOf(blank, component).forEach {
                it.gestures().close()
                it.screenshots().close()
            }
        }
    }

    @Test
    fun blankFrameFallsBackToTheDefaultOffset() {
        for (server in GameServers.values) {
            val outcome = measure(server) { GoldenScene.Blank }
            assertTrue(outcome.defaultOffsetLogged, server.simple)
        }
    }

    @Test
    fun buttonFoundAtItsUsualPlaceIsNotTheFallback() {
        for (server in GameServers.values) {
            val outcome = measure(server) { GoldenLocations.foundScene(it) }
            assertFalse(outcome.defaultOffsetLogged, server.simple)
        }
    }

    @Test
    fun buttonFoundElsewhereMovesTheOffsetByTheDistanceItWasMoved() {
        for (server in GameServers.values) {
            val blank = measure(server) { GoldenScene.Blank }
            val moved = measure(server) { component ->
                val usual = GoldenLocations.foundScene(component).pasted!!
                GoldenScene(usual.copy(topLeft = Location(usual.topLeft.x - shift, usual.topLeft.y)))
            }
            assertFalse(moved.defaultOffsetLogged, server.simple)
            assertEquals(blank.openClick.y, moved.openClick.y, server.simple)
            assertTrue(abs(moved.openClick.x - (blank.openClick.x - shift)) <= 1, "${server.simple}: ${blank.openClick} -> ${moved.openClick}")
        }
    }
}
