#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use scripts::enums::card_priority_style::CardPriorityStyle::{Affinity, Servant};
use scripts::models::per_entry_settings as per_entry;

#[test]
fn blank_stored_text_is_an_empty_list_so_the_old_values_apply() {
    assert!(per_entry::parse_styles("").is_empty());
    assert!(per_entry::parse_flags("  ").is_empty());
    assert_eq!(
        per_entry::with_legacy(&per_entry::parse_styles(""), Servant),
        vec![Servant]
    );
    assert_eq!(
        per_entry::with_legacy(&per_entry::parse_flags(""), true),
        vec![true]
    );
}

#[test]
fn stored_text_round_trips_and_tolerates_bad_entries() {
    let styles = vec![Affinity, Servant, Servant];
    assert_eq!(
        per_entry::serialize_styles(&styles),
        "Affinity,Servant,Servant"
    );
    assert_eq!(per_entry::parse_styles("Affinity,Servant,Servant"), styles);
    assert_eq!(
        per_entry::parse_styles("Servant,Nonsense"),
        vec![Servant, Affinity]
    );

    let flags = vec![true, false];
    assert_eq!(per_entry::serialize_flags(&flags), "true,false");
    assert_eq!(per_entry::parse_flags("true,false"), flags);
}
