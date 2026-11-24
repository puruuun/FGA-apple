package io.github.fga_apple.host.prefs

import io.github.fate_grand_automata.scripts.enums.GameServer
import io.github.fate_grand_automata.scripts.enums.GameServers
import io.github.fate_grand_automata.scripts.enums.RefillResourceEnum
import io.github.fate_grand_automata.scripts.enums.ScriptModeEnum
import io.github.fate_grand_automata.scripts.modules.FpSellPolicy
import io.github.fate_grand_automata.scripts.modules.MissionPopupClear
import io.github.fate_grand_automata.scripts.modules.SkillLevelMode
import io.github.fate_grand_automata.scripts.prefs.IBattleConfig
import io.github.fate_grand_automata.scripts.prefs.IGesturesPreferences
import io.github.fate_grand_automata.scripts.prefs.IPerServerConfigPrefs
import io.github.fate_grand_automata.scripts.prefs.IPreferences
import io.github.fate_grand_automata.scripts.prefs.IServantEnhancementPreferences
import io.github.fate_grand_automata.scripts.prefs.ISkillLevelPreferences
import io.github.fate_grand_automata.scripts.prefs.ISupportPreferencesCommon
import io.github.lib_automata.PlatformPrefs
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import java.io.File
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.seconds

class HostPerServerConfig(override val server: GameServer, private val kv: KvStore) : IPerServerConfigPrefs {
    override var selectedAutoSkillKey by kv.string("autoskill_selected")
    override var rainbowApple by kv.int("rainbowApple", 0)
    override var goldApple by kv.int("goldApple", 0)
    override var silverApple by kv.int("silverApple", 0)
    override var blueApple by kv.int("blueApple", 0)
    override var copperApple by kv.int("copperApple", 0)
    override var waitForAPRegen by kv.bool("wait_for_ap_regeneration")
    override var useStormPod by kv.bool("quest_use_storm_pod")
    override var useTeapot by kv.bool("quest_use_teapot")
    override var selectedApple by kv.enum("selected_apple", RefillResourceEnum.Copper)

    override var currentAppleCount: Int
        get() = when (selectedApple) {
            RefillResourceEnum.Copper -> copperApple
            RefillResourceEnum.Bronze -> blueApple
            RefillResourceEnum.Silver -> silverApple
            RefillResourceEnum.Gold -> goldApple
            RefillResourceEnum.SQ -> rainbowApple
        }
        set(value) {
            when (selectedApple) {
                RefillResourceEnum.Copper -> copperApple = value
                RefillResourceEnum.Bronze -> blueApple = value
                RefillResourceEnum.Silver -> silverApple = value
                RefillResourceEnum.Gold -> goldApple = value
                RefillResourceEnum.SQ -> rainbowApple = value
            }
        }

    override val resources: List<RefillResourceEnum>
        get() = kv.getStringSet("refill_resource_x")
            .mapNotNull { runCatching { enumValueOf<RefillResourceEnum>(it) }.getOrNull() }
            .sortedBy { it.ordinal }

    override fun updateResources(resources: Set<RefillResourceEnum>) =
        kv.putStringSet("refill_resource_x", resources.map { it.name }.toSet())

    override var shouldLimitRuns by kv.bool("should_limit_runs")
    override var limitRuns by kv.int("limit_runs", 1)
    override var shouldLimitMats by kv.bool("should_limit_mats")
    override var limitMats by kv.int("limit_mats", 1)
    override var shouldLimitCEs by kv.bool("should_limit_ces")
    override var limitCEs by kv.int("limit_ces", 1)
}

class HostPreferences(root: JsonObject) : IPreferences {
    companion object {
        private const val BATTLE_CONFIG_ID = "host"

        fun load(file: File, exportedBattleConfig: File? = null): HostPreferences {
            val root = Json.parseToJsonElement(file.readText()).jsonObject
            if (exportedBattleConfig == null) return HostPreferences(root)
            val battle = Json.parseToJsonElement(exportedBattleConfig.readText()).jsonObject
            return HostPreferences(JsonObject(root + ("battleConfig" to battle)))
        }

        fun empty() = HostPreferences(JsonObject(emptyMap()))
    }

    private fun section(root: JsonObject, name: String) =
        KvStore(root[name]?.jsonObject ?: JsonObject(emptyMap()))

    private val kv = section(root, "global")
    val battleConfigRaw: JsonObject = root["battleConfig"]?.jsonObject ?: JsonObject(emptyMap())
    private val battleConfig = HostBattleConfig(BATTLE_CONFIG_ID, section(root, "battleConfig"))

    override var gameServer: GameServer =
        root["server"]?.jsonPrimitive?.content?.let { GameServers.deserialize(it) } ?: GameServer.Kr

    private val perServer = HostPerServerConfig(gameServer, section(root, "perServer")).also {
        it.selectedAutoSkillKey = BATTLE_CONFIG_ID
    }

    override var scriptMode by kv.enum("script_mode", ScriptModeEnum.Battle)
    override val battleConfigs: List<IBattleConfig> get() = listOf(battleConfig)
    override var showGameServers: List<GameServer> = listOf(gameServer)
    override var selectedServerConfigPref: IPerServerConfigPrefs
        get() = perServer
        set(_) {}
    override var selectedBattleConfig: IBattleConfig
        get() = battleConfig
        set(_) {}

