use super::*;
use crate::FixtureManifest;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn loads_fixture_manifest() {
    let manifest_path = repo_root().join("fixtures").join("v0.1").join("manifest.toml");
    let manifest = FixtureManifest::load(manifest_path).expect("manifest should load");

    assert_eq!(manifest.corpus, "v0.1");
    assert_eq!(manifest.scenes.len(), 6);
    assert!(manifest
        .scenes
        .iter()
        .any(|scene| scene.id == "profile_card_absolute"));
}

#[test]
fn builds_source_graph_for_profile_card() {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")
        .expect("fixture source graph should load");

    assert_eq!(graph.scene_id, "profile_card_absolute");
    assert!(graph
        .html_nodes
        .iter()
        .any(|node| node.name.as_deref() == Some("button")));
    assert!(graph
        .html_nodes
        .iter()
        .any(|node| node.text.as_deref() == Some("Ava Martinez")));
    assert!(graph
        .css_rules
        .iter()
        .any(|rule| rule.selectors.iter().any(|selector| selector == ".profile-card")));
}

#[test]
fn returns_error_for_unknown_scene() {
    let error =
        load_scene_source_graph(repo_root(), "missing_scene").expect_err("unknown scenes should fail");

    let message = error.to_string();
    assert!(message.contains("missing_scene"));
}

#[test]
fn rejects_css_at_rules_in_v0_subset() {
    let css = "@media (min-width: 800px) { .card { width: 100px; } }";
    let error =
        css::parse_css_document(1, css).expect_err("at-rules should be rejected in the v0 subset");

    assert!(matches!(error, SourceGraphError::UnsupportedCss { .. }));
}
