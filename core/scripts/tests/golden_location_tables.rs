use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use automata::kotlin_math::round_to_int;
use automata::{
    GameAreaManager, Location, Match, RealScale, RealTransformer, Region, Scale, Size, Transformer,
};
use scripts::enums::game_server_enum::{En, GameServer, GameServers, Jp};
use scripts::enums::refill_resource_enum::RefillResourceEnum;
use scripts::enums::support_class::SupportClass;
use scripts::fgo_game_area_manager::FgoGameAreaManager;
use scripts::locations::attack_screen_locations::AttackScreenLocations;
use scripts::locations::battle_screen_locations::BattleScreenLocations;
use scripts::locations::bond_locations::BondLocations;
use scripts::locations::ce_enhance_locations::CEEnhanceLocations;
use scripts::locations::fp_locations::FPLocations;
use scripts::locations::fp_sell_locations::FpSellLocations;
use scripts::locations::location_error::LocationResult;
use scripts::locations::locations::Locations;
use scripts::locations::lottery_locations::LotteryLocations;
use scripts::locations::master_locations::{BattleMenuFinder, MasterLocations};
use scripts::locations::quest_item_locations::QuestItemLocations;
use scripts::locations::quest_panel_locations::QuestPanelLocations;
use scripts::locations::script_area_transforms::{
    IScriptAreaTransforms, ScriptAreaTransforms, Transforms,
};
use scripts::locations::servant_level_locations::ServantLevelLocations;
use scripts::locations::skill_level_locations::SkillLevelLocations;
use scripts::locations::support_np_layout::SupportNpLayout;
use scripts::locations::support_score_layout::SupportScoreLayout;
use scripts::locations::support_screen_locations::SupportScreenLocations;
use scripts::locations::support_skill_layout::SupportSkillLayout;
use scripts::locations::weekly_route_locations::WeeklyRouteLocations;
use scripts::models::boost_item::Enabled;
use scripts::models::command_card::{CommandCard, Face, NP};
use scripts::models::enemy_target::{EnemyTarget, Normal};
use scripts::models::field_slot::FieldSlot;
use scripts::models::order_change_member::{OrderChangeMember, Starting, Sub};
use scripts::models::servant_target::{ServantTarget, SpecialTarget};
use scripts::models::skill::{CommandSpell, Master, Servant};
use scripts::modules::skill_level_flow::SkillLevelMode;
use serde_json::Value as Json;

const FILES: usize = 77;
const ENTRIES_PER_FILE: usize = 501;

const MASTER_PATHS: [&str; 5] = [
    "battle.master.locate(Skill.Master.A)",
    "battle.master.locate(Skill.Master.B)",
    "battle.master.locate(Skill.Master.C)",
    "battle.master.masterSkillOpenClick",
    "battle.master.stageCountRegion",
];

const BATTLE_MENU_SIZES: [(&str, i32, i32); 5] = [
    ("Cn", 42, 20),
    ("En", 65, 15),
    ("Jp", 40, 18),
    ("Kr", 40, 21),
    ("Tw", 65, 15),
];

fn battle_menu_size(server: GameServer) -> Result<(i32, i32), String> {
    let lookup = |folder: &str| {
        BATTLE_MENU_SIZES
            .iter()
            .find(|(name, _, _)| *name == folder)
            .map(|(_, w, h)| (*w, *h))
    };
    lookup(server.simple())
        .or_else(|| lookup(GameServers::DEFAULT.simple()))
        .ok_or_else(|| format!("no battle_menu size for {server}"))
}

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/golden/locations")
}

fn golden_files() -> Result<Vec<PathBuf>, String> {
    let dir = golden_dir();
    let entries = fs::read_dir(&dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    let mut files = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|e| format!("cannot list {}: {e}", dir.display()))?
            .path();
        if path.extension().is_some_and(|ext| ext == "json") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Value {
    Null,
    Int(i32),
    Loc(Location),
    Reg(Region),
}

struct Call<'a> {
    path: &'a str,
    args: Vec<&'a str>,
    index: Option<usize>,
}

fn parse_call(name: &str) -> Result<Call<'_>, String> {
    let (body, index) = match name.strip_suffix(']') {
        Some(rest) => {
            let open = rest
                .rfind('[')
                .ok_or_else(|| format!("{name}: unbalanced index"))?;
            let digits = &rest[open + 1..];
            let index = digits
                .parse::<usize>()
                .map_err(|_| format!("{name}: bad index {digits}"))?;
            (&rest[..open], Some(index))
        }
        None => (name, None),
    };
    match body.find('(') {
        None => Ok(Call {
            path: body,
            args: Vec::new(),
            index,
        }),
        Some(open) => {
            let inner = body[open + 1..]
                .strip_suffix(')')
                .ok_or_else(|| format!("{name}: unbalanced call"))?;
            Ok(Call {
                path: &body[..open],
                args: inner.split(", ").collect(),
                index,
            })
        }
    }
}

fn find_by_name<T: Copy + std::fmt::Debug>(all: &[T], name: &str) -> Option<T> {
    all.iter()
        .copied()
        .find(|value| format!("{value:?}") == name)
}

fn face(arg: &str) -> Result<Face, String> {
    arg.strip_prefix("CommandCard.Face.")
        .and_then(|name| find_by_name(&Face::LIST, name))
        .ok_or_else(|| format!("unmapped argument {arg}"))
}

fn command_card(arg: &str) -> Result<CommandCard, String> {
    if let Some(name) = arg.strip_prefix("CommandCard.NP.") {
        return find_by_name(&NP::LIST, name)
            .map(CommandCard::NP)
            .ok_or_else(|| format!("unmapped argument {arg}"));
    }
    face(arg).map(CommandCard::Face)
}

fn normal(arg: &str) -> Result<Normal, String> {
    arg.strip_prefix("EnemyTarget.")
        .and_then(|name| find_by_name(&[Normal::A, Normal::B, Normal::C], name))
        .ok_or_else(|| format!("unmapped argument {arg}"))
}

fn enemy_target(arg: &str) -> Result<EnemyTarget, String> {
    arg.strip_prefix("EnemyTarget.")
        .and_then(|name| find_by_name(&EnemyTarget::ALL, name))
        .ok_or_else(|| format!("unmapped argument {arg}"))
}

fn order_change_member(arg: &str) -> Result<OrderChangeMember, String> {
    if let Some(name) = arg.strip_prefix("OrderChangeMember.Starting.") {
        return find_by_name(&Starting::LIST, name)
            .map(OrderChangeMember::Starting)
            .ok_or_else(|| format!("unmapped argument {arg}"));
    }
    arg.strip_prefix("OrderChangeMember.Sub.")
        .and_then(|name| find_by_name(&Sub::LIST, name))
        .map(OrderChangeMember::Sub)
        .ok_or_else(|| format!("unmapped argument {arg}"))
}

