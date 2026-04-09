use std::path::PathBuf;

use crate::load_scene_source_graph;
use crate::resolver::resolve_scene;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The profile_card_absolute fixture is the primary anchor scene and the most
/// demanding for the resolver: absolute positioning, compound selectors,
/// cascade between shared and specific rules, and font-family inheritance.
#[test]
fn profile_card_absolute_resolves_without_error() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let scene = resolve_scene(&graph)?;
    assert_eq!(scene.scene_id, "profile_card_absolute");
    assert!(!scene.nodes.is_empty(), "resolved scene must have element nodes");
    Ok(())
}

#[test]
fn profile_card_card_has_absolute_position() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let scene = resolve_scene(&graph)?;

    let Some(card) = scene.nodes.iter().find(|n| {
        n.node.attributes.iter().any(|(k, v)| k == "class" && v.contains("profile-card"))
    }) else {
        return Err("profile-card element not found in resolved scene".into());
    };

    assert_eq!(card.style.position, "absolute");
    assert_eq!(card.style.background_color, Some("#ffffff".to_owned()));
    assert_eq!(card.style.border_width, 1.0);
    assert_eq!(card.style.border_color, Some("#e5e7eb".to_owned()));
    assert_eq!(card.style.width, Some(320.0));
    assert_eq!(card.style.height, Some(180.0));
    Ok(())
}

#[test]
fn cascade_specificity_button_primary_wins_over_button() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let scene = resolve_scene(&graph)?;

    // .button.primary has specificity (2,0); it overrides .button specificity (1,0)
    let Some(primary) = scene.nodes.iter().find(|n| {
        n.node.name.as_deref() == Some("button")
            && n.node.attributes.iter().any(|(k, v)| k == "class" && v.contains("primary"))
    }) else {
        return Err("primary button not found".into());
    };
    assert_eq!(primary.style.background_color, Some("#111827".to_owned()));
    assert_eq!(primary.style.color, Some("#ffffff".to_owned()));

    let Some(secondary) = scene.nodes.iter().find(|n| {
        n.node.name.as_deref() == Some("button")
            && n.node.attributes.iter().any(|(k, v)| k == "class" && v.contains("secondary"))
    }) else {
        return Err("secondary button not found".into());
    };
    assert_eq!(secondary.style.background_color, Some("#ffffff".to_owned()));
    assert_eq!(secondary.style.color, Some("#111827".to_owned()));
    Ok(())
}

#[test]
fn font_family_inherits_from_body_to_children() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let scene = resolve_scene(&graph)?;

    // body sets font-family: Inter, sans-serif
    // .button sets font-family: inherit → inherits from ancestor body
    let Some(primary) = scene.nodes.iter().find(|n| {
        n.node.name.as_deref() == Some("button")
            && n.node.attributes.iter().any(|(k, v)| k == "class" && v.contains("primary"))
    }) else {
        return Err("primary button not found".into());
    };
    assert_eq!(
        primary.style.font_family.as_deref(),
        Some("Inter, sans-serif"),
        "font-family must propagate from body through inheritance"
    );
    Ok(())
}

#[test]
fn stacked_info_card_flex_properties_resolved() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "stacked_info_card")?;
    let scene = resolve_scene(&graph)?;

    let Some(card) = scene.nodes.iter().find(|n| {
        n.node.attributes.iter().any(|(k, v)| k == "class" && v == "card")
    }) else {
        return Err("card element not found".into());
    };
    assert_eq!(card.style.display, "flex");
    assert_eq!(card.style.flex_direction, Some("column".to_owned()));
    assert_eq!(card.style.gap, Some(12.0));
    assert_eq!(card.style.width, Some(320.0));
    Ok(())
}

/// Negative: unknown scene fails at the graph load level, not the resolver.
#[test]
fn unknown_scene_fails_before_resolve() {
    let result = load_scene_source_graph(repo_root(), "does_not_exist");
    assert!(result.is_err(), "unknown scene must return an error");
}
