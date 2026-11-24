package io.github.fga_apple.host.prefs

import io.github.fate_grand_automata.scripts.enums.BondCEEffectEnum
import io.github.fate_grand_automata.scripts.enums.BraveChainEnum
import io.github.fate_grand_automata.scripts.enums.CardPrioritySplit
import io.github.fate_grand_automata.scripts.enums.CardPriorityStyle
import io.github.fate_grand_automata.scripts.enums.CommandStartCombineMode
import io.github.fate_grand_automata.scripts.enums.CommandStartMode
import io.github.fate_grand_automata.scripts.enums.GameServer
import io.github.fate_grand_automata.scripts.enums.GameServers
import io.github.fate_grand_automata.scripts.enums.MaterialEnum
import io.github.fate_grand_automata.scripts.enums.ShuffleCardsEnum
import io.github.fate_grand_automata.scripts.enums.SupportClass
import io.github.fate_grand_automata.scripts.enums.SupportSelectionModeEnum
import io.github.fate_grand_automata.scripts.models.CardPriorityPerWave
import io.github.fate_grand_automata.scripts.models.PerEntrySettings
import io.github.fate_grand_automata.scripts.models.ServantCardPriorityPerWave
import io.github.fate_grand_automata.scripts.models.ServantPriorityPerWave
import io.github.fate_grand_automata.scripts.models.ServantSpamConfig
import io.github.fate_grand_automata.scripts.prefs.IBattleConfig
import io.github.fate_grand_automata.scripts.prefs.IExecutionProfile
import io.github.fate_grand_automata.scripts.prefs.ISupportPreferences
import kotlinx.serialization.json.Json

private val spamJson = Json {
    ignoreUnknownKeys = true
    encodeDefaults = true
}

class HostExecutionProfile(kv: KvStore, prefix: String = "") : IExecutionProfile {
    override var skillCommand by kv.string(prefix + "autoskill_cmd")
    override var cardPriority by kv.serialized(
        prefix + "card_priority", CardPriorityPerWave.default,
        { CardPriorityPerWave.of(it) }, { it.toString() }
    )
    override val cardPrioritySplit by kv.enum(prefix + "card_priority_split", CardPrioritySplit.Wave)
    override val cardPriorityStyle by kv.enum(prefix + "card_priority_style", CardPriorityStyle.Affinity)
    override val cardPriorityStyles by kv.serialized(
        prefix + "card_priority_styles", emptyList(),
        { PerEntrySettings.parseStyles(it) }, { PerEntrySettings.serializeStyles(it) }
    )
    override val servantCardPriority by kv.serialized(
        prefix + "card_priority_servant_list", ServantCardPriorityPerWave.default,
        { runCatching { ServantCardPriorityPerWave.of(it) }.getOrDefault(ServantCardPriorityPerWave.default) },
        { it.toString() }
    )
    override val useServantPriority by kv.bool(prefix + "use_servant_priority")
    override val useServantPriorities by kv.serialized(
        prefix + "use_servant_priorities", emptyList(),
        { PerEntrySettings.parseFlags(it) }, { PerEntrySettings.serializeFlags(it) }
    )
    override val servantPriority by kv.serialized(
        prefix + "servant_priority", ServantPriorityPerWave.default,
        { runCatching { ServantPriorityPerWave.of(it) }.getOrDefault(ServantPriorityPerWave.default) },
        { it.toString() }
    )
    override val rearrangeCards by kv.serialized(
        prefix + "auto_skill_rearrange_cards", emptyList(),
        { s -> s.split(",").map { it == "T" } }, { v -> v.joinToString(",") { if (it) "T" else "F" } }
    )
    override val braveChains by kv.serialized(
        prefix + "auto_skill_brave_chains", emptyList(),
        { s -> s.split(",").map { runCatching { enumValueOf<BraveChainEnum>(it) }.getOrDefault(BraveChainEnum.None) } },
        { v -> v.joinToString(",") { it.toString() } }
    )
}

class HostSupportPreferences(private val kv: KvStore) : ISupportPreferences {
    override val friendNames get() = kv.getStringSet("support_friend_names_list").toList()
    override val preferredServants get() = kv.getStringSet("support_pref_servant_list").toList()
    override val mlb by kv.bool("support_pref_ce_mlb")
    override val preferredCEs get() = kv.getStringSet("support_pref_ce_list").toList()
    override val preferredNoblePhantasms get() = kv.getStringSet("support_pref_np_list").toList()
    override val friendsOnly by kv.bool("support_friends_only")
    override val autoFriendRequest by kv.bool("support_auto_friend_request")
    override val selectionMode by kv.enum("support_mode", SupportSelectionModeEnum.Preferred)
    override val fallbackTo by kv.enum("support_fallback", SupportSelectionModeEnum.Manual)
    override val supportClass by kv.enum("autoskill_support_class", SupportClass.None)
    override val alsoCheckAll by kv.bool("also_check_all")
    override val maxAscended by kv.bool("support_max_ascended")
    override val skill1Max by kv.bool("support_skill_max_1")
    override val skill2Max by kv.bool("support_skill_max_2")
    override val skill3Max by kv.bool("support_skill_max_3")
    override val appendSkillCheck by kv.bool("support_append_skill_check")
    override val appendSkillsMax get() = (1..5).map { kv.getBool("support_append_skill_max_$it", false) }
    override val score62Check by kv.bool("support_score_62_check")
    override val grandScore16Check by kv.bool("support_grand_score_16_check")
    override val minimumNpLevel by kv.int("support_np_min_level", 5)
    override val npLevelCheck get() = kv.getBool("support_np_level_check", false) && minimumNpLevel > 1
    override val grandServant by kv.bool("support_grand_servant")
    override val bondCEEffect by kv.enum("support_bond_ce_effect", BondCEEffectEnum.Ignore)
    override val requireBothNormalAndRewardMatch by kv.bool("support_require_both_normal_and_reward_match")
}