fn servant_target(arg: &str) -> Result<ServantTarget, String> {
    if let Some(name) = arg.strip_prefix("ServantTarget.SpecialTarget.") {
        return find_by_name(&SpecialTarget::LIST, name)
            .map(ServantTarget::SpecialTarget)
            .ok_or_else(|| format!("unmapped argument {arg}"));
    }
    arg.strip_prefix("ServantTarget.")
        .and_then(|name| find_by_name(&ServantTarget::LIST, name))
        .ok_or_else(|| format!("unmapped argument {arg}"))
}

fn servant_skill(arg: &str) -> Result<Servant, String> {
    arg.strip_prefix("Skill.Servant.")
        .and_then(|name| find_by_name(&Servant::LIST, name))
        .ok_or_else(|| format!("unmapped argument {arg}"))
}

fn master_skill(arg: &str) -> Result<Master, String> {
    arg.strip_prefix("Skill.Master.")
        .and_then(|name| find_by_name(&Master::LIST, name))
        .ok_or_else(|| format!("unmapped argument {arg}"))
}

fn command_spell(arg: &str) -> Result<CommandSpell, String> {
    arg.strip_prefix("Skill.CommandSpell.")
        .and_then(|name| find_by_name(&CommandSpell::LIST, name))
        .ok_or_else(|| format!("unmapped argument {arg}"))
}

fn field_slot(arg: &str) -> Result<FieldSlot, String> {
    arg.strip_prefix("FieldSlot.")
        .and_then(|name| find_by_name(&FieldSlot::LIST, name))
        .ok_or_else(|| format!("unmapped argument {arg}"))
}

fn boost(arg: &str) -> Result<Enabled, String> {
    arg.strip_prefix("BoostItem.Enabled.")
        .and_then(|name| {
            find_by_name(
                &[
                    Enabled::Skip,
                    Enabled::BoostItem1,
                    Enabled::BoostItem2,
                    Enabled::BoostItem3,
                ],
                name,
            )
        })
        .ok_or_else(|| format!("unmapped argument {arg}"))
}

fn refill(arg: &str) -> Result<RefillResourceEnum, String> {
    arg.strip_prefix("RefillResourceEnum.")
        .and_then(RefillResourceEnum::from_name)
        .ok_or_else(|| format!("unmapped argument {arg}"))
}

fn support_class(arg: &str) -> Result<SupportClass, String> {
    arg.strip_prefix("SupportClass.")
        .and_then(SupportClass::from_name)
        .ok_or_else(|| format!("unmapped argument {arg}"))
}

fn level_mode(arg: &str) -> Result<SkillLevelMode, String> {
    match arg {
        "SkillLevelMode.Skill" => Ok(SkillLevelMode::Skill),
        "SkillLevelMode.Append" => Ok(SkillLevelMode::Append),
        _ => Err(format!("unmapped argument {arg}")),
    }
}

fn game_server(arg: &str) -> Result<GameServer, String> {
    match arg {
        "GameServer.En.Original" => Ok(GameServer::En(En::Original)),
        "GameServer.En.BetterFGO" => Ok(GameServer::En(En::BetterFgo)),
        "GameServer.Jp.Original" => Ok(GameServer::Jp(Jp::Original)),
        "GameServer.Jp.BetterFGO" => Ok(GameServer::Jp(Jp::BetterFgo)),
        "GameServer.Cn" => Ok(GameServer::Cn),
        "GameServer.Tw" => Ok(GameServer::Tw),
        "GameServer.Kr" => Ok(GameServer::Kr),
        _ => Err(format!("unmapped argument {arg}")),
    }
}

fn int(arg: &str) -> Result<i32, String> {
    arg.parse::<i32>()
        .map_err(|_| format!("unmapped argument {arg}"))
}

fn lift<T>(result: LocationResult<T>) -> Result<T, String> {
    result.map_err(|e| e.to_string())
}

struct Env {
    locations: Locations,
    ce_enhance: CEEnhanceLocations,
    fp_sell: FpSellLocations,
    quest_panel: QuestPanelLocations,
    skill_level: SkillLevelLocations,
    weekly: WeeklyRouteLocations,
}

fn build_env(transforms: &Transforms, finder: Box<dyn BattleMenuFinder>) -> Result<Env, String> {
    let master = lift(MasterLocations::new(transforms.clone(), finder))?;
    let locations = lift(Locations::new(
        transforms.clone(),
        lift(FPLocations::new(transforms.clone()))?,
        lift(LotteryLocations::new(transforms.clone()))?,
        lift(SupportScreenLocations::new(transforms.clone()))?,
        AttackScreenLocations::new(transforms.clone()),
        lift(BattleScreenLocations::new(transforms.clone(), master))?,
        lift(ServantLevelLocations::new(transforms.clone()))?,
        lift(QuestItemLocations::new(transforms.clone()))?,
        lift(BondLocations::new(transforms.clone()))?,
    ))?;
    Ok(Env {
        locations,
        ce_enhance: lift(CEEnhanceLocations::new(transforms.clone()))?,
        fp_sell: lift(FpSellLocations::new(transforms.clone()))?,
        quest_panel: lift(QuestPanelLocations::new(transforms.clone()))?,
        skill_level: lift(SkillLevelLocations::new(transforms.clone()))?,
        weekly: lift(WeeklyRouteLocations::new(transforms.clone()))?,
    })
}

struct NothingOnScreen;

impl BattleMenuFinder for NothingOnScreen {
    fn find_battle_menu(&self, _search: Region) -> Option<Match> {
        None
    }
}

struct PastedTemplate {
    frame: Region,
    pasted: Region,
    script_to_image: f64,
}

impl BattleMenuFinder for PastedTemplate {
    fn find_battle_menu(&self, search: Region) -> Option<Match> {
        let wanted = (search * self.script_to_image).ok()?;
        let crop = self.frame.clip(&wanted).ok()?;
        if !crop.contains(&self.pasted) {
            return None;
        }
        let relative = Region::new(
            self.pasted.x() - crop.x(),
            self.pasted.y() - crop.y(),
            self.pasted.width(),
            self.pasted.height(),
        )
        .ok()?;
        let script = (relative * (1.0 / self.script_to_image)).ok()?;
        Some(Match::new(script + search.location(), 1.0))
    }
}

fn one(value: Value) -> Result<Vec<Value>, String> {
    Ok(vec![value])
}

fn loc(value: Location) -> Result<Vec<Value>, String> {
    one(Value::Loc(value))
}

fn reg(value: Region) -> Result<Vec<Value>, String> {
    one(Value::Reg(value))
}

fn regions(values: Vec<Region>) -> Result<Vec<Value>, String> {
    Ok(values.into_iter().map(Value::Reg).collect())
}

fn evaluate(env: &Env, name: &str) -> Result<Value, String> {
    let call = parse_call(name)?;
    let values = evaluate_call(env, &call)?;
    match call.index {
        Some(index) => values
            .get(index)
            .copied()
            .ok_or_else(|| format!("{name}: index {index} out of {}", values.len())),
        None => match values.as_slice() {
            [single] => Ok(*single),
            other => Err(format!("{name}: expected one value, got {}", other.len())),
        },
    }
}

