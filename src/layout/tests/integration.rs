use std::path::PathBuf;

use crate::layout::compute_layout;
use crate::load_scene_source_graph;
use crate::resolver::resolve_scene;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// profile_card_absolute is the primary geometry fixture: every element has
/// explicit top/left/width/height from CSS. Geometry extraction must be lossless.
#[test]
fn profile_card_absolute_geometry_extracted() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let resolved = resolve_scene(&graph)?;
    let scene = compute_layout(&resolved);

    assert_eq!(scene.scene_id, "profile_card_absolute");
    assert!(!scene.nodes.is_empty());
    Ok(())
}

#[test]
fn profile_card_has_correct_explicit_coords_and_content_box() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let resolved = resolve_scene(&graph)?;
    let scene = compute_layout(&resolved);

    let Some(card) = scene.nodes.iter().find(|n| {
        n.node.attributes.iter().any(|(k, v)| k == "class" && v.contains("profile-card"))
    }) else {
        return Err("profile-card not found".into());
    };

    let g = &card.geometry;
    // .profile-card: position:absolute left:24 top:20 width:320 height:180 border:1px box-sizing:border-box
    assert_eq!(g.explicit_x, Some(24.0));
    assert_eq!(g.explicit_y, Some(20.0));
    assert_eq!(g.width, Some(320.0));
    assert_eq!(g.height, Some(180.0));
    // content-box: 320 - 0padding - 2border = 318; 180 - 0padding - 2border = 178
    assert_eq!(g.content_width, Some(318.0));
    assert_eq!(g.content_height, Some(178.0));
    Ok(())
}

#[test]
fn avatar_circle_has_border_box_content_size() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let resolved = resolve_scene(&graph)?;
    let scene = compute_layout(&resolved);

    let Some(avatar) = scene.nodes.iter().find(|n| {
        n.node.attributes.iter().any(|(k, v)| k == "class" && v.contains("avatar"))
    }) else {
        return Err("avatar not found".into());
    };

    let g = &avatar.geometry;
    // .avatar: position:absolute left:20 top:20 width:48 height:48 no-border box-sizing:border-box
    assert_eq!(g.explicit_x, Some(20.0));
    assert_eq!(g.explicit_y, Some(20.0));
    assert_eq!(g.width, Some(48.0));
    // no padding, no border → content size = declared size
    assert_eq!(g.content_width, Some(48.0));
    assert_eq!(g.content_height, Some(48.0));
    Ok(())
}

#[test]
fn stacked_card_has_explicit_container_width() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "stacked_info_card")?;
    let resolved = resolve_scene(&graph)?;
    let scene = compute_layout(&resolved);

    let Some(card) = scene.nodes.iter().find(|n| {
        n.node.attributes.iter().any(|(k, v)| k == "class" && v == "card")
    }) else {
        return Err("card not found".into());
    };

    let g = &card.geometry;
    // .card: display:flex width:320 padding:20 border:1 box-sizing:border-box
    assert_eq!(g.width, Some(320.0));
    // content_width = 320 - (20+20) - (1+1) = 278
    assert_eq!(g.content_width, Some(278.0));
    // Not absolutely positioned → no explicit coords
    assert_eq!(g.explicit_x, None);
    assert_eq!(g.explicit_y, None);
    Ok(())
}

/// Negative: border_radius and border_width pass through correctly.
#[test]
fn profile_card_border_radius_and_width_in_geometry() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let resolved = resolve_scene(&graph)?;
    let scene = compute_layout(&resolved);

    let Some(card) = scene.nodes.iter().find(|n| {
        n.node.attributes.iter().any(|(k, v)| k == "class" && v.contains("profile-card"))
    }) else {
        return Err("profile-card not found".into());
    };

    assert_eq!(card.geometry.border_width, 1.0);
    assert_eq!(card.geometry.border_radius, 16.0);
    Ok(())
}
