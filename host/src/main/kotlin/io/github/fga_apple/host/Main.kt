package io.github.fga_apple.host

import io.github.fate_grand_automata.scripts.Images
import io.github.fate_grand_automata.scripts.entrypoints.AutoBattle
import io.github.fate_grand_automata.scripts.entrypoints.isInSupport
import io.github.fga_apple.host.capture.MacFrameSource
import io.github.fga_apple.host.di.DaggerHostComponent
import io.github.fga_apple.host.di.DaggerRecordingHostComponent
import io.github.fga_apple.host.di.HostConfig
import io.github.fga_apple.host.di.HostGraph
import io.github.fga_apple.host.recording.HttpFrameSource
import io.github.fga_apple.host.recording.Recorder
import io.github.fga_apple.host.recording.RecordingException
import io.github.fga_apple.host.recording.recordingMeta
import io.github.fga_apple.host.prefs.HostPreferences
import io.github.fga_apple.host.imaging.CvPattern
import io.github.fga_apple.host.imaging.use
import io.github.lib_automata.Size
import org.opencv.core.Mat
import org.opencv.imgproc.Imgproc
import java.io.File

fun main(args: Array<String>) {
    when (args.firstOrNull()) {
        "probe" -> probe(args.drop(1))
        "screen" -> screen(args.drop(1))
        "battle" -> battle(args.drop(1))
        "regions" -> regions()
        else -> println(
            """
            usage:
              host battle <config.json> [--fga=<내보낸 전투 설정>] [--dry-run] [--allow-refill] [--record=<폴더>]
                                                           전투 스크립트 실행. data/stop 파일을 만들면 멈춤
                                                           --record: 화면과 동작을 폴더에 녹화 (docs/recording.md)
              host probe [server=Kr] [top=15] [frameUrl]   템플릿 전부를 화면 전체에서 찾아 점수 순으로 출력
              host screen [config.json]                    스크립트의 화면 판별 코드를 현재 화면에 돌려 봄 (터치 없음)
            """.trimIndent()
        )
    }
}

private fun regions() {
    val component = DaggerHostComponent.builder()
        .preferences(HostPreferences.empty())
        .config(HostConfig.load().copy(dryRun = true))
        .build()
    val t = component.transformer()
    val loc = component.api().locations
    val config = HostConfig.load()
    fun board(l: io.github.lib_automata.Location) =
        "${l.x.toLong() * 32767 / config.screenWidth} ${l.y.toLong() * 32767 / config.screenHeight}"

    println("image scale: ${component.scale().scriptToImage}")
    println("menuScreenRegion (image): ${t.toImage(loc.menuScreenRegion)}")
    println("support.screenCheckRegion (image): ${t.toImage(loc.support.screenCheckRegion)}")
    val quest = t.toScreen(loc.menuSelectQuestClick)
    println("menuSelectQuestClick: screen $quest, board ${board(quest)}")
    component.gestures().close()
}

private fun battle(args: List<String>) {
    val configFile = args.firstOrNull { !it.startsWith("--") }?.let(::File)
        ?: return println("전투 설정 파일이 필요합니다: host battle <config.json>")
    val dryRun = "--dry-run" in args
    val exported = args.firstOrNull { it.startsWith("--fga=") }?.removePrefix("--fga=")?.let(::File)
    val prefs = HostPreferences.load(configFile, exported)
    val server = prefs.selectedServerConfigPref
    val config = prefs.selectedBattleConfig

    println("전투 설정: ${config.name} (서버 ${prefs.gameServer})")
    println("  스킬 커맨드: ${config.skillCommand.ifBlank { "(없음)" }}")
    println("  서포트: ${config.support.selectionMode}, 대체 ${config.support.fallbackTo}")
    with(config.support) {
        println("    서번트 $preferredServants, 예장 $preferredCEs (풀돌 $mlb), 클래스 탭 $supportClass")
        println("    최종 재림 $maxAscended, 스킬 맥스 $skill1Max/$skill2Max/$skill3Max, 친구만 $friendsOnly")
        val ocr = listOfNotNull(
            "서포트 점수 +62".takeIf { score62Check }, "그랜드 +16".takeIf { grandScore16Check },
            "보구 레벨".takeIf { npLevelCheck }
        )
        if (ocr.isNotEmpty()) println("    주의: OCR이 필요한 조건이 켜져 있어 실행 중 멈춥니다: $ocr")
    }
    if (config.trackBond) println("  주의: 인연 추적은 OCR이 필요해 실행 중 멈춥니다")
    println("  실행 횟수 제한: ${if (server.shouldLimitRuns) "${server.limitRuns}회" else "없음"}")
    println("  AP 회복: ${server.resources.ifEmpty { "사용 안 함" }}, AP 자연 회복 대기 ${server.waitForAPRegen}")

    if (server.resources.isNotEmpty() && "--allow-refill" !in args) {
        return println("설정에 AP 회복 아이템 사용이 켜져 있습니다. 의도한 것이면 --allow-refill 을 붙이세요.")
    }

    val hostConfig = HostConfig.load().copy(dryRun = dryRun)
    val stopFile = File(hostConfig.dataDir, "stop")
    stopFile.delete()

    val recordArg = args.firstOrNull { it.startsWith("--record") }
    val recorder = if (recordArg == null) null else {
        val dir = recordArg.removePrefix("--record").removePrefix("=")
        if (dir.isBlank()) return println("--record=<폴더> 형식으로 폴더를 지정하세요.")
        try {
            Recorder.open(
                File(dir),
                HttpFrameSource(hostConfig.frameUrl),
                recordingMeta(
                    hostConfig.screenWidth, hostConfig.screenHeight, prefs.gameServer.simple,
                    prefs.battleConfigRaw, fgaCommit()
                )
            )
        } catch (e: RecordingException) {
            return println("녹화를 시작하지 못했습니다: ${e.message}")
        }
    }

    val graph: HostGraph = if (recorder == null) {
        DaggerHostComponent.builder().preferences(prefs).config(hostConfig).build()
    } else {
        DaggerRecordingHostComponent.builder().preferences(prefs).config(hostConfig).recorder(recorder).build()
    }
    val entry = try {
        graph.battle()
    } catch (e: Throwable) {
        recorder?.close()
        throw e
    }
    val done = java.util.concurrent.CountDownLatch(1)
    var exit: Exception? = null
    entry.scriptExitListener = {
        exit = it
        done.countDown()
    }

    println(if (dryRun) "시작 (dry-run: 터치를 보내지 않음)" else "시작")
    recorder?.start()
    entry.run()
    var recordingFailed = false
    while (!done.await(500, java.util.concurrent.TimeUnit.MILLISECONDS)) {
        if (stopFile.exists()) {
            println("data/stop 발견: 중단 요청")
            stopFile.delete()
            entry.stop()
        }
        if (!recordingFailed && recorder?.failure != null) {
            recordingFailed = true
            println("녹화 실패로 실행을 중단합니다: ${recorder.failure?.message}")
            entry.stop()
        }
    }

    val e = exit
    val reason = (e as? AutoBattle.ExitException)?.reason
    val exitText = reason?.let { it.javaClass.simpleName + (it.cause?.let { c -> " / $c" } ?: "") } ?: e.toString()
    println("종료: $exitText")
    if (reason is AutoBattle.ExitReason.Unexpected) reason.cause?.printStackTrace()
    println("완료한 실행 횟수: ${prefs.completedRuns}")
    var failure: Throwable? = null
    if (recorder != null) {
        runCatching { recorder.events.exit(recorder.events.now(), exitText, prefs.completedRuns) }
        runCatching { recorder.close() }
        failure = recorder.failure
    }
    graph.screenshotService().close()
    graph.gestures().close()
    if (failure != null) {
        println("녹화가 중간에 실패했습니다. 이 녹화는 쓸 수 없습니다: ${failure.message}")
        kotlin.system.exitProcess(1)
    }
    kotlin.system.exitProcess(0)
}