fn evaluate_call(env: &Env, call: &Call<'_>) -> Result<Vec<Value>, String> {
    let l = &env.locations;
    let args = call.args.as_slice();
    match (call.path, args) {
        ("edgeInset", []) => one(Value::Int(l.edge_inset())),
        ("scriptArea", []) => reg(l.script_area()),

        ("continueRegion", []) => reg(l.continue_region),
        ("continueBoostClick", []) => loc(l.continue_boost_click),
        ("missionPopupClearClick", []) => loc(l.mission_popup_clear_click),
        ("inventoryFullRegion", []) => reg(l.inventory_full_region),
        ("ordealCallConfirmPodUseRegion", []) => reg(l.ordeal_call_confirm_pod_use_region),
        ("ordealCallOutOfPodsRegion", []) => reg(l.ordeal_call_out_of_pods_region),
        ("ordealCallOutOfPodsClick", []) => loc(l.ordeal_call_out_of_pods_click),
        ("interludeCloseClick", []) => loc(l.interlude_close_click),
        ("interludeEndScreenClose", []) => reg(l.interlude_end_screen_close),
        ("menuScreenRegion", []) => reg(l.menu_screen_region),
        ("menuSelectQuestClick", []) => loc(l.menu_select_quest_click),
        ("menuStartQuestClick", []) => loc(l.menu_start_quest_click),
        ("menuStorySkipYesClick", []) => loc(l.menu_story_skip_yes_click),
        ("retryRegion", []) => reg(l.retry_region),
        ("staminaScreenRegion", []) => reg(l.stamina_screen_region),
        ("staminaOkClick", []) => loc(l.stamina_ok_click),
        ("staminaCloseClick", []) => loc(l.stamina_close_click),
        ("withdrawRegion", []) => reg(l.withdraw_region),
        ("withdrawAcceptClick", []) => loc(l.withdraw_accept_click),
        ("withdrawCloseClick", []) => loc(l.withdraw_close_click),
        ("turnLimitRetreatRegion", []) => reg(l.turn_limit_retreat_region),
        ("turnLimitRetreatClick", []) => loc(l.turn_limit_retreat_click),
        ("turnLimitRetreatOkRegion", []) => reg(l.turn_limit_retreat_ok_region),
        ("battleMenuCloseClick", []) => loc(l.battle_menu_close_click),
        ("selectedPartyRegion", []) => reg(l.selected_party_region),
        ("partySelectionArray", []) => Ok(l
            .party_selection_array
            .iter()
            .copied()
            .map(Value::Loc)
            .collect()),
        ("menuStorySkipRegion", []) => reg(l.menu_story_skip_region),
        ("menuStorySkipClick", []) => loc(l.menu_story_skip_click),
        ("resultFriendRequestRegion", []) => reg(l.result_friend_request_region),
        ("resultFriendRequestRejectClick", []) => loc(l.result_friend_request_reject_click),
        ("resultFriendRequestSendClick", []) => loc(l.result_friend_request_send_click),
        ("resultMatRewardsRegion", []) => reg(l.result_mat_rewards_region),
        ("resultClick", []) => loc(l.result_click),
        ("resultQuestRewardRegion", []) => reg(l.result_quest_reward_region),
        ("resultDropScrollbarRegion", []) => reg(l.result_drop_scrollbar_region),
        ("resultDropScrollEndClick", []) => loc(l.result_drop_scroll_end_click),
        ("resultMasterExpRegion", []) => reg(l.result_master_exp_region),
        ("resultMasterLvlUpRegion", []) => reg(l.result_master_lvl_up_region),
        ("resultScreenRegion", []) => reg(l.result_screen_region),
        ("resultBondRegion", []) => reg(l.result_bond_region),
        ("resultCeRewardRegion", []) => reg(l.result_ce_reward_region),
        ("resultCeRewardDetailsRegion", []) => reg(l.result_ce_reward_details_region),
        ("resultCeRewardCloseClick", []) => loc(l.result_ce_reward_close_click),
        ("giftBoxSwipeStart", []) => loc(l.gift_box_swipe_start),
        ("giftBoxSwipeEnd", []) => loc(l.gift_box_swipe_end),
        ("emptyEnhanceRegion", []) => reg(l.empty_enhance_region),
        ("npStartedRegion", []) => reg(l.np_started_region),
        ("rankUpRegion", []) => reg(l.rank_up_region),
        ("middleOfScreenClick", []) => loc(l.middle_of_screen_click),
        ("enhancementBannerRegion", []) => reg(l.enhancement_banner_region),
        ("enhancementClick", []) => loc(l.enhancement_click),
        ("tempServantEnhancementRegion", []) => reg(l.temp_servant_enhancement_region),
        ("enhancementSkipRapidClick", []) => loc(l.enhancement_skip_rapid_click),
        ("tempServantEnhancementLocation", []) => loc(l.temp_servant_enhancement_location),
        ("locate", [arg]) if arg.starts_with("BoostItem.") => loc(l.locate_boost(boost(arg)?)),
        ("locate", [arg]) if arg.starts_with("RefillResourceEnum.") => Ok(l
            .locate_refill_resource(refill(arg)?)
            .into_iter()
            .map(Value::Loc)
            .collect()),

        ("attack.backClick", []) => loc(l.attack.back_click),
        ("attack.clickLocation", [arg]) => loc(l.attack.click_location(command_card(arg)?)),
        ("attack.affinityRegion", [arg]) => reg(lift(l.attack.affinity_region(face(arg)?))?),
        ("attack.typeRegion", [arg]) => reg(lift(l.attack.type_region(face(arg)?))?),
        ("attack.swapIconRegion", [arg]) => reg(lift(l.attack.swap_icon_region(face(arg)?))?),
        ("attack.servantMatchRegion", [arg]) => {
            reg(lift(l.attack.servant_match_region(face(arg)?))?)
        }
        ("attack.supportCheckRegion", [arg]) => {
            reg(lift(l.attack.support_check_region(face(arg)?))?)
        }

        ("battle.attackClick", []) => loc(l.battle.attack_click),
        ("battle.battleSafeMiddleOfScreenClick", []) => {
            loc(l.battle.battle_safe_middle_of_screen_click)
        }
        ("battle.extraInfoWindowCloseClick", []) => loc(l.battle.extra_info_window_close_click),
        ("battle.orderChangeOkClick", []) => loc(l.battle.order_change_ok_click),
        ("battle.screenCheckRegion", []) => reg(l.battle.screen_check_region),
        ("battle.servantDetailsFaceCardRegion", []) => {
            reg(l.battle.servant_details_face_card_region)
        }
        ("battle.servantDetailsInfoClick", []) => loc(l.battle.servant_details_info_click),
        ("battle.shuffleNPShortageRegion", []) => reg(l.battle.shuffle_np_shortage_region),
        ("battle.skillOkClick", []) => loc(l.battle.skill_ok_click),
        ("battle.skillUseRegion", []) => reg(l.battle.skill_use_region),
        ("battle.dangerRegion", [arg]) => reg(lift(l.battle.danger_region(normal(arg)?))?),
        ("battle.imageRegion", [arg]) => reg(lift(l.battle.image_region(servant_skill(arg)?))?),
        ("battle.servantChangeCheckRegion", [arg]) => reg(lift(
            l.battle.servant_change_check_region(field_slot(arg)?),
        )?),
        ("battle.servantChangeSupportCheckRegion", [arg]) => reg(lift(
            l.battle
                .servant_change_support_check_region(field_slot(arg)?),
        )?),
        ("battle.servantOpenDetailsClick", [arg]) => {
            loc(l.battle.servant_open_details_click(field_slot(arg)?))
        }
        ("battle.servantPresentRegion", [arg]) => {
            reg(lift(l.battle.servant_present_region(field_slot(arg)?))?)
        }
        ("battle.locate", [arg]) if arg.starts_with("EnemyTarget.") => {
            loc(l.battle.locate_enemy(enemy_target(arg)?))
        }
        ("battle.locate", [arg]) if arg.starts_with("OrderChangeMember.") => loc(l
            .battle
            .locate_order_change_member(order_change_member(arg)?)),
        ("battle.locate", [arg]) if arg.starts_with("ServantTarget.") => {
            match l.battle.locate_servant_target(servant_target(arg)?) {
                Some(location) => loc(location),
                None => one(Value::Null),
            }
        }
        ("battle.locate", [arg]) if arg.starts_with("Skill.Servant.") => {
            loc(l.battle.locate_skill(servant_skill(arg)?))
        }

        ("battle.master.battleMenuSearchRegion", []) => {
            reg(lift(l.battle.master.battle_menu_search_region())?)
        }
        ("battle.master.cancelCommandSpellRegion", []) => {
            reg(l.battle.master.cancel_command_spell_region)
        }
        ("battle.master.commandSpellClick", []) => loc(l.battle.master.command_spell_click),
        ("battle.master.locate", [arg]) if arg.starts_with("Skill.CommandSpell.") => {
            loc(l.battle.master.locate_command_spell(command_spell(arg)?))
        }
        ("battle.master.locate", [arg]) if arg.starts_with("Skill.Master.") => {
            loc(lift(l.battle.master.locate_master(master_skill(arg)?))?)
        }
        ("battle.master.masterSkillOpenClick", []) => {
            loc(lift(l.battle.master.master_skill_open_click())?)
        }
        ("battle.master.stageCountRegion", []) => reg(lift(l.battle.master.stage_count_region())?),

        ("bond.resultTitle", []) => reg(l.bond.result_title),
        ("bond.popupTitle", []) => reg(l.bond.popup_title),
        ("bond.popupArrow", []) => reg(l.bond.popup_arrow),
        ("bond.gain", [arg]) => reg(lift(l.bond.gain(int(arg)?))?),
        ("bond.gainText", [arg]) => reg(lift(l.bond.gain_text(int(arg)?))?),
        ("bond.levelUpMarker", [arg]) => reg(lift(l.bond.level_up_marker(int(arg)?))?),

        ("fp.continueSummonClick", []) => loc(l.fp.continue_summon_click),
        ("fp.continueSummonRegion", []) => reg(l.fp.continue_summon_region),
        ("fp.first10SummonClick", []) => loc(l.fp.first10_summon_click),
        ("fp.initialSummonCheck", []) => reg(l.fp.initial_summon_check),
        ("fp.initialSummonClick", []) => loc(l.fp.initial_summon_click),
        ("fp.initialSummonContinueClick", []) => reg(l.fp.initial_summon_continue_click),
        ("fp.okClick", []) => loc(l.fp.ok_click),
        ("fp.skipRapidClick", []) => loc(l.fp.skip_rapid_click),
        ("fp.summonCheck", []) => reg(l.fp.summon_check),

        ("lottery.checkRegion", []) => reg(l.lottery.check_region),
        ("lottery.doneRegion", []) => reg(l.lottery.done_region),
        ("lottery.finishedRegion", []) => reg(l.lottery.finished_region),
        ("lottery.fullPresentBoxRegion", []) => reg(l.lottery.full_present_box_region),
        ("lottery.spinClick", []) => loc(l.lottery.spin_click),
        ("lottery.transitionRegion", []) => reg(l.lottery.transition_region),

        ("questItems.dialogClose", []) => reg(l.quest_items.dialog_close),
        ("questItems.dialogHeading", []) => reg(l.quest_items.dialog_heading),
        ("questItems.mainControl", []) => reg(l.quest_items.main_control),
        ("questItems.stormPodControl", []) => reg(l.quest_items.storm_pod_control),
        ("questItems.teapotControl", []) => reg(l.quest_items.teapot_control),

        ("servant.ascensionReturnToLevelRegion", []) => {
            reg(l.servant.ascension_return_to_level_region)
        }
        ("servant.autoFillStateRegion", []) => reg(l.servant.auto_fill_state_region),
        ("servant.autoSelectMinEmberLowQPLocation", []) => {
            loc(l.servant.auto_select_min_ember_low_qp_location)
        }
        ("servant.autoSelectMinEmberLowQPRegion", []) => {
            reg(l.servant.auto_select_min_ember_low_qp_region)
        }
        ("servant.emberConfirmationDialogRegion", []) => {
            reg(l.servant.ember_confirmation_dialog_region)
        }
        ("servant.emptyEmberOrQPDialogRegion", []) => {
            reg(l.servant.empty_ember_or_qp_dialog_region)
        }
        ("servant.finalConfirmRegion", []) => reg(l.servant.final_confirm_region),
        ("servant.returnToServantMenuFromAscensionLocation", []) => {
            loc(l.servant.return_to_servant_menu_from_ascension_location)
        }
        ("servant.servantAutoSelectRegion", []) => reg(l.servant.servant_auto_select_region),
        ("servant.servantMaxLevelRegion", []) => reg(l.servant.servant_max_level_region),
        ("servant.servantRedirectCheckRegion", [arg]) => reg(lift(
            l.servant.servant_redirect_check_region(game_server(arg)?),
        )?),

        ("support.bondCeRegion", []) => reg(l.support.bond_ce_region),
        ("support.bottomScrollbarRegion", []) => reg(l.support.bottom_scrollbar_region),
        ("support.confirmSetupButtonRegion", []) => reg(l.support.confirm_setup_button_region),
        ("support.defaultBounds", []) => reg(l.support.default_bounds),
        ("support.defaultCeBounds", []) => reg(l.support.default_ce_bounds),
        ("support.extraRegion", []) => reg(l.support.extra_region),
        ("support.firstSupportClick", []) => loc(l.support.first_support_click),
        ("support.friendRegion", []) => reg(l.support.friend_region),
        ("support.friendsRegion", []) => reg(l.support.friends_region),
        ("support.grandCeClipRegion", []) => reg(l.support.grand_ce_clip_region),
        ("support.grandCeLabelRegion", []) => reg(l.support.grand_ce_label_region),
        ("support.grandCeListRegion", []) => reg(l.support.grand_ce_list_region),
        ("support.grandCeRegion1", []) => reg(l.support.grand_ce_region1),
        ("support.grandCeRegion2", []) => reg(l.support.grand_ce_region2),
        ("support.grandCeRegion3", []) => reg(l.support.grand_ce_region3),
        ("support.grandLimitBreakRegion", []) => reg(l.support.grand_limit_break_region),
        ("support.limitBreakRegion", []) => reg(l.support.limit_break_region),
        ("support.listRegion", []) => reg(l.support.list_region),
        ("support.listSwipeEnd", []) => loc(l.support.list_swipe_end),
        ("support.listSwipeStart", []) => loc(l.support.list_swipe_start),
        ("support.listTopClick", []) => loc(l.support.list_top_click),
        ("support.locate", [arg]) => loc(l.support.locate(support_class(arg)?)),
        ("support.maxAscendedRegion", []) => reg(l.support.max_ascended_region),
        ("support.notFoundRegion", []) => reg(l.support.not_found_region),
        ("support.refreshRegion", []) => reg(l.support.refresh_region),
        ("support.screenCheckRegion", []) => reg(l.support.screen_check_region),
        ("support.skillDisplayLabelsRegion", []) => reg(l.support.skill_display_labels_region),
        ("support.skillDisplaySwitchClick", []) => loc(l.support.skill_display_switch_click),
        ("support.topScrollbarRegion", []) => reg(l.support.top_scrollbar_region),
        ("support.updateClick", []) => loc(l.support.update_click),
        ("support.updateYesClick", []) => loc(l.support.update_yes_click),

        ("sibling.CEEnhanceLocations.autoFill", []) => reg(env.ce_enhance.auto_fill),
        ("sibling.CEEnhanceLocations.execute", []) => reg(env.ce_enhance.execute),
        ("sibling.CEEnhanceLocations.filterWarningConfirm", []) => {
            loc(env.ce_enhance.filter_warning_confirm)
        }
        ("sibling.CEEnhanceLocations.filterWarningConfirmRegion", []) => {
            reg(env.ce_enhance.filter_warning_confirm_region)
        }
        ("sibling.CEEnhanceLocations.reselect", []) => reg(env.ce_enhance.reselect),

        ("sibling.FpSellLocations.bannerTitle", []) => reg(env.fp_sell.banner_title),
        ("sibling.FpSellLocations.bottomScrollbarRegion", []) => {
            reg(env.fp_sell.bottom_scrollbar_region)
        }
        ("sibling.FpSellLocations.cardHeight", []) => one(Value::Int(FpSellLocations::CARD_HEIGHT)),
        ("sibling.FpSellLocations.cardWidth", []) => one(Value::Int(FpSellLocations::CARD_WIDTH)),
        ("sibling.FpSellLocations.close", []) => loc(env.fp_sell.close),
        ("sibling.FpSellLocations.columnX", [arg]) => {
            one(Value::Int(env.fp_sell.column_x(int(arg)?)))
        }
        ("sibling.FpSellLocations.confirm", []) => reg(env.fp_sell.confirm),
        ("sibling.FpSellLocations.convert", []) => reg(env.fp_sell.convert),
        ("sibling.FpSellLocations.dailySummonButton", []) => reg(env.fp_sell.daily_summon_button),
        ("sibling.FpSellLocations.doneClose", []) => reg(env.fp_sell.done_close),
        ("sibling.FpSellLocations.frameBands", []) => Ok(FpSellLocations::FRAME_BANDS
            .iter()
            .copied()
            .map(Value::Int)
            .collect()),
        ("sibling.FpSellLocations.frameOffsets", []) => Ok(FpSellLocations::FRAME_OFFSETS
            .iter()
            .copied()
            .map(Value::Int)
            .collect()),
        ("sibling.FpSellLocations.fullDialogLine", []) => reg(env.fp_sell.full_dialog_line),
        ("sibling.FpSellLocations.fullSell", []) => reg(env.fp_sell.full_sell),
        ("sibling.FpSellLocations.grid", []) => reg(env.fp_sell.grid),
        ("sibling.FpSellLocations.layoutCardHeight", [arg]) => {
            one(Value::Int(env.fp_sell.layout_card_height(int(arg)?)))
        }
        ("sibling.FpSellLocations.layoutColumns", [arg]) => Ok(env
            .fp_sell
            .layout_columns(int(arg)?)
            .into_iter()
            .map(Value::Int)
            .collect()),
        ("sibling.FpSellLocations.layoutPitch", [arg]) => {
            one(Value::Int(env.fp_sell.layout_pitch(int(arg)?)))
        }
        ("sibling.FpSellLocations.leftArrow", []) => loc(env.fp_sell.left_arrow),
        ("sibling.FpSellLocations.leftArrowRegion", []) => reg(env.fp_sell.left_arrow_region),
        ("sibling.FpSellLocations.menu", []) => reg(env.fp_sell.menu),
        ("sibling.FpSellLocations.rowPitch", []) => one(Value::Int(FpSellLocations::ROW_PITCH)),
        ("sibling.FpSellLocations.servantTab", []) => reg(env.fp_sell.servant_tab),
        ("sibling.FpSellLocations.servantTabClick", []) => loc(env.fp_sell.servant_tab_click),
        ("sibling.FpSellLocations.sizeSwitch", []) => loc(env.fp_sell.size_switch),
        ("sibling.FpSellLocations.summonButton", []) => reg(env.fp_sell.summon_button),
        ("sibling.FpSellLocations.summonMenu", []) => reg(env.fp_sell.summon_menu),
        ("sibling.FpSellLocations.swipeStart", []) => loc(env.fp_sell.swipe_start),

        ("sibling.QuestPanelLocations.apDigits", []) => reg(env.quest_panel.ap_digits),
        ("sibling.QuestPanelLocations.enlargedSpotName", []) => {
            reg(env.quest_panel.enlarged_spot_name)
        }
        ("sibling.QuestPanelLocations.freeLabel", []) => reg(env.quest_panel.free_label),
        ("sibling.QuestPanelLocations.freeSearchBand", []) => {
            reg(lift(env.quest_panel.free_search_band())?)
        }
        ("sibling.QuestPanelLocations.name", []) => reg(env.quest_panel.name),
        ("sibling.QuestPanelLocations.tap", [arg]) => loc(lift(env.quest_panel.tap(int(arg)?))?),

        ("sibling.SkillLevelLocations.buttonLabel", []) => reg(env.skill_level.button_label),
        ("sibling.SkillLevelLocations.currentLevel", [arg]) => {
            reg(lift(env.skill_level.current_level(level_mode(arg)?))?)
        }
        ("sibling.SkillLevelLocations.dialogCancel", [arg]) => {
            loc(env.skill_level.dialog_cancel(level_mode(arg)?))
        }
        ("sibling.SkillLevelLocations.dialogConfirm", [arg]) => {
            loc(env.skill_level.dialog_confirm(level_mode(arg)?))
        }
        ("sibling.SkillLevelLocations.dialogTitle", [arg]) => {
            reg(lift(env.skill_level.dialog_title(level_mode(arg)?))?)
        }
        ("sibling.SkillLevelLocations.dialogWatermark", []) => {
            reg(env.skill_level.dialog_watermark)
        }
        ("sibling.SkillLevelLocations.label", []) => reg(env.skill_level.label),
        ("sibling.SkillLevelLocations.nextLevel", [arg]) => {
            reg(lift(env.skill_level.next_level(level_mode(arg)?))?)
        }
        ("sibling.SkillLevelLocations.panelLevel", [mode, slot]) => reg(lift(
            env.skill_level.panel_level(level_mode(mode)?, int(slot)?),
        )?),
        ("sibling.SkillLevelLocations.reselect", []) => reg(env.skill_level.reselect),
        ("sibling.SkillLevelLocations.result", []) => reg(env.skill_level.result),
        ("sibling.SkillLevelLocations.skipClick", []) => loc(env.skill_level.skip_click),
        ("sibling.SkillLevelLocations.slotClick", [mode, slot]) => loc(lift(
            env.skill_level.slot_click(level_mode(mode)?, int(slot)?),
        )?),
        ("sibling.SkillLevelLocations.slotGlowSides", [mode, slot]) => regions(lift(
            env.skill_level
                .slot_glow_sides(level_mode(mode)?, int(slot)?),
        )?),
        ("sibling.SkillLevelLocations.slotLock", [mode, slot]) => reg(lift(
            env.skill_level.slot_lock(level_mode(mode)?, int(slot)?),
        )?),

        ("sibling.SupportNpLayout.BAND_BOTTOM_MARGIN", []) => {
            one(Value::Int(SupportNpLayout::BAND_BOTTOM_MARGIN))
        }
        ("sibling.SupportNpLayout.BAND_TOP_MARGIN", []) => {
            one(Value::Int(SupportNpLayout::BAND_TOP_MARGIN))
        }
        ("sibling.SupportNpLayout.BAND_WIDTH", []) => one(Value::Int(SupportNpLayout::BAND_WIDTH)),
        ("sibling.SupportNpLayout.BAND_X", []) => one(Value::Int(SupportNpLayout::BAND_X)),
        ("sibling.SupportNpLayout.CROP_VERTICAL_MARGIN", []) => {
            one(Value::Int(SupportNpLayout::CROP_VERTICAL_MARGIN))
        }
        ("sibling.SupportNpLayout.DIGIT_WIDTH", []) => {
            one(Value::Int(SupportNpLayout::DIGIT_WIDTH))
        }
        ("sibling.SupportNpLayout.GRAND_ICON_X", []) => {
            one(Value::Int(SupportNpLayout::GRAND_ICON_X))
        }
        ("sibling.SupportNpLayout.ICON_CROP_MARGIN", []) => {
            one(Value::Int(SupportNpLayout::ICON_CROP_MARGIN))
        }
        ("sibling.SupportNpLayout.LEVEL_CROP_GAP", []) => {
            one(Value::Int(SupportNpLayout::LEVEL_CROP_GAP))
        }
        ("sibling.SupportNpLayout.NAME_HEIGHT", []) => {
            one(Value::Int(SupportNpLayout::NAME_HEIGHT))
        }
        ("sibling.SupportNpLayout.NAME_LINE_OFFSET", []) => {
            one(Value::Int(SupportNpLayout::NAME_LINE_OFFSET))
        }
        ("sibling.SupportNpLayout.NAME_WIDTH", []) => one(Value::Int(SupportNpLayout::NAME_WIDTH)),
        ("sibling.SupportNpLayout.NAME_X", []) => one(Value::Int(SupportNpLayout::NAME_X)),
        ("sibling.SupportNpLayout.NAME_Y", []) => one(Value::Int(SupportNpLayout::NAME_Y)),
        ("sibling.SupportNpLayout.NORMAL_ICON_X", []) => {
            one(Value::Int(SupportNpLayout::NORMAL_ICON_X))
        }
        ("sibling.SupportNpLayout.SUFFIX_MARGIN", []) => {
            one(Value::Int(SupportNpLayout::SUFFIX_MARGIN))
        }
        ("sibling.SupportNpLayout.SUFFIX_RIGHT_OVERHANG", []) => {
            one(Value::Int(SupportNpLayout::SUFFIX_RIGHT_OVERHANG))
        }
        ("sibling.SupportNpLayout.TEXT_START_OFFSET", []) => {
            one(Value::Int(SupportNpLayout::TEXT_START_OFFSET))
        }

        ("sibling.SupportScoreLayout.SCORE_HEIGHT", []) => {
            one(Value::Int(SupportScoreLayout::SCORE_HEIGHT))
        }
        ("sibling.SupportScoreLayout.SCORE_WIDTH", []) => {
            one(Value::Int(SupportScoreLayout::SCORE_WIDTH))
        }
        ("sibling.SupportScoreLayout.SCORE_X", []) => one(Value::Int(SupportScoreLayout::SCORE_X)),
        ("sibling.SupportScoreLayout.SCORE_Y", []) => one(Value::Int(SupportScoreLayout::SCORE_Y)),

        ("sibling.SupportSkillLayout.APPEND_SPACING", []) => {
            one(Value::Int(SupportSkillLayout::APPEND_SPACING))
        }
        ("sibling.SupportSkillLayout.LEVEL_HEIGHT", []) => {
            one(Value::Int(SupportSkillLayout::LEVEL_HEIGHT))
        }
        ("sibling.SupportSkillLayout.LEVEL_WIDTH", []) => {
            one(Value::Int(SupportSkillLayout::LEVEL_WIDTH))
        }
        ("sibling.SupportSkillLayout.LEVEL_X", []) => one(Value::Int(SupportSkillLayout::LEVEL_X)),
        ("sibling.SupportSkillLayout.LEVEL_Y", []) => one(Value::Int(SupportSkillLayout::LEVEL_Y)),
        ("sibling.SupportSkillLayout.NORMAL_SPACING", []) => {
            one(Value::Int(SupportSkillLayout::NORMAL_SPACING))
        }

        ("sibling.WeeklyRouteLocations.bottomScrollbarRegion", []) => {
            reg(env.weekly.bottom_scrollbar_region)
        }
        ("sibling.WeeklyRouteLocations.homeOtherBanner", []) => reg(env.weekly.home_other_banner),
        ("sibling.WeeklyRouteLocations.listArea", []) => reg(env.weekly.list_area),
        ("sibling.WeeklyRouteLocations.listScrollEnd", []) => loc(env.weekly.list_scroll_end),
        ("sibling.WeeklyRouteLocations.listScrollStart", []) => loc(env.weekly.list_scroll_start),
        ("sibling.WeeklyRouteLocations.mapMotion", []) => reg(env.weekly.map_motion),
        ("sibling.WeeklyRouteLocations.panelColumn", []) => reg(env.weekly.panel_column),
        ("sibling.WeeklyRouteLocations.pinchBounds", []) => reg(env.weekly.pinch_bounds),
        ("sibling.WeeklyRouteLocations.repeatDialogClose", []) => {
            loc(env.weekly.repeat_dialog_close)
        }
        ("sibling.WeeklyRouteLocations.repeatDialogSignal", []) => {
            reg(env.weekly.repeat_dialog_signal)
        }
        ("sibling.WeeklyRouteLocations.topLeftButton", []) => reg(env.weekly.top_left_button),
        ("sibling.WeeklyRouteLocations.topScrollbarRegion", []) => {
            reg(env.weekly.top_scrollbar_region)
        }

        _ => Err(format!("{}: no Rust call is mapped", call.path)),
    }
}

