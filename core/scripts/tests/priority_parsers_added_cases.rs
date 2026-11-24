#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use scripts::enums::card_affinity_enum::CardAffinityEnum;
use scripts::enums::card_type_enum::CardTypeEnum;
use scripts::models::card_priority::CardPriority;
use scripts::models::card_priority_per_wave::CardPriorityPerWave;
use scripts::models::card_score::CardScore;
use scripts::models::model_error::ModelError;
use scripts::models::per_entry_settings as per_entry;
use scripts::models::servant_card_key::ServantCardKey;
use scripts::models::servant_card_priority_per_wave::ServantCardPriorityPerWave;
use scripts::models::servant_priority_per_wave::ServantPriorityPerWave;

const DEFAULT: &str = "WB, WA, WQ, B, A, Q, RB, RA, RQ";
const NINE: &str = "WB,WA,WQ,B,A,Q,RB,RA,RQ";

fn show(result: Result<String, ModelError>) -> String {
    match result {
        Ok(text) => text,
        Err(ModelError::NumberFormat(message)) => format!("ERR NumberFormat: {message}"),
        Err(ModelError::IllegalArgument(message)) => format!("ERR IllegalArgument: {message}"),
        Err(ModelError::CardPriorityParse(message)) => format!("ERR CardPriorityParse: {message}"),
        Err(ModelError::IndexOutOfBounds(_)) => "ERR IndexOutOfBounds".to_string(),
        Err(other) => format!("ERR {other:?}"),
    }
}

fn priority_error(message: &str) -> String {
    format!("ERR CardPriorityParse: Battle_CardPriority Error at '{message}")
}

fn card_priority(input: &str) -> String {
    show(CardPriority::of(input).map(|p| p.to_string()))
}

