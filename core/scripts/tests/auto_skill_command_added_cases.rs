#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use scripts::models::direct_card_sequence::DirectCardSequence;
use scripts::models::model_error::ModelError;

enum Expect {
    Ok(&'static str),
    Parse(&'static str),
    Raw(&'static str),
}

use Expect::{Ok as Rendered, Parse, Raw};

fn check(input: &str, expect: &Expect) {
    match (expect, parse(input)) {
        (Rendered(expected), Ok(command)) => assert_eq!(render(&command), *expected, "{input:?}"),
        (Parse(fragment), Err(ModelError::SkillCommandParse(message))) => {
            assert!(message.contains(fragment), "{input:?}: {message}");
        }
        (Raw(fragment), Err(ModelError::IllegalArgument(message))) => {
            assert!(message.contains(fragment), "{input:?}: {message}");
        }
        (_, other) => panic!("{input:?}: unexpected {other:?}"),
    }
}

#[test]
fn shuffle_token_branches_match_kotlin() {
    let cases = [
        ("aH", Parse("Expected { after shuffle H")),
        ("aH{", Parse("Unclosed shuffle token")),
        ("aH{8:13}", Rendered("SU(a,0,8,[13],false)")),
        ("aH{8:13", Parse("Unclosed shuffle token")),
        ("aH{x}", Parse("Invalid shuffle token: x")),
        ("aH{8:1b3}", Rendered("SU(a,0,8,[1b3],false)")),
        ("aH{ 8:13}", Parse("Invalid shuffle token:  8:13")),
        ("aH{1-30:13}", Rendered("SU(a,1,30,[13],false)")),
        ("aH{0-0:13}", Parse("")),
        ("aH{00008:13}", Rendered("SU(a,0,8,[13],false)")),
        (
            "aH{2147483648:13}",
            Parse("For input string: \"2147483648\""),
        ),
        ("aH{8:13|1b2!}", Rendered("SU(a,0,8,[13,1b2],true)")),
        ("aH{8:13}!", Parse("Unknown character: !")),
        ("bH{8:13}", Parse("")),
        ("a{8:13}", Parse("Shuffle requires H before {")),
        ("aH{8:13}b", Rendered("SU(a,0,8,[13],false);S(b,[])")),
        (
            "aH{8:13}aH{8:13}",
            Rendered("SU(a,0,8,[13],false);SU(a,0,8,[13],false)"),
        ),
        (
            "aH{8:13}4",
            Rendered("SU(a,0,8,[13],false);Atk([4],0,null,4)"),
        ),
        ("aH{3-2:13}", Parse("")),
        ("aH{31:13}", Parse("")),
        ("aH{8:13||22}", Parse("Invalid shuffle token: 8:13||22")),
    ];
    for (input, expect) in &cases {
        check(input, expect);
    }
}

#[test]
fn servant_skill_target_branches_match_kotlin() {
    let cases = [
        ("a", Rendered("S(a,[])")),
        ("a1", Rendered("S(a,[1])")),
        ("a2", Rendered("S(a,[2])")),
        ("a3", Rendered("S(a,[3])")),
        ("a7", Rendered("S(a,[7])")),
        ("a8", Rendered("S(a,[8])")),
        ("aK", Rendered("S(a,[K])")),
        ("aU", Rendered("S(a,[U])")),
        ("aM", Rendered("S(a,[M])")),
        ("a4", Rendered("S(a,[]);Atk([4],0,null,4)")),
        ("a9", Parse("Unknown character: 9")),
        ("a[Ch2A]", Rendered("S(a,[Ch2A])")),
        ("a[Ch3C]", Rendered("S(a,[Ch3C])")),
        ("a[Ch2]", Parse("Special target \"Ch2\" not found")),
        (
            "a[Ch2A",
            Parse("Found [ but no matching ] in Skill Command"),
        ),
        ("a[]", Parse("Command Can't be empty")),
        ("a[Zzz]", Parse("Special target \"Zzz\" not found")),
        ("a[Ch2AB]", Parse("Special target \"Ch2AB\" not found")),
        ("a[Ch2A]b", Rendered("S(a,[Ch2A]);S(b,[])")),
        ("a(12)", Rendered("S(a,[1,2])")),
        ("a(1[Ch2A]2)", Rendered("S(a,[1,Ch2A,2])")),
        ("a(1", Parse("Found ( but no matching ) in Skill Command")),
        ("a()", Rendered("S(a,[])")),
        ("a(1x2)", Rendered("S(a,[1,2])")),
        ("a([])", Parse("Command Can't be empty")),
        ("a([Foo])", Parse("Special target \"Foo\" not found")),
        (
            "a([Ch2A)",
            Parse("Found [ but no matching ] in Skill Command"),
        ),
        ("a(([Ch2A]))", Parse("Unknown character: )")),
        (
            "a(1[Ch2A]",
            Parse("Found ( but no matching ) in Skill Command"),
        ),
        ("a(K)", Rendered("S(a,[K])")),
        ("a(78)", Rendered("S(a,[7,8])")),
        ("a(1[Ch2A][Ch2B])", Rendered("S(a,[1,Ch2A,Ch2B])")),
        ("a(1)(2)", Parse("Unknown character: (")),
        ("a[Ch2A]]", Parse("Unknown character: ]")),
        ("a[[Ch2A]", Rendered("S(a,[Ch2A])")),
        ("a[C[h]2A]", Parse("Special target \"Ch\" not found")),
        ("aK1", Parse("Unknown character: 1")),
        ("a(12\u{1F600})", Rendered("S(a,[1,2])")),
        ("a(\u{FF11})", Rendered("S(a,[])")),
        ("a\u{FF11}", Parse("Unknown character: \u{FF11}")),
        (
            "a1b2c3d4e5f6g7h8i",
            Rendered(
                "S(a,[1]);S(b,[2]);S(c,[3]);S(d,[]);Atk([4],0,null,4);S(e,[]);Atk([5],0,null,5);S(f,[]);Atk([6],0,null,6);S(g,[7]);S(h,[8]);S(i,[])",
            ),
        ),
        (
            "a(1[Ch2A]2[Ch3B])j1k2l3o4",
            Rendered("S(a,[1,Ch2A,2,Ch3B]);M(j,1);M(k,2);M(l,3);C(o,null);Atk([4],0,null,4)"),
        ),
    ];
    for (input, expect) in &cases {
        check(input, expect);
    }
}

#[test]
fn master_skill_and_command_spell_branches_match_kotlin() {
    let cases = [
        ("j", Rendered("M(j,null)")),
        ("j1", Rendered("M(j,1)")),
        ("j[Ch2A]", Rendered("M(j,Ch2A)")),
        ("j[", Parse("Found [ but no matching ] in Skill Command")),
        ("k7", Rendered("M(k,7)")),
        ("l3", Rendered("M(l,3)")),
        ("l[Ch3A]", Rendered("M(l,Ch3A)")),
        ("jklo", Rendered("M(j,null);M(k,null);M(l,null);C(o,null)")),
        ("o", Rendered("C(o,null)")),
        ("o1", Rendered("C(o,1)")),
        ("o[Ch3B]", Rendered("C(o,Ch3B)")),
        ("oo", Rendered("C(o,null);C(o,null)")),
    ];
    for (input, expect) in &cases {
        check(input, expect);
    }
}

#[test]
fn np_and_cards_before_np_branches_match_kotlin() {
    let cases = [
        ("4", Rendered("Atk([4],0,null,4)")),
        ("5", Rendered("Atk([5],0,null,5)")),
        ("6", Rendered("Atk([6],0,null,6)")),
        ("45", Rendered("Atk([4,5],0,null,45)")),
        ("54", Rendered("Atk([5,4],0,null,54)")),
        ("456", Rendered("Atk([4,5,6],0,null,456)")),
        ("654", Rendered("Atk([6,5,4],0,null,654)")),
        ("446", Rendered("Atk([4,6],0,null,46)")),
        ("4n1", Rendered("Atk([4],1,null,n14)")),
        ("n14", Rendered("Atk([4],1,null,n14)")),
        ("n1", Rendered("Atk([],1,null,n1)")),
        ("n2", Rendered("Atk([],2,null,n2)")),
        ("n3", Parse("Only 0, 1 or 2 cards can be used before NP")),
        ("n0", Rendered("Atk([],0,null,0)")),
        ("n", Parse("")),
        ("nx", Parse("For input string: \"x\"")),
        ("n+", Parse("For input string: \"+\"")),
        ("n-", Parse("For input string: \"-\"")),
        ("n\u{661}", Rendered("Atk([],1,null,n1)")),
        ("n\u{662}", Rendered("Atk([],2,null,n2)")),
        (
            "n\u{663}",
            Parse("Only 0, 1 or 2 cards can be used before NP"),
        ),
        ("n\u{FF12}", Rendered("Atk([],2,null,n2)")),
        ("n\u{B2}", Parse("For input string: \"\u{B2}\"")),
        ("n12", Parse("Unknown character: 2")),
        ("n1n1", Rendered("Atk([],2,null,n2)")),
        ("n1n1n1", Raw("Only 0, 1 or 2 cards can be used before NP")),
        ("n2n2", Raw("Only 0, 1 or 2 cards can be used before NP")),
        ("n1n2", Raw("Only 0, 1 or 2 cards can be used before NP")),
        ("04", Rendered("Atk([4],0,null,4)")),
        ("40", Rendered("Atk([4],0,null,4)")),
        ("0", Rendered("Atk([],0,null,0)")),
        ("00", Rendered("Atk([],0,null,0)")),
        ("0n1", Rendered("Atk([],1,null,n1)")),
        ("n10", Rendered("Atk([],1,null,n1)")),
        ("n04", Rendered("Atk([4],0,null,4)")),
    ];
    for (input, expect) in &cases {
        check(input, expect);
    }
}

#[test]
fn enemy_target_and_order_change_branches_match_kotlin() {
    let cases = [
        ("t", Parse("")),
        ("t1", Rendered("T(1)")),
        ("t4", Rendered("T(4)")),
        ("t9", Rendered("T(9)")),
        ("tR", Rendered("T(R)")),
        ("tA", Rendered("T(A)")),
        ("tF", Rendered("T(F)")),
        ("tG", Parse("Unknown enemy target: tG")),
        ("t10", Rendered("T(A)")),
        ("t11", Rendered("T(B)")),
        ("t12", Rendered("T(C)")),
        ("t13", Parse("Unknown character: 3")),
        ("t14", Rendered("T(1);Atk([4],0,null,4)")),
        ("t15", Rendered("T(1);Atk([5],0,null,5)")),
        ("t1a", Rendered("T(1);S(a,[])")),
        ("t100", Rendered("T(A);Atk([],0,null,0)")),
        ("t0", Parse("Unknown enemy target: t0")),
        ("tr", Parse("Unknown enemy target: tr")),
        ("t\u{FF11}", Parse("Unknown enemy target: t\u{FF11}")),
        ("x", Parse("")),
        ("x1", Parse("")),
        ("x12", Rendered("O(1,2)")),
        ("x13", Rendered("O(1,3)")),
        ("x31", Rendered("O(3,1)")),
        ("x11", Rendered("O(1,1)")),
        ("x4", Parse("")),
        ("x41", Parse("")),
        ("x1a", Parse("")),
        ("xa", Parse("")),
        ("x12a", Rendered("O(1,2);S(a,[])")),
        ("x\u{FF11}\u{FF12}", Parse("")),
    ];
    for (input, expect) in &cases {
        check(input, expect);
    }
}

#[test]
fn direct_attack_branches_match_kotlin() {
    let cases = [
        ("&", Parse("")),
        ("&b", Parse("")),
        ("&ba", Parse("")),
        ("&ba4", Rendered("Atk([4],2,&ba4,&ba4)")),
        (
            "&ba4a",
            Parse("Direct attack must be the last command in the turn"),
        ),
        (
            "&ba4,&bba",
            Rendered("Atk([4],2,&ba4,&ba4)|Atk([],0,&bba,&bba)"),
        ),
        ("&?ba4", Rendered("Atk([4],2,&?ba4,&?ba4)")),
        ("&?", Parse("")),
        (
            "&??ba4",
            Parse("Direct attack must be the last command in the turn"),
        ),
        (
            "&b?4",
            Parse("Direct attack requires exactly three choices: b, a, q, p, 4, 5 or 6"),
        ),
        ("&BA4", Rendered("Atk([4],2,&ba4,&ba4)")),
        ("&Ba4", Rendered("Atk([4],2,&ba4,&ba4)")),
        (
            "&b4a4",
            Parse("Direct attack must be the last command in the turn"),
        ),
        ("&4a4", Parse("The same NP cannot be selected twice")),
        (
            "&pba",
            Parse("Unselected cards require the priority fallback"),
        ),
        ("&?pba", Rendered("Atk([],0,&?pba,&?pba)")),
        ("&?ppp", Rendered("Atk([],0,&?ppp,&?ppp)")),
        ("&?4pp", Rendered("Atk([4],0,&?4pp,&?4pp)")),
        ("&4", Parse("")),
        ("&456", Rendered("Atk([4,5,6],0,&456,&456)")),
        ("&?456", Rendered("Atk([4,5,6],0,&?456,&?456)")),
        (
            "&ba4&bba",
            Parse("Direct attack must be the last command in the turn"),
        ),
        (
            "4&ba4",
            Parse("Direct and automatic attacks cannot share a turn"),
        ),
        ("0&ba4", Rendered("Atk([4],2,&ba4,&ba4)")),
        (
            "n1&ba4",
            Parse("Direct and automatic attacks cannot share a turn"),
        ),
        ("n0&ba4", Rendered("Atk([4],2,&ba4,&ba4)")),
        ("0a&ba4", Rendered("S(a,[]);Atk([4],2,&ba4,&ba4)")),
        ("a0&ba4", Rendered("S(a,[]);Atk([4],2,&ba4,&ba4)")),
        ("00&ba4", Rendered("Atk([4],2,&ba4,&ba4)")),
        ("a&ba4", Rendered("S(a,[]);Atk([4],2,&ba4,&ba4)")),
        (
            "&ba4,0,#,&?456",
            Rendered("Atk([4],2,&ba4,&ba4)|Atk([],0,null,0)/Atk([4,5,6],0,&?456,&?456)"),
        ),
    ];
    for (input, expect) in &cases {
        check(input, expect);
    }
}

#[test]
fn wave_and_turn_separators_match_kotlin() {
    let cases = [
        ("", Rendered("")),
        ("a,b", Rendered("S(a,[])|S(b,[])")),
        ("a,#,b", Rendered("S(a,[])/S(b,[])")),
        ("a,#,b,c,#,d", Rendered("S(a,[])/S(b,[])|S(c,[])/S(d,[])")),
        ("a,,b", Rendered("S(a,[])||S(b,[])")),
        (",", Rendered("|")),
        (",#,", Rendered("/")),
        ("a,#,", Rendered("S(a,[])/")),
        ("a,#", Parse("Unknown character: #")),
        (",a", Rendered("|S(a,[])")),
        (",,", Rendered("||")),
        (",#,,", Rendered("/|")),
        ("a,#,#,b", Parse("Unknown character: #")),
        ("a,b,#,c", Rendered("S(a,[])|S(b,[])/S(c,[])")),
        ("0,0", Rendered("Atk([],0,null,0)|Atk([],0,null,0)")),
        (" a", Parse("Unknown character:  ")),
        ("z", Parse("Unknown character: z")),
        ("H", Parse("Unknown character: H")),
        ("a\n", Parse("Unknown character: \n")),
        ("a\tb", Parse("Unknown character: \t")),
        ("\u{e9}", Parse("Unknown character: \u{e9}")),
        ("\u{FF41}", Parse("Unknown character: \u{FF41}")),
        ("a\u{1F600}", Parse("Unknown character")),
    ];
    for (input, expect) in &cases {
        check(input, expect);
    }
}

#[test]
fn command_lookup_outside_the_parsed_range_matches_kotlin() {
    let command = parse_ok("a,b,#,c");
    let rendered = |stage: i32, turn: i32| {
        command.get(stage, turn).map(|actions| {
            actions
                .iter()
                .map(render_action)
                .collect::<Vec<_>>()
                .join(";")
        })
    };
    assert_eq!(rendered(0, 0), Ok("S(a,[])".to_string()));
    assert_eq!(rendered(0, 1), Ok("S(b,[])".to_string()));
    assert_eq!(rendered(0, 2), Ok(String::new()));
    assert_eq!(rendered(1, 0), Ok("S(c,[])".to_string()));
    assert_eq!(rendered(2, 0), Ok(String::new()));
    assert!(matches!(
        rendered(-1, 0),
        Err(ModelError::IndexOutOfBounds(_))
    ));
    assert!(matches!(
        rendered(0, -1),
        Err(ModelError::IndexOutOfBounds(_))
    ));
    assert_eq!(rendered(5, -1), Ok(String::new()));
    assert!(matches!(
        rendered(-1, 5),
        Err(ModelError::IndexOutOfBounds(_))
    ));
}

#[test]
fn direct_card_sequence_parse_matches_kotlin() {
    let described = |value: &str| {
        DirectCardSequence::parse(value).map(|d| {
            let nps: Vec<String> = d
                .nps()
                .iter()
                .map(|np| np.auto_skill_code().to_string())
                .collect();
            format!(
                "code={} fb={} str={} nps={} before={}",
                d.code(),
                d.fallback(),
                d,
                nps.join(","),
                d.cards_before_np()
            )
        })
    };
    let ok = [
        ("bba", "code=bba fb=false str=&bba nps= before=0"),
        ("?bba", "code=bba fb=true str=&?bba nps= before=0"),
        ("BBA", "code=bba fb=false str=&bba nps= before=0"),
        ("?BBA", "code=bba fb=true str=&?bba nps= before=0"),
        ("?bpa", "code=bpa fb=true str=&?bpa nps= before=0"),
        ("?ppp", "code=ppp fb=true str=&?ppp nps= before=0"),
        ("?456", "code=456 fb=true str=&?456 nps=4,5,6 before=0"),
        ("456", "code=456 fb=false str=&456 nps=4,5,6 before=0"),
        ("?ba4", "code=ba4 fb=true str=&?ba4 nps=4 before=2"),
        ("?64a", "code=64a fb=true str=&?64a nps=6,4 before=0"),
        ("?4ba", "code=4ba fb=true str=&?4ba nps=4 before=0"),
        ("?b6a", "code=b6a fb=true str=&?b6a nps=6 before=1"),
    ];
    for (value, expected) in ok {
        assert_eq!(described(value), Ok(expected.to_string()), "{value:?}");
    }
    for value in [
        "??bba",
        "bb",
        "bbbb",
        "bpa",
        "?4p4",
        "?44a",
        "?45",
        "4 5",
        " ba4",
        "b\u{1F600}",
        "bxa",
        "",
        "?",
    ] {
        assert!(
            matches!(
                DirectCardSequence::parse(value),
                Err(ModelError::IllegalArgument(_))
            ),
            "{value:?}"
        );
    }
}