fn json_int(value: &Json, at: &str) -> Result<i32, String> {
    value
        .as_i64()
        .and_then(|v| i32::try_from(v).ok())
        .ok_or_else(|| format!("{at}: not an int: {value}"))
}

fn json_field<'a>(value: &'a Json, name: &str, at: &str) -> Result<&'a Json, String> {
    value
        .get(name)
        .ok_or_else(|| format!("{at}: missing {name}"))
}

fn json_region(value: &Json, at: &str) -> Result<[i32; 4], String> {
    Ok([
        json_int(json_field(value, "x", at)?, at)?,
        json_int(json_field(value, "y", at)?, at)?,
        json_int(json_field(value, "width", at)?, at)?,
        json_int(json_field(value, "height", at)?, at)?,
    ])
}

fn json_location(value: &Json, at: &str) -> Result<[i32; 2], String> {
    Ok([
        json_int(json_field(value, "x", at)?, at)?,
        json_int(json_field(value, "y", at)?, at)?,
    ])
}

fn parts(region: Region) -> [i32; 4] {
    [region.x(), region.y(), region.width(), region.height()]
}

#[derive(Default)]
struct Counts {
    files: usize,
    entries: usize,
    nulls: usize,
    ints: usize,
    locations: usize,
    regions: usize,
}

