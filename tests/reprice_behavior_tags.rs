//! The wire tags of `RepriceBehavior` are the abbreviations of the variant names, and the doc
//! comments describe the variant they sit on. A reader who trusted the comments before this
//! sent `Reject` expecting a reprice.

use ax_exchange_sdk::types::trading::RepriceBehavior;

#[test]
fn the_wire_tags_abbreviate_the_variant_names() {
    assert_eq!(
        serde_json::to_string(&RepriceBehavior::Reject).unwrap(),
        "\"rej\""
    );
    assert_eq!(
        serde_json::to_string(&RepriceBehavior::BestOffer).unwrap(),
        "\"bo\""
    );
    assert_eq!(
        serde_json::to_string(&RepriceBehavior::TopOfBookLimit).unwrap(),
        "\"tbl\""
    );
}

#[test]
fn the_doc_comments_describe_the_variant_they_sit_on() {
    let src = include_str!("../src/types/trading.rs");
    let enum_src = &src[src.find("pub enum RepriceBehavior").unwrap()..];
    let enum_src = &enum_src[..enum_src.find("\n}\n").unwrap()];
    let comment_before = |variant: &str| -> String {
        let at = enum_src.find(&format!("    {variant},")).unwrap();
        enum_src[..at]
            .lines()
            .rev()
            .take_while(|l| l.trim_start().starts_with("///") || l.trim_start().starts_with("#["))
            .filter(|l| l.trim_start().starts_with("///"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    assert!(
        comment_before("Reject").contains("Reject the order"),
        "{}",
        comment_before("Reject")
    );
    assert!(
        comment_before("BestOffer").contains("Reprice the order"),
        "{}",
        comment_before("BestOffer")
    );
}
