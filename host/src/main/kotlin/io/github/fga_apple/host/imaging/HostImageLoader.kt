package io.github.fga_apple.host.imaging

import io.github.fate_grand_automata.SupportImageKind
import io.github.fate_grand_automata.scripts.IImageLoader
import io.github.fate_grand_automata.scripts.Images
import io.github.fate_grand_automata.scripts.enums.GameServer
import io.github.fate_grand_automata.scripts.enums.GameServers
import io.github.fate_grand_automata.scripts.enums.MaterialEnum
import io.github.fate_grand_automata.scripts.prefs.IPreferences
import io.github.fga_apple.host.di.HostConfig
import io.github.lib_automata.ColorManager
import io.github.lib_automata.Pattern
import io.github.lib_automata.dagger.ScriptScope
import java.io.File
import javax.inject.Inject

/**
 * 템플릿 이미지를 FGA-Modified의 assets 폴더에서 읽는다.
 * 규칙은 Android의 `app/.../util/ImageLoader.kt`와 같다: 서버 폴더에 있으면 그것을, 없으면 En 것을 쓴다.
 */
@ScriptScope
class HostImageLoader @Inject constructor(
    private val config: HostConfig,
    private val prefs: IPreferences,
    private val colorManager: ColorManager
) : IImageLoader {
    private data class CacheKey(val name: String, val gameServer: GameServer?, val isColor: Boolean)

    private fun key(name: String, gameServer: GameServer? = null) = CacheKey(name, gameServer, colorManager.isColor)

    private var currentGameServer: GameServer = GameServers.default
    private val cached = mutableMapOf<CacheKey, Pattern>()
    private val supportCached = mutableMapOf<CacheKey, List<Pattern>>()

    private fun createPattern(gameServer: GameServer, fileName: String, masked: Boolean): Pattern {
        val own = File(File(config.assetsDir, gameServer.simple), fileName)
        val file = if (own.isFile) own else File(File(config.assetsDir, GameServers.default.simple), fileName)
        check(file.isFile) { "템플릿 이미지가 없습니다: ${file.path}" }
        val tag = "${gameServer.simple}/$fileName"
        return file.inputStream().use {
            if (masked) CvPattern(it, colorManager.isColor, tag, masked = true)
            else CvPattern(it, colorManager.isColor, tag)
        }
    }

    override fun get(img: Images, gameServer: GameServer?): Pattern = synchronized(cached) {
        val server = prefs.gameServer
        if (currentGameServer != server) {
            clearImageCache()
            currentGameServer = server
        }
        cached.getOrPut(key(img.path, gameServer)) {
            createPattern(gameServer ?: currentGameServer, img.path, img.masked)
        }
    }

    override fun clearImageCache() = synchronized(cached) {
        cached.values.forEach { it.close() }
        cached.clear()
        clearSupportCache()
    }

    override fun clearSupportCache() = synchronized(supportCached) {
        supportCached.values.forEach { list -> list.forEach { it.close() } }
        supportCached.clear()
    }

    /**
     * 서포트 이미지는 `data/support/<종류>/<이름>`에서 읽고, 없으면 FGA 내장 이미지
     * (`assets/Support/<종류>/<이름>`)에서 읽는다. 이름이 폴더면 그 안의 PNG 전부를, 파일이면 그 하나를 쓴다.
     * 종류 폴더 이름은 servant, ce, friend, np.
     */
    override fun loadSupportPattern(kind: SupportImageKind, name: String): List<Pattern> = synchronized(supportCached) {
        supportCached.getOrPut(key("$kind:$name")) {
            val kindDir = when (kind) {
                SupportImageKind.Servant -> "servant"
                SupportImageKind.CE -> "ce"
                SupportImageKind.Friend -> "friend"
                SupportImageKind.NoblePhantasm -> "np"
            }
            // 사용자가 넣은 이미지가 먼저고, 없으면 FGA-Modified에 내장된 서번트·예장 이미지를 쓴다.
            // (Android 앱은 내장 이미지를 저장소에 풀어 놓고 같은 이름으로 읽는다.)
            val target = File(File(File(config.dataDir, "support"), kindDir), name).takeIf { it.exists() }
                ?: File(File(File(config.assetsDir, "Support"), kindDir), name)
            val files = when {
                target.isDirectory -> target.listFiles { f -> f.isFile && f.extension.equals("png", true) }
                    ?.sortedBy { it.name }.orEmpty()
                target.isFile -> listOf(target)
                else -> emptyList()
            }
            check(files.isNotEmpty()) { "서포트 이미지가 없습니다: ${target.path}" }
            files.mapIndexed { i, f -> f.inputStream().use { CvPattern(it, colorManager.isColor, "$name:$i") } }
        }
    }

    // Android에서는 재료 이미지가 앱 리소스(res/drawable)에 있고 이름 대응표가 app 모듈에 있다.
    // 재료 개수 제한 기능을 쓸 때 옮겨 온다.
    override fun loadMaterial(material: MaterialEnum): Pattern =
        throw UnsupportedOperationException("재료 이미지($material)는 아직 지원하지 않습니다")
}