fn compare_entry(
    at: &str,
    expected: &Json,
    actual: Value,
    transformer: &dyn Transformer,
    counts: &mut Counts,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    let mut fail = |what: &str, expected: String, actual: String| {
        failures.push(format!("{at}: {what}: expected {expected}, got {actual}"));
    };

    match actual {
        Value::Null => {
            counts.nulls += 1;
            if !expected.is_null() {
                fail("value", expected.to_string(), "null".to_owned());
            }
        }
        Value::Int(value) => {
            counts.ints += 1;
            match expected.as_i64() {
                Some(want) if want == i64::from(value) => {}
                _ => fail("int", expected.to_string(), value.to_string()),
            }
        }
        Value::Loc(location) => {
            counts.locations += 1;
            let script = json_field(expected, "script", at)?;
            let want_script = json_location(script, at)?;
            if want_script != [location.x, location.y] {
                fail(
                    "script",
                    format!("{want_script:?}"),
                    format!("{:?}", [location.x, location.y]),
                );
            }
            let want_screen = json_location(json_field(expected, "screen", at)?, at)?;
            match transformer.to_screen_location(location) {
                Ok(screen) if [screen.x, screen.y] == want_screen => {}
                Ok(screen) => fail(
                    "screen",
                    format!("{want_screen:?}"),
                    format!("{:?}", [screen.x, screen.y]),
                ),
                Err(e) => fail("screen", format!("{want_screen:?}"), e.to_string()),
            }
        }
        Value::Reg(region) => {
            counts.regions += 1;
            let script = json_field(expected, "script", at)?;
            let want_script = json_region(script, at)?;
            if want_script != parts(region) {
                fail(
                    "script",
                    format!("{want_script:?}"),
                    format!("{:?}", parts(region)),
                );
            }
            let want_screen = json_region(json_field(expected, "screen", at)?, at)?;
            match transformer.to_screen_region(region) {
                Ok(screen) if parts(screen) == want_screen => {}
                Ok(screen) => fail(
                    "screen",
                    format!("{want_screen:?}"),
                    format!("{:?}", parts(screen)),
                ),
                Err(e) => fail("screen", format!("{want_screen:?}"), e.to_string()),
            }
            let want_image = json_region(json_field(expected, "image", at)?, at)?;
            match transformer.to_image(region) {
                Ok(image) if parts(image) == want_image => {}
                Ok(image) => fail(
                    "image",
                    format!("{want_image:?}"),
                    format!("{:?}", parts(image)),
                ),
                Err(e) => fail("image", format!("{want_image:?}"), e.to_string()),
            }
        }
    }
    Ok(())
}

