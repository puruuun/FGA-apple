package io.github.fga_apple.host.platform

import io.github.fate_grand_automata.IStorageProvider
import io.github.fate_grand_automata.SupportImageKind
import io.github.fate_grand_automata.scripts.IScriptMessages
import io.github.fate_grand_automata.scripts.ScriptLog
import io.github.fate_grand_automata.scripts.ScriptNotify
import io.github.fate_grand_automata.scripts.enums.GameServer
import io.github.fga_apple.host.di.HostConfig
import io.github.lib_automata.OcrService
import io.github.lib_automata.OcrText
import io.github.lib_automata.Pattern
import io.github.lib_automata.TextLineOcrOptions
import java.io.File
import java.io.InputStream
import java.io.OutputStream
import java.lang.reflect.Modifier
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter
import javax.inject.Inject

fun describeScriptItem(item: Any): String {
    val fields = item.javaClass.declaredFields
        .filter { !Modifier.isStatic(it.modifiers) }
        .joinToString(", ") { f ->
            f.isAccessible = true
            "${f.name}=${runCatching { f.get(item) }.getOrNull()}"
        }
    return item.javaClass.simpleName + if (fields.isEmpty()) "" else "($fields)"
}

fun notifyLine(action: ScriptNotify) = "[notify] ${describeScriptItem(action)}"

fun logLine(item: ScriptLog) = "[log] ${describeScriptItem(item)}"

class ConsoleMessages @Inject constructor() : IScriptMessages {
    override fun notify(action: ScriptNotify) = println(notifyLine(action))
    override fun log(item: ScriptLog) = println(logLine(item))
}

class UnsupportedOcr @Inject constructor() : OcrService {
    private fun fail(): Nothing = throw UnsupportedOperationException(
        "이 호스트에는 아직 OCR이 없습니다. OCR이 필요한 설정(서포트 점수·보구 레벨 조건, 인연 추적 등)을 끄세요."
    )

    override fun detectText(pattern: Pattern): String = fail()
    override fun detectTextLine(pattern: Pattern, options: TextLineOcrOptions): OcrText = fail()
    override fun close() {}
}

class HostStorage @Inject constructor(private val config: HostConfig) : IStorageProvider {
    private fun dir(name: String) = File(config.dataDir, name).apply { mkdirs() }
    private fun stamp() = LocalDateTime.now().format(DateTimeFormatter.ofPattern("yyyyMMdd-HHmmss-SSS"))

    private fun kindDir(kind: SupportImageKind) = File(
        dir("support"),
        when (kind) {
            SupportImageKind.Servant -> "servant"
            SupportImageKind.CE -> "ce"
            SupportImageKind.Friend -> "friend"
            SupportImageKind.NoblePhantasm -> "np"
        }
    ).apply { mkdirs() }

    private fun save(pattern: Pattern, file: File) = file.outputStream().buffered().use { pattern.save(it) }

    override val supportImageTempDir: File get() = dir("support-temp")

    override fun writeSupportImage(kind: SupportImageKind, name: String): OutputStream =
        File(kindDir(kind), name).apply { parentFile.mkdirs() }.outputStream()

    override fun readSupportImage(kind: SupportImageKind, name: String): List<InputStream> {
        val target = File(kindDir(kind), name)
        val files = when {
            target.isDirectory -> target.listFiles { f -> f.isFile }?.sortedBy { it.name }.orEmpty()
            target.isFile -> listOf(target)
            else -> emptyList()
        }
        return files.map { it.inputStream() }
    }

    override fun list(kind: SupportImageKind): List<String> =
        kindDir(kind).listFiles()?.map { it.name }?.sorted().orEmpty()

    override fun dropScreenshot(patterns: List<Pattern>) {
        val base = stamp()
        patterns.forEachIndexed { i, p -> save(p, File(dir("drops"), "$base-$i.png")) }
    }

    override fun dropBondScreenShot(pattern: Pattern, server: GameServer) =
        save(pattern, File(dir("bond"), "${stamp()}-${server.simple}.png"))

    override fun dump(name: String, image: Pattern) = save(image, File(dir("dump"), "$name.png"))

    override fun createNoMediaFile() {}

    override fun writeDiagnosticFile(path: String, mimeType: String, write: (OutputStream) -> Unit) {
        val file = File(dir("diagnostics"), path)
        file.parentFile.mkdirs()
        file.outputStream().buffered().use(write)
    }
}
