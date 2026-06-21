//! Integration tests: run analyze() against the real fixture corpus.

use std::path::PathBuf;

use crate::diagnostics::{DiagnosticKind, analyze};
use crate::load_scene_source_graph;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// ---------------------------------------------------------------------------
// profile_card_absolute
// ---------------------------------------------------------------------------

#[test]
fn profile_card_produces_no_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let diags = analyze(&graph);
    assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
    Ok(())
}

#[test]
fn profile_card_has_no_unsupported_selectors() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let diags = analyze(&graph);

    let bad_selectors: Vec<_> = diags
        .iter()
        .filter(|d| d.kind == DiagnosticKind::UnsupportedSelector)
        .collect();

    assert!(
        bad_selectors.is_empty(),
        "unexpected selector issues: {bad_selectors:?}"
    );
    Ok(())
}

#[test]
fn profile_card_has_no_unsupported_values() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let diags = analyze(&graph);

    let bad_values: Vec<_> = diags
        .iter()
        .filter(|d| d.kind == DiagnosticKind::UnsupportedValue)
        .collect();

    assert!(
        bad_values.is_empty(),
        "unexpected value issues: {bad_values:?}"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// stacked_info_card — clean scene
// ---------------------------------------------------------------------------

#[test]
fn stacked_info_card_produces_no_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "stacked_info_card")?;
    let diags = analyze(&graph);
    assert!(
        diags.is_empty(),
        "stacked_info_card should produce no diagnostics; got: {diags:?}"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// action_row_variants uses the supported `appearance: none` reset
// ---------------------------------------------------------------------------

#[test]
fn action_row_produces_no_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "action_row_variants")?;
    let diags = analyze(&graph);

    assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
    Ok(())
}

// ---------------------------------------------------------------------------
// analyze returns diagnostic with correct rule_id
// ---------------------------------------------------------------------------

#[test]
fn diagnostic_rule_id_is_valid_index() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let diags = analyze(&graph);

    for d in &diags {
        assert!(
            graph.css_rules.iter().any(|r| r.id == d.rule_id),
            "diagnostic rule_id {} not found in css_rules",
            d.rule_id
        );
    }
    Ok(())
}