fn evaluate_and_compare(
    env: &Env,
    at: &str,
    key: &str,
    expected: &Json,
    transformer: &dyn Transformer,
    counts: &mut Counts,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    match evaluate(env, key) {
        Ok(actual) => compare_entry(at, expected, actual, transformer, counts, failures),
        Err(message) => {
            failures.push(format!("{at}: {message}"));
            Ok(())
        }
    }
}

fn check_file(path: &Path, counts: &mut Counts, failures: &mut Vec<String>) -> Result<(), String> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let text = fs::read_to_string(path).map_err(|e| format!("{name}: {e}"))?;
    let doc: Json = serde_json::from_str(&text).map_err(|e| format!("{name}: {e}"))?;

    let server_name = json_field(&doc, "server", &name)?
        .as_str()
        .ok_or_else(|| format!("{name}: server is not a string"))?;
    let server = GameServers::deserialize(&server_name.replace('_', " "))
        .ok_or_else(|| format!("{name}: unknown server {server_name}"))?;

    let screen = json_field(&doc, "screen", &name)?;
    let screen_size = Size::new(
        json_int(json_field(screen, "width", &name)?, &name)?,
        json_int(json_field(screen, "height", &name)?, &name)?,
    )
    .map_err(|e| format!("{name}: screen: {e}"))?;

    let manager = FgoGameAreaManager::new(screen_size, Location::default)
        .map_err(|e| format!("{name}: game area manager: {e}"))?;
    let scale = RealScale::new(&manager);
    let transformer = RealTransformer::new(&manager, &scale);
    let script_to_image = scale.script_to_image();

    let transforms: Transforms = Arc::new(
        ScriptAreaTransforms::new(&scale, &manager, server, true)
            .map_err(|e| format!("{name}: transforms: {e}"))?,
    );

    let blank =
        build_env(&transforms, Box::new(NothingOnScreen)).map_err(|e| format!("{name}: {e}"))?;

    let values = json_field(&doc, "values", &name)?
        .as_object()
        .ok_or_else(|| format!("{name}: values is not an object"))?;
    let mut entries = 0usize;
    for (key, expected) in values {
        entries += 1;
        let at = format!("{name}:{key}");
        evaluate_and_compare(&blank, &at, key, expected, &transformer, counts, failures)?;
    }
    if entries != ENTRIES_PER_FILE {
        failures.push(format!(
            "{name}: expected {ENTRIES_PER_FILE} entries, found {entries}"
        ));
    }
    counts.entries += entries;

    let outcomes = json_field(&doc, "masterOutcomes", &name)?;
    let not_found = json_field(outcomes, "buttonNotFound", &name)?;
    let found = json_field(outcomes, "buttonFound", &name)?;

    let blank_logged = blank.locations.battle.master.default_offset_logged();
    let want_blank_logged = json_field(not_found, "defaultOffsetLogged", &name)?
        .as_bool()
        .ok_or_else(|| format!("{name}: buttonNotFound.defaultOffsetLogged is not a bool"))?;
    if blank_logged != want_blank_logged {
        failures.push(format!(
            "{name}: buttonNotFound.defaultOffsetLogged: expected {want_blank_logged}, got {blank_logged}"
        ));
    }
    let not_found_values = json_field(not_found, "values", &name)?;
    for path in MASTER_PATHS {
        let at = format!("{name}:buttonNotFound:{path}");
        let expected = json_field(not_found_values, path, &at)?;
        evaluate_and_compare(&blank, &at, path, expected, &transformer, counts, failures)?;
        counts.entries += 1;
    }

    let master = &blank.locations.battle.master;
    let search = lift(master.battle_menu_search_region()).map_err(|e| format!("{name}: {e}"))?;
    let usual_center_x = lift(master.master_skill_open_click())
        .map_err(|e| format!("{name}: {e}"))?
        .x;
    let (template_width, template_height) =
        battle_menu_size(server).map_err(|e| format!("{name}: {e}"))?;
    let template = Region::new(0, 0, template_width, template_height)
        .map_err(|e| format!("{name}: template: {e}"))?;
    let template_in_script = transformer
        .image_to_script(template)
        .map_err(|e| format!("{name}: template: {e}"))?;
    let top_left = Location::new(
        usual_center_x - template_in_script.width() / 2,
        search.y() + (search.height() - template_in_script.height()) / 2,
    );

    let ratio = scale.screen_to_image().unwrap_or(1.0);
    let frame_width = round_to_int(f64::from(screen_size.width()) * ratio)
        .map_err(|e| format!("{name}: frame: {e}"))?;
    let frame_height = round_to_int(f64::from(screen_size.height()) * ratio)
        .map_err(|e| format!("{name}: frame: {e}"))?;
    let game_area = manager.game_area();
    let paste_x =
        round_to_int(f64::from(game_area.x()) * ratio + f64::from(top_left.x) * script_to_image)
            .map_err(|e| format!("{name}: paste: {e}"))?;
    let paste_y =
        round_to_int(f64::from(game_area.y()) * ratio + f64::from(top_left.y) * script_to_image)
            .map_err(|e| format!("{name}: paste: {e}"))?;
    let frame =
        Region::new(0, 0, frame_width, frame_height).map_err(|e| format!("{name}: frame: {e}"))?;
    let pasted = Region::new(paste_x, paste_y, template_width, template_height)
        .map_err(|e| format!("{name}: paste: {e}"))?;
    if !frame.contains(&pasted) {
        failures.push(format!(
            "{name}: battle_menu pasted at {pasted:?} does not fit in the frame {frame:?}"
        ));
    }
    let game_crop = (game_area * ratio)
        .map_err(|e| format!("{name}: crop: {e}"))
        .and_then(|wanted| {
            frame
                .clip(&wanted)
                .map_err(|e| format!("{name}: crop: {e}"))
        })?;
    let found_env = build_env(
        &transforms,
        Box::new(PastedTemplate {
            frame: Region::new(0, 0, game_crop.width(), game_crop.height())
                .map_err(|e| format!("{name}: crop: {e}"))?,
            pasted: pasted - game_crop.location(),
            script_to_image,
        }),
    )
    .map_err(|e| format!("{name}: {e}"))?;
    let found_values = json_field(found, "values", &name)?;
    for path in MASTER_PATHS {
        let at = format!("{name}:buttonFound:{path}");
        let expected = json_field(found_values, path, &at)?;
        evaluate_and_compare(
            &found_env,
            &at,
            path,
            expected,
            &transformer,
            counts,
            failures,
        )?;
        counts.entries += 1;
    }
    let found_logged = found_env.locations.battle.master.default_offset_logged();
    let want_found_logged = json_field(found, "defaultOffsetLogged", &name)?
        .as_bool()
        .ok_or_else(|| format!("{name}: buttonFound.defaultOffsetLogged is not a bool"))?;
    if found_logged != want_found_logged {
        failures.push(format!(
            "{name}: buttonFound.defaultOffsetLogged: expected {want_found_logged}, got {found_logged}"
        ));
    }

    counts.files += 1;
    Ok(())
}

