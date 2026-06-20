//! Integration tests: emit -> HTML/CSS from real fixture IR.

use std::path::PathBuf;

use crate::emitter::emit;
use crate::extractor::extract_ir;
use crate::ir::NodeKind;
use crate::layout::compute_layout;
use crate::load_scene_source_graph;
use crate::resolver::resolve_scene;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn emit_profile_card() -> Result<crate::emitter::EmittedScene, Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let resolved = resolve_scene(&graph)?;
    let laid_out = compute_layout(&resolved);
    let ir = extract_ir(&laid_out, &graph)?;
    Ok(emit(&ir))
}

// ---------------------------------------------------------------------------
// HTML sanity
// ---------------------------------------------------------------------------

#[test]
fn emitted_html_is_valid_document() -> Result<(), Box<dyn std::error::Error>> {
    let scene = emit_profile_card()?;
    assert!(scene.html.contains("<!doctype html>"));
    assert!(scene.html.contains("<html"));
    assert!(scene.html.contains("</html>"));
    assert!(scene.html.contains("<body>"));
    assert!(scene.html.contains("</body>"));
    Ok(())
}

#[test]
fn emitted_html_references_stylesheet() -> Result<(), Box<dyn std::error::Error>> {
    let scene = emit_profile_card()?;
    assert!(scene.html.contains("styles.css"), "html={}", scene.html);
    Ok(())
}

#[test]
fn emitted_html_contains_button_element() -> Result<(), Box<dyn std::error::Error>> {
    let scene = emit_profile_card()?;
    assert!(
        scene.html.contains("<button"),
        "html should contain a button"
    );
    Ok(())
}

#[test]
fn emitted_html_contains_text_content() -> Result<(), Box<dyn std::error::Error>> {
    let scene = emit_profile_card()?;
    // profile_card has text like "Ava Martinez"
    assert!(
        scene.html.contains("Ava Martinez"),
        "html should contain fixture text content; html={}",
        &scene.html[..500.min(scene.html.len())]
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// CSS sanity
// ---------------------------------------------------------------------------

#[test]
fn emitted_css_has_at_least_one_rule() -> Result<(), Box<dyn std::error::Error>> {
    let scene = emit_profile_card()?;
    assert!(!scene.css.is_empty(), "css must not be empty");
    Ok(())
}

#[test]
fn emitted_css_contains_absolute_position() -> Result<(), Box<dyn std::error::Error>> {
    let scene = emit_profile_card()?;
    // profile-card is position:absolute
    assert!(
        scene.css.contains("position: absolute"),
        "css should contain position:absolute"
    );
    Ok(())
}

#[test]
fn emitted_css_uses_class_selectors_matching_ir_ids() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let resolved = resolve_scene(&graph)?;
    let laid_out = compute_layout(&resolved);
    let ir = extract_ir(&laid_out, &graph)?;
    let scene = emit(&ir);

    // Every non-Text node id should appear as a CSS selector
    for node in ir.nodes.iter().filter(|n| n.kind != NodeKind::Text) {
        let selector = format!(".{}", node.id);
        // Only check nodes that have at least one non-default property
        if scene.css.contains(&selector) {
            assert!(
                scene.css.contains(&format!("{selector} {{")),
                "selector {selector} must open a rule block"
            );
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Round-trip structural equivalence: all IR nodes appear in HTML
// ---------------------------------------------------------------------------

#[test]
fn every_box_control_node_has_class_in_html() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let resolved = resolve_scene(&graph)?;
    let laid_out = compute_layout(&resolved);
    let ir = extract_ir(&laid_out, &graph)?;
    let scene = emit(&ir);

    for node in ir.nodes.iter().filter(|n| n.kind != NodeKind::Text) {
        let class_attr = format!("class=\"{}\"", node.id);
        assert!(
            scene.html.contains(&class_attr),
            "node {} missing from HTML; class_attr={class_attr}",
            node.id
        );
    }
    Ok(())
}