class HostBattleConfig(override val id: String, private val kv: KvStore) :
    IBattleConfig, IExecutionProfile by HostExecutionProfile(kv) {

    override var name by kv.string("autoskill_name", "--")

    override val commandStartMode by kv.enum("command_start_mode", CommandStartMode.Basic)
    override val commandStartServant by kv.int("command_start_servant", 1)
    override val commandStartCardCount by kv.int("command_start_card_count", 2)
    override val commandStartBuster by kv.int("command_start_buster", 0)
    override val commandStartArts by kv.int("command_start_arts", 0)
    override val commandStartQuick by kv.int("command_start_quick", 0)
    override val commandStartCombineMode by kv.enum("command_start_combine_mode", CommandStartCombineMode.And)
    override val commandStartMaxTurns by kv.int("command_start_max_turns", 0)

    override val secondaryStartEnabled by kv.bool("secondary_start_enabled")
    override val secondaryStartServant by kv.int("secondary_start_servant", 2)
    override val secondaryStartCardCount by kv.int("secondary_start_card_count", 2)
    override val secondaryStartBuster by kv.int("secondary_start_buster", 0)
    override val secondaryStartArts by kv.int("secondary_start_arts", 0)
    override val secondaryStartQuick by kv.int("secondary_start_quick", 0)
    override val secondaryStartCombineMode by kv.enum("secondary_start_combine_mode", CommandStartCombineMode.And)
    override val secondaryExecution: IExecutionProfile = HostExecutionProfile(kv, "secondary_")

    override val tertiaryStartEnabled by kv.bool("tertiary_start_enabled")
    override val tertiaryStartServant by kv.int("tertiary_start_servant", 3)
    override val tertiaryStartCardCount by kv.int("tertiary_start_card_count", 2)
    override val tertiaryStartBuster by kv.int("tertiary_start_buster", 0)
    override val tertiaryStartArts by kv.int("tertiary_start_arts", 0)
    override val tertiaryStartQuick by kv.int("tertiary_start_quick", 0)
    override val tertiaryStartCombineMode by kv.enum("tertiary_start_combine_mode", CommandStartCombineMode.And)
    override val tertiaryExecution: IExecutionProfile = HostExecutionProfile(kv, "tertiary_")

    private val defaultSpam = (1..6).map { ServantSpamConfig() }
    override var spam: List<ServantSpamConfig> by kv.serialized(
        "spam_x", defaultSpam,
        { runCatching { spamJson.decodeFromString<List<ServantSpamConfig>>(it) }.getOrDefault(defaultSpam) },
        { spamJson.encodeToString(it) }
    )

    override val trackBond by kv.bool("bond_result_track")
    override val stopOnBondMax by kv.bool("bond_result_stop_on_max")
    override val bondMaxIgnoredCharacters: Set<Int>
        get() = (1..5).toSet() - kv.getString("bond_result_stop_characters", "1,2,3,4,5")
            .split(',').mapNotNull { it.toIntOrNull()?.takeIf { n -> n in 1..5 } }.toSet()

    override val shuffleCards by kv.enum("shuffle_cards", ShuffleCardsEnum.None)
    override val shuffleCardsWave by kv.int("shuffle_cards_wave", 3)
    override val cardSwapDetection by kv.bool("card_swap_detection")

    override val party by kv.int("autoskill_party", -1)
    override val materials: Set<MaterialEnum>
        get() = kv.getStringSet("battle_config_mat")
            .mapNotNull { runCatching { enumValueOf<MaterialEnum>(it) }.getOrNull() }.toSet()

    override val support: ISupportPreferences = HostSupportPreferences(kv)
    override val autoChooseTarget by kv.bool("auto_choose_target")
    override val server: GameServer?
        get() = runCatching { GameServers.deserialize(kv.getString("battle_config_server", "")) }.getOrNull()

    override val addRaidTurnDelay by kv.bool("add_raid_delay")
    override val raidTurnDelaySeconds by kv.int("raid_delay_seconds", 3)
    override val turnLimitRetreatEnabled by kv.bool("turn_limit_retreat_enabled")
    override val turnLimitRetreatTurns by kv.int("turn_limit_retreat_turns", 0)
    override val turnLimitRetreatStopAfterStreak by kv.bool("turn_limit_retreat_stop_after_streak")
    override val turnLimitRetreatStreakCount by kv.int("turn_limit_retreat_streak_count", 3)

    override fun export(): Map<String, *> = kv.export()
    override fun import(map: Map<String, *>) = kv.putAll(map)

    override fun equals(other: Any?) = if (other is IBattleConfig) other.id == id else super.equals(other)
    override fun hashCode() = id.hashCode()
}