#[test]
fn location_tables_match_the_kotlin_golden_files() {
    let files = golden_files().expect("golden files");
    let mut counts = Counts::default();
    let mut failures = Vec::new();
    for file in &files {
        if let Err(message) = check_file(file, &mut counts, &mut failures) {
            failures.push(message);
        }
    }
    println!(
        "golden files: {}, entries compared: {}, null: {}, int: {}, location: {}, region: {}",
        counts.files, counts.entries, counts.nulls, counts.ints, counts.locations, counts.regions
    );
    assert!(
        failures.is_empty(),
        "{} mismatches, first 20:\n{}",
        failures.len(),
        failures
            .iter()
            .take(20)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(files.len(), FILES, "golden file count");
    assert_eq!(counts.files, FILES, "files checked");
    assert_eq!(
        counts.entries,
        FILES * (ENTRIES_PER_FILE + 2 * MASTER_PATHS.len()),
        "entries compared"
    );
}

fn png_size(path: &Path) -> Result<(i32, i32), String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let header = bytes
        .get(..24)
        .ok_or_else(|| format!("{}: too short", path.display()))?;
    if &header[..8] != b"\x89PNG\r\n\x1a\n" || &header[12..16] != b"IHDR" {
        return Err(format!("{}: not a PNG", path.display()));
    }
    let word = |at: usize| {
        let raw = [header[at], header[at + 1], header[at + 2], header[at + 3]];
        i32::try_from(u32::from_be_bytes(raw)).map_err(|e| e.to_string())
    };
    Ok((word(16)?, word(20)?))
}

#[test]
fn battle_menu_sizes_match_the_template_files() {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fga/app/src/main/assets");
    if !assets.join("En").join("battle_menu.png").is_file() {
        println!("fga submodule not checked out: template sizes not compared with the files");
        return;
    }
    for server in GameServers::VALUES {
        let own = assets.join(server.simple()).join("battle_menu.png");
        let file = if own.is_file() {
            own
        } else {
            assets.join("En").join("battle_menu.png")
        };
        let from_file = png_size(&file).expect("png header");
        let from_table = battle_menu_size(server).expect("table size");
        assert_eq!(from_table, from_file, "{server}");
    }
}