#[test]
fn card_priority_of_matches_kotlin() {
    let cases: Vec<(String, String)> = vec![
        (DEFAULT.to_string(), DEFAULT.to_string()),
        ("wb,wa,wq,b,a,q,rb,ra,rq".to_string(), DEFAULT.to_string()),
        (
            "  WB ,\tWA,WQ ,B,A,Q,RB,RA,RQ  ".to_string(),
            DEFAULT.to_string(),
        ),
        (
            "B,B,B,B,B,B,B,B,B".to_string(),
            "B, B, B, B, B, B, B, B, B".to_string(),
        ),
        (
            "XB,B,B,B,B,B,B,B,B".to_string(),
            "B, B, B, B, B, B, B, B, B".to_string(),
        ),
        ("WB ,WA,WQ,B,A,Q,RB,RA,RQ".to_string(), DEFAULT.to_string()),
        (
            "B,B,B,B,B,B,B,B".to_string(),
            priority_error("': Expected 9 cards, but 8 found."),
        ),
        (
            "B,B,B,B,B,B,B,B,B,B".to_string(),
            priority_error("': Expected 9 cards, but 10 found."),
        ),
        (String::new(), priority_error("': Invalid card length.")),
        (",".to_string(), priority_error("': Invalid card length.")),
        (
            "WB,WA,WQ,B,A,Q,RB,RA,ZQ".to_string(),
            priority_error("Z': Only 'W', and 'R' are valid card affinities."),
        ),
        (
            "WB,WA,WQ,B,A,Q,RB,RA,RX".to_string(),
            priority_error("X': Only 'B', 'A' and 'Q' are valid card types."),
        ),
        (
            "WB,WA,WQ,B,A,Q,RB,RA,R".to_string(),
            priority_error("R': Only 'B', 'A' and 'Q' are valid card types."),
        ),
        (
            "WB,WA,WQ,B,A,Q,RB,RA,WBB".to_string(),
            priority_error("WBB': Invalid card length."),
        ),
        (
            "\u{df},B,B,B,B,B,B,B,B".to_string(),
            priority_error("S': Only 'B', 'A' and 'Q' are valid card types."),
        ),
        (
            "\u{df}B,B,B,B,B,B,B,B,B".to_string(),
            priority_error("SSB': Invalid card length."),
        ),
        (
            "B\u{131},B,B,B,B,B,B,B,B".to_string(),
            priority_error("I': Only 'B', 'A' and 'Q' are valid card types."),
        ),
        (
            "WB,WA,WQ,B,A,Q,RB,RA,\u{fb00}".to_string(),
            priority_error("F': Only 'B', 'A' and 'Q' are valid card types."),
        ),
        (
            "WB,WA,WQ,B,A,Q,RB,RA,".to_string(),
            priority_error("': Invalid card length."),
        ),
        (
            "X,B,B,B,B,B,B,B,B".to_string(),
            priority_error("X': Only 'B', 'A' and 'Q' are valid card types."),
        ),
        (
            "U,B,B,B,B,B,B,B,B".to_string(),
            priority_error("U': Only 'B', 'A' and 'Q' are valid card types."),
        ),
        (
            "WU,B,B,B,B,B,B,B,B".to_string(),
            priority_error("U': Only 'B', 'A' and 'Q' are valid card types."),
        ),
        (
            "ZZ,Y,WBB,B,B,B,B,B,B".to_string(),
            priority_error("Z': Only 'B', 'A' and 'Q' are valid card types."),
        ),
        (
            "Y,ZZ,WBB,B,B,B,B,B,B".to_string(),
            priority_error("Y': Only 'B', 'A' and 'Q' are valid card types."),
        ),
        (
            "WB,WA,WQ\nB,A,Q,RB,RA,RQ".to_string(),
            priority_error("WQ\nB': Invalid card length."),
        ),
        (
            "WB,WA,WQ,B,A,Q,RB,RA,R\u{e9}".to_string(),
            priority_error("\u{c9}': Only 'B', 'A' and 'Q' are valid card types."),
        ),
        (
            "WB\u{85},WA,WQ,B,A,Q,RB,RA,RQ".to_string(),
            priority_error("WB\u{85}': Invalid card length."),
        ),
        (
            "WB\u{180e},WA,WQ,B,A,Q,RB,RA,RQ".to_string(),
            priority_error("WB\u{180e}': Invalid card length."),
        ),
        (
            "WB\u{1c},WA,WQ,B,A,Q,RB,RA,RQ".to_string(),
            DEFAULT.to_string(),
        ),
        (
            "WB\u{1f},WA,WQ,B,A,Q,RB,RA,RQ".to_string(),
            DEFAULT.to_string(),
        ),
        (
            "WB\u{a0},WA,WQ,B,A,Q,RB,RA,RQ".to_string(),
            DEFAULT.to_string(),
        ),
        (
            "WB\u{2028},WA,WQ,B,A,Q,RB,RA,RQ".to_string(),
            DEFAULT.to_string(),
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(card_priority(&input), expected, "{input:?}");
    }
    assert!(matches!(
        CardPriority::of("WB,WA,WQ,B,A,Q,RB,RA,\u{1F600}"),
        Err(ModelError::CardPriorityParse(_))
    ));
}

fn per_wave(input: &str) -> String {
    show(CardPriorityPerWave::of(input).and_then(|p| {
        let at = |wave: i32| -> Result<String, ModelError> {
            let scores: Vec<String> = p.at_wave(wave)?.iter().map(ToString::to_string).collect();
            Ok(format!("[{}]", scores.join(", ")))
        };
        Ok(format!(
            "str={} size={} w-5={} w0={} w1={} w7={}",
            p.to_string().replace('\n', "\\n"),
            p.len(),
            at(-5)?,
            at(0)?,
            at(1)?,
            at(7)?
        ))
    }))
}

#[test]
fn card_priority_per_wave_of_matches_kotlin() {
    let default_all = format!("[{DEFAULT}, U]");
    let entry1 = "Q, WQ, RQ, A, WA, RA, B, WB, RB";
    let default_case = format!(
        "str={DEFAULT} size=1 w-5={default_all} w0={default_all} w1={default_all} w7={default_all}"
    );
    for blank in ["", "   ", "\n", "\u{a0}", "\u{1c}"] {
        assert_eq!(per_wave(blank), default_case, "{blank:?}");
    }
    assert_eq!(per_wave(DEFAULT), default_case);
    let two = format!(
        "str={DEFAULT}\\n{entry1} size=2 w-5={default_all} w0={default_all} w1=[{entry1}, U] w7=[{entry1}, U]"
    );
    assert_eq!(per_wave(&format!("{DEFAULT}\n{entry1}")), two);
    assert_eq!(per_wave(&format!("{DEFAULT}\r\n{entry1}")), two);
    let all_b = "[B, B, B, B, B, B, B, B, B, U]";
    assert_eq!(
        per_wave("b,b,b,b,b,b,b,b,b"),
        format!(
            "str=B, B, B, B, B, B, B, B, B size=1 w-5={all_b} w0={all_b} w1={all_b} w7={all_b}"
        )
    );
    assert_eq!(
        per_wave(&format!("{DEFAULT}\n")),
        priority_error("': Invalid card length.")
    );
    assert_eq!(
        per_wave(&format!("{DEFAULT}\nWB")),
        priority_error("': Expected 9 cards, but 1 found.")
    );
    assert_eq!(
        per_wave(&format!("\n{DEFAULT}")),
        priority_error("': Invalid card length.")
    );
    assert_eq!(
        per_wave(&format!("{DEFAULT}\u{85}")),
        priority_error("RQ\u{85}': Invalid card length.")
    );
    assert_eq!(
        per_wave("\u{85}"),
        priority_error("\u{85}': Only 'B', 'A' and 'Q' are valid card types.")
    );
    assert_eq!(
        per_wave("\u{180e}"),
        priority_error("\u{180e}': Only 'B', 'A' and 'Q' are valid card types.")
    );
}

#[test]
fn card_priority_per_wave_default_and_empty_wave_lookup() {
    assert_eq!(CardPriorityPerWave::default().to_string(), DEFAULT);
    assert_eq!(
        CardPriorityPerWave::of(DEFAULT).unwrap(),
        CardPriorityPerWave::default()
    );
    let empty = CardPriorityPerWave::from(Vec::new());
    assert!(matches!(
        empty.at_wave(0),
        Err(ModelError::IllegalArgument(_))
    ));
}

fn servant_priority(input: &str) -> String {
    show(ServantPriorityPerWave::of(input).and_then(|p| {
        let last: Vec<String> = p.at_wave(9)?.iter().map(ToString::to_string).collect();
        Ok(format!(
            "str={} size={} w9={}",
            p.to_string().replace('\n', "\\n"),
            p.len(),
            last.join(",")
        ))
    }))
}

#[test]
fn servant_priority_per_wave_of_matches_kotlin() {
    let full = "str=1,2,3,4,5,6 size=1 w9=[1],[2],[3],[4],[5],[6]";
    let cases = [
        ("1,2,3,4,5,6", full),
        ("3,2,1", "str=3,2,1 size=1 w9=[3],[2],[1]"),
        (
            "3,2,1,4,5,6\n1,2,3,4,5,6",
            "str=3,2,1,4,5,6\\n1,2,3,4,5,6 size=2 w9=[1],[2],[3],[4],[5],[6]",
        ),
        (" 3,2", "ERR NumberFormat: For input string: \" 3\""),
        ("1, 2", "ERR NumberFormat: For input string: \" 2\""),
        ("7", "ERR IndexOutOfBounds"),
        ("0", "ERR IndexOutOfBounds"),
        ("-1", "ERR IndexOutOfBounds"),
        ("a", "ERR NumberFormat: For input string: \"a\""),
        ("+3", "str=3 size=1 w9=[3]"),
        ("\u{663},\u{662},\u{661}", "str=3,2,1 size=1 w9=[3],[2],[1]"),
        ("", full),
        (" ", full),
        ("1,,2", "ERR NumberFormat: For input string: \"\""),
        ("1,2,", "ERR NumberFormat: For input string: \"\""),
        (
            "99999999999",
            "ERR NumberFormat: For input string: \"99999999999\"",
        ),
        ("-2147483648", "ERR IndexOutOfBounds"),
        ("1\n", "ERR NumberFormat: For input string: \"\""),
        ("\n1", "ERR NumberFormat: For input string: \"\""),
        ("\u{ff11}", "str=1 size=1 w9=[1]"),
        ("1\r\n2", "ERR NumberFormat: For input string: \"1\r\""),
        ("1\r", "ERR NumberFormat: For input string: \"1\r\""),
        ("\u{a0}1", "ERR NumberFormat: For input string: \"\u{a0}1\""),
        ("1\u{a0}", "ERR NumberFormat: For input string: \"1\u{a0}\""),
        ("\u{1c}", full),
        ("\u{85}", "ERR NumberFormat: For input string: \"\u{85}\""),
    ];
    for (input, expected) in cases {
        assert_eq!(servant_priority(input), expected, "{input:?}");
    }
    assert_eq!(ServantPriorityPerWave::default().to_string(), "1,2,3,4,5,6");
    let empty = ServantPriorityPerWave::from(Vec::new());
    assert!(matches!(
        empty.at_wave(0),
        Err(ModelError::IllegalArgument(_))
    ));
}

fn servant_card_priority(input: &str) -> String {
    show(ServantCardPriorityPerWave::of(input).map(|p| {
        format!(
            "str={} size={}",
            p.to_string().replace('\n', "\\n"),
            p.len()
        )
    }))
}

#[test]
fn servant_card_priority_per_wave_of_matches_kotlin() {
    let nine = "1B,1A,1Q,2B,2A,2Q,3B,3A,3Q";
    let one = format!("str={nine} size=1");
    let cases = [
        (String::new(), one.clone()),
        (nine.to_string(), one.clone()),
        ("1b,1a,1q,2b,2a,2q,3b,3a,3q".to_string(), one.clone()),
        ("1b, 1a ,1q,2b,2a,2q,3b,3a,3q".to_string(), one.clone()),
        ("\u{ff11}B,1A,1Q,2B,2A,2Q,3B,3A,3Q".to_string(), one.clone()),
        (
            "1B,1A,1Q,2B,2A,2Q,3B,3A".to_string(),
            "ERR IllegalArgument: Each entry needs the nine distinct cards".to_string(),
        ),
        (
            "1B,1B,1Q,2B,2A,2Q,3B,3A,3Q".to_string(),
            "ERR IllegalArgument: Each entry needs the nine distinct cards".to_string(),
        ),
        (
            "4B,1A,1Q,2B,2A,2Q,3B,3A,3Q".to_string(),
            "ERR IllegalArgument: Invalid card '4B'".to_string(),
        ),
        (
            "1U,1A,1Q,2B,2A,2Q,3B,3A,3Q".to_string(),
            "ERR IllegalArgument: Invalid card '1U'".to_string(),
        ),
        (
            format!("{nine}\r\n3Q,3A,3B,2Q,2A,2B,1Q,1A,1B"),
            format!("str={nine}\\n3Q,3A,3B,2Q,2A,2B,1Q,1A,1B size=2"),
        ),
        (
            format!("{nine}\n"),
            "ERR IllegalArgument: Invalid card ''".to_string(),
        ),
        (" ".to_string(), one.clone()),
        (
            format!("{nine},"),
            "ERR IllegalArgument: Invalid card ''".to_string(),
        ),
        (
            format!("{nine},1B"),
            "ERR IllegalArgument: Each entry needs the nine distinct cards".to_string(),
        ),
        (
            "\u{85}".to_string(),
            "ERR IllegalArgument: Invalid card '\u{85}'".to_string(),
        ),
        ("\u{1c}".to_string(), one.clone()),
        (
            format!("{nine}\u{85}"),
            "ERR IllegalArgument: Invalid card '3Q\u{85}'".to_string(),
        ),
        (format!("{nine}\u{a0}"), one.clone()),
    ];
    for (input, expected) in cases {
        assert_eq!(servant_card_priority(&input), expected, "{input:?}");
    }
}

#[test]
fn servant_card_key_parse_matches_kotlin() {
    let key = |text: &str| ServantCardKey::parse(text).map(|k| k.to_string());
    let some = |s: &str| Some(s.to_string());
    assert_eq!(key(" 1b "), some("1B"));
    assert_eq!(key("1B"), some("1B"));
    assert_eq!(key("1b2"), None);
    assert_eq!(key("4B"), None);
    assert_eq!(key("0B"), None);
    assert_eq!(key("B1"), None);
    assert_eq!(key("1U"), None);
    assert_eq!(key("\u{ff11}\u{ff22}"), None);
    assert_eq!(key("\u{662}A"), some("2A"));
    assert_eq!(key("1q"), some("1Q"));
    assert_eq!(key(""), None);
    assert_eq!(key("1"), None);
    assert_eq!(key("\u{df}"), None);
    assert_eq!(key("1\u{1F600}"), None);
    assert_eq!(key("2\u{a0}"), None);
    assert_eq!(key("1\u{1c}"), None);
    assert_eq!(key("1B\u{85}"), None);
    assert_eq!(key("\u{ff11}b"), some("1B"));
    assert_eq!(key("\u{ff11}B"), some("1B"));
    assert_eq!(key("\u{662}b"), some("2B"));
    assert_eq!(key("\u{663}q"), some("3Q"));
    assert_eq!(key("\u{664}q"), None);
    assert_eq!(key("\u{660}q"), None);
    assert_eq!(key("\u{661}\u{ff22}"), None);
}

#[test]
fn per_entry_settings_parse_matches_kotlin() {
    let styles = |text: &str| {
        let parsed: Vec<&str> = per_entry::parse_styles(text)
            .iter()
            .map(|s| s.name())
            .collect();
        parsed.join(",")
    };
    let flags = |text: &str| {
        let parsed: Vec<String> = per_entry::parse_flags(text)
            .iter()
            .map(ToString::to_string)
            .collect();
        parsed.join(",")
    };
    assert_eq!(styles("Affinity,Servant"), "Affinity,Servant");
    assert_eq!(styles(" Servant , Affinity "), "Servant,Affinity");
    assert_eq!(styles("servant"), "Affinity");
    assert_eq!(styles("Servant,,Affinity"), "Servant,Affinity,Affinity");
    assert_eq!(styles("  "), "");
    assert_eq!(styles("Servant\u{a0}"), "Servant");
    assert_eq!(styles("X"), "Affinity");
    assert_eq!(styles(""), "");
    assert_eq!(flags("true,false"), "true,false");
    assert_eq!(
        flags(" TRUE , False,yes,1,"),
        "true,false,false,false,false"
    );
    assert_eq!(flags("tRuE"), "true");
    assert_eq!(flags(""), "");
    assert_eq!(flags(" "), "");
    assert_eq!(flags("\u{212A}rue"), "false");
    assert_eq!(flags("\u{131}rue"), "false");
    assert_eq!(flags("\u{17f}rue"), "false");
    assert_eq!(flags("TRUE"), "true");
    assert_eq!(flags("tRUE"), "true");
}

#[test]
fn per_entry_settings_trim_treats_the_same_characters_as_whitespace_as_kotlin() {
    let whitespace_true = [
        0x09, 0x0b, 0x0c, 0x1c, 0x1d, 0x1e, 0x1f, 0xa0, 0x1680, 0x2000, 0x2005, 0x200a, 0x2028,
        0x2029, 0x202f, 0x205f, 0x3000,
    ];
    let not_whitespace = [0x08, 0x1b, 0x85, 0x180e, 0x200b, 0x200c, 0x2060, 0xfeff];
    for code in whitespace_true {
        let text = format!("true{}", char::from_u32(code).unwrap());
        assert_eq!(per_entry::parse_flags(&text), vec![true], "{code:#x}");
        let text = format!("Servant{}", char::from_u32(code).unwrap());
        assert_eq!(per_entry::parse_styles(&text).len(), 1);
        assert_eq!(
            per_entry::parse_styles(&text)[0].name(),
            "Servant",
            "{code:#x}"
        );
    }
    for code in not_whitespace {
        let text = format!("true{}", char::from_u32(code).unwrap());
        assert_eq!(per_entry::parse_flags(&text), vec![false], "{code:#x}");
        let text = format!("Servant{}", char::from_u32(code).unwrap());
        assert_eq!(
            per_entry::parse_styles(&text)[0].name(),
            "Affinity",
            "{code:#x}"
        );
    }
}

#[test]
fn card_score_display_matches_kotlin() {
    let expected = [
        (CardTypeEnum::Buster, CardAffinityEnum::Weak, "WB"),
        (CardTypeEnum::Arts, CardAffinityEnum::Weak, "WA"),
        (CardTypeEnum::Quick, CardAffinityEnum::Weak, "WQ"),
        (CardTypeEnum::Buster, CardAffinityEnum::Normal, "B"),
        (CardTypeEnum::Arts, CardAffinityEnum::Normal, "A"),
        (CardTypeEnum::Quick, CardAffinityEnum::Normal, "Q"),
        (CardTypeEnum::Buster, CardAffinityEnum::Resist, "RB"),
        (CardTypeEnum::Arts, CardAffinityEnum::Resist, "RA"),
        (CardTypeEnum::Quick, CardAffinityEnum::Resist, "RQ"),
        (CardTypeEnum::Unknown, CardAffinityEnum::Normal, "U"),
        (CardTypeEnum::Unknown, CardAffinityEnum::Weak, "WU"),
        (CardTypeEnum::Unknown, CardAffinityEnum::Resist, "RU"),
    ];
    for (card_type, affinity, text) in expected {
        assert_eq!(CardScore::new(card_type, affinity).to_string(), text);
    }
}

#[test]
fn per_entry_at_clamps_the_index_like_kotlin() {
    assert_eq!(per_entry::at(&[1, 2, 3], -4, -1), 1);
    assert_eq!(per_entry::at(&[1, 2, 3], 0, -1), 1);
    assert_eq!(per_entry::at(&[1, 2, 3], 2, -1), 3);
    assert_eq!(per_entry::at(&[1, 2, 3], 9, -1), 3);
    assert_eq!(per_entry::at(&[], 3, -1), -1);
}

#[test]
fn nine_is_the_default_text_of_the_stored_priority() {
    assert_eq!(CardPriority::of(NINE).unwrap().to_string(), DEFAULT);
}