private fun fgaCommit(): String? = try {
    val process = ProcessBuilder("git", "-C", "fga", "rev-parse", "HEAD").redirectErrorStream(true).start()
    val text = process.inputStream.bufferedReader().readText().trim()
    if (process.waitFor() == 0 && text.isNotEmpty()) text else null
} catch (e: java.io.IOException) {
    null
}

private fun screen(args: List<String>) {
    val prefs = args.firstOrNull()?.let { HostPreferences.load(File(it)) } ?: HostPreferences.empty()
    val component = DaggerHostComponent.builder()
        .preferences(prefs)
        .config(HostConfig.load().copy(dryRun = true))
        .build()

    val area = component.gameArea()
    val scale = component.scale()
    println("server ${prefs.gameServer}, game area ${area.gameArea}")
    println("scale: screenToImage=${scale.screenToImage}, scriptToScreen=${scale.scriptToScreen}")

    val api = component.api()
    with(api) {
        useSameSnapIn {
            val checks = listOf(
                "support screen" to { isInSupport() },
                "menu (quest list)" to { images[Images.Menu] in locations.menuScreenRegion },
                "battle screen" to { images[Images.BattleScreen] in locations.battle.screenCheckRegion },
            )
            for ((name, check) in checks) {
                println("%-20s %s".format(name, runCatching { check() }.fold({ it.toString() }, { "error: $it" })))
            }
        }
    }
    println("auto-detected mode: ${runCatching { component.autoDetect().get() }.fold({ it.toString() }, { "error: $it" })}")
    component.screenshotService().close()
    component.gestures().close()
}

private fun probe(args: List<String>) {
    val server = args.getOrElse(0) { "Kr" }
    val top = args.getOrElse(1) { "15" }.toInt()
    val url = args.getOrElse(2) { HostConfig.load().frameUrl }
    val assets = File("fga/app/src/main/assets").let { if (it.isDirectory) it else File("../fga/app/src/main/assets") }

    val frame = MacFrameSource(url).grab()
    println("frame ${frame.image.width}x${frame.image.height}, age ${frame.ageMs} ms")

    val gray = Mat()
    Imgproc.cvtColor(frame.image.mat, gray, Imgproc.COLOR_BGR2GRAY)
    val scaledWidth = Math.round(frame.image.width * 720.0 / frame.image.height).toInt()
    val screen = CvPattern(gray, tag = "gray").use { it.resize(Size(scaledWidth, 720)) }
    println("compare image ${screen.width}x${screen.height}")

    val files = linkedMapOf<String, File>()
    for (dir in listOf("En", server)) {
        File(assets, dir).listFiles { f -> f.isFile && f.extension == "png" }?.forEach { files[it.name] = it }
    }

    val results = files.values.mapNotNull { file ->
        file.inputStream().use { CvPattern(it, isColor = false, tag = file.parentFile.name + "/" + file.name) }.use { template ->
            screen.findMatches(template, 0.0).firstOrNull()?.let { Triple(template.tag, it.score, it.region) }
        }
    }.sortedByDescending { it.second }

    println("templates tried: ${files.size}")
    for ((tag, score, region) in results.take(top)) {
        println("%.3f  %-40s at %d,%d  %dx%d".format(score, tag, region.x, region.y, region.width, region.height))
    }
    screen.close()
}

private inline fun <T> CvPattern.use(block: (CvPattern) -> T): T =
    try {
        block(this)
    } finally {
        close()
    }