    override val storySkip by kv.bool("story_skip")
    override val stopOnCEGet by kv.bool("stop_on_ce_get")
    override val stopOnFirstClearRewards by kv.bool("stop_on_first_clear_rewards")
    override val boostItemSelectionMode by kv.int("selected_boost_item", -1)

    override val useRootForScreenshots = false
    override val recordScreen = false
    override val skillDelay: Duration get() = kv.getInt("skill_delay", 500).milliseconds
    override val masterSkillMenuDelay: Duration
        get() = kv.getInt("master_skill_menu_delay", 1500).coerceIn(300, 3000).milliseconds
    override val screenshotDrops by kv.bool("screenshot_drops")
    override val screenshotDropsUnmodified by kv.bool("screenshot_drops_unmodified")
    override val screenshotBond by kv.bool("screenshot_bond")
    override val missionPopupClear by kv.bool("mission_popup_clear")
    override val missionPopupClearDuration: Duration
        get() = MissionPopupClear.clampSeconds(kv.getInt("mission_popup_clear_seconds", 10)).seconds
    override val supportReaderDiagnostics by kv.bool("support_reader_diagnostics")
    override var hidePlayButton by kv.bool("hide_play_button")
    override val hideSQInAPResources by kv.bool("hide_sq_in_ap_resources", true)
    override val bondTrackingFeatureEnabled by kv.bool("bond_tracking_feature_enabled", true)
    override val ignoreQuestItemUsage by kv.bool("ignore_teapot_storm_pod_usage", false)

    override var maxGoldEmberStackSize by kv.int("max_gold_ember_set_size", 1)
    override var maxGoldEmberTotalCount by kv.int("max_gold_ember_total_count", 100)
    override var stopAfterThisRun by kv.bool("stop_after_this_run")

    override var shouldLimitFP by kv.bool("should_fp_limit")
    override var limitFP by kv.int("fp_limit", 1)
    override var fpAutoSell by kv.bool("fpAutoSell", false)
    override var fpBannerPosition by kv.int("fpBannerPosition", 1)
    override var fpSellDryRun by kv.bool("fpSellDryRun", FpSellPolicy.DEFAULT_DRY_RUN)
    override val fpSellStableBuild = false

    override var receiveEmbersWhenGiftBoxFull by kv.bool("receive_embers_when_gift_box_full")

    override val stageCounterSimilarity get() = kv.getInt("stage_counter_similarity", 85) / 100.0
    override val stageCounterNew by kv.bool("stage_counter_new")
    override val waitBeforeTurn: Duration get() = kv.getInt("wait_before_turn", 500).milliseconds
    override val waitBeforeCards: Duration get() = kv.getInt("wait_before_cards", 2000).milliseconds

    override val support = object : ISupportPreferencesCommon {
        override val mlbSimilarity get() = kv.getInt("mlb_similarity", 70) / 100.0
        override val swipesPerUpdate get() = kv.getInt("support_swipes_per_update_x", 10)
        override val maxUpdates get() = kv.getInt("support_max_updates_x", 20)
    }

    override val platformPrefs = object : PlatformPrefs {
        override val debugMode get() = kv.getBool("debug_mode", false)
        override val minSimilarity get() = kv.getInt("min_similarity", 80) / 100.0
        override val waitMultiplier get() = kv.getInt("wait_multiplier", 100) / 100.0
        override val swipeMultiplier get() = kv.getInt("swipe_multiplier", 100) / 100.0
    }

    override val gestures = object : IGesturesPreferences {
        override val clickWaitTime: Duration get() = kv.getInt("click_wait_time", 300).milliseconds
        override val clickDuration: Duration get() = kv.getInt("click_duration", 50).milliseconds
        override val clickDelay: Duration get() = kv.getInt("click_delay", 10).milliseconds
        override val swipeWaitTime: Duration get() = kv.getInt("swipe_wait_time", 700).milliseconds
        override val swipeDuration: Duration get() = kv.getInt("swipe_duration", 300).milliseconds
    }

    override val servant = object : IServantEnhancementPreferences {
        override var shouldRedirectAscension by kv.bool("level_redirect_servant_ascension")
        override val shouldPerformAscension by kv.bool("level_perform_servant_ascension")
        override var shouldRedirectGrail by kv.bool("level_redirect_servant_grail")
    }

    override val skillLevel = object : ISkillLevelPreferences {
        private val held = mutableMapOf<SkillLevelMode, List<Int>>()
        override var checkOnly = false
        override fun targets(mode: SkillLevelMode): List<Int> {
            val saved = held[mode].orEmpty()
            return List(mode.slots) { saved.getOrElse(it) { 0 }.coerceIn(0, 10) }
        }

        override fun setTargets(mode: SkillLevelMode, targets: List<Int>) {
            held[mode] = targets.toList()
        }
    }

    override fun getPerServerConfigPref(server: GameServer): IPerServerConfigPrefs = perServer
    override fun addPerServerConfigPref(server: GameServer): IPerServerConfigPrefs = perServer

    override fun forBattleConfig(id: String): IBattleConfig = battleConfig
    override fun addBattleConfig(id: String): IBattleConfig = battleConfig
    override fun removeBattleConfig(id: String) {}
    override fun isOnboardingRequired() = false
    override fun completedOnboarding() {}

    var completedRuns = 0
        private set

    override fun updateCompletedRuns(runs: Int) { completedRuns = runs }
    override fun resetCompletedRuns() { completedRuns = 0 }
}
