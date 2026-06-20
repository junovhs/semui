use crate::HtmlNode;
use crate::HtmlNodeKind;
use crate::layout::compute_layout;
use crate::resolver::{ComputedStyle, ResolvedNode, ResolvedScene};

fn elem(id: usize, name: &str) -> HtmlNode {
    HtmlNode {
        id,
        parent_id: None,
        kind: HtmlNodeKind::Element,
        name: Some(name.to_owned()),
        text: None,
        attributes: Vec::new(),
        dom_path: format!("0/{id}"),
        document_id: 0,
    }
}

fn scene_with(style: ComputedStyle) -> ResolvedScene {
    ResolvedScene {
        scene_id: "test".to_owned(),
        nodes: vec![ResolvedNode {
            node: elem(0, "div"),
            style,
        }],
    }
}

// --- Absolute coordinate extraction ---

#[test]
fn absolute_position_extracts_left_and_top() {
    let style = ComputedStyle {
        position: "absolute".to_owned(),
        left: Some(24.0),
        top: Some(20.0),
        width: Some(320.0),
        height: Some(180.0),
        ..ComputedStyle::default()
    };
    let scene = compute_layout(&scene_with(style));
    let g = &scene.nodes[0].geometry;
    assert_eq!(g.explicit_x, Some(24.0));
    assert_eq!(g.explicit_y, Some(20.0));
    assert_eq!(g.width, Some(320.0));
    assert_eq!(g.height, Some(180.0));
}

#[test]
fn static_position_has_no_explicit_coords() {
    let style = ComputedStyle {
        position: "static".to_owned(),
        left: Some(10.0),
        top: Some(10.0),
        ..ComputedStyle::default()
    };
    let scene = compute_layout(&scene_with(style));
    let g = &scene.nodes[0].geometry;
    assert_eq!(
        g.explicit_x, None,
        "static elements must not produce explicit_x"
    );
    assert_eq!(
        g.explicit_y, None,
        "static elements must not produce explicit_y"
    );
}

#[test]
fn absolute_without_coords_produces_none() {
    let style = ComputedStyle {
        position: "absolute".to_owned(),
        left: None,
        top: None,
        ..ComputedStyle::default()
    };
    let scene = compute_layout(&scene_with(style));
    let g = &scene.nodes[0].geometry;
    assert_eq!(g.explicit_x, None);
    assert_eq!(g.explicit_y, None);
}

// --- Box-sizing normalization ---

#[test]
fn border_box_subtracts_padding_and_border_from_width() {
    let style = ComputedStyle {
        box_sizing: "border-box".to_owned(),
        width: Some(320.0),
        height: Some(180.0),
        padding_left: 20.0,
        padding_right: 20.0,
        padding_top: 10.0,
        padding_bottom: 10.0,
        border_width: 1.0,
        ..ComputedStyle::default()
    };
    let scene = compute_layout(&scene_with(style));
    let g = &scene.nodes[0].geometry;
    // content_width = 320 - (20+20) - (1+1) = 278
    assert_eq!(g.content_width, Some(278.0));
    // content_height = 180 - (10+10) - (1+1) = 158
    assert_eq!(g.content_height, Some(158.0));
}

#[test]
fn content_box_passes_width_through_unchanged() {
    let style = ComputedStyle {
        box_sizing: "content-box".to_owned(),
        width: Some(200.0),
        height: Some(100.0),
        padding_left: 16.0,
        padding_right: 16.0,
        border_width: 2.0,
        ..ComputedStyle::default()
    };
    let scene = compute_layout(&scene_with(style));
    let g = &scene.nodes[0].geometry;
    assert_eq!(
        g.content_width,
        Some(200.0),
        "content-box: declared width is the content width"
    );
    assert_eq!(g.content_height, Some(100.0));
}

#[test]
fn border_box_clamped_to_zero_when_inset_exceeds_width() {
    let style = ComputedStyle {
        box_sizing: "border-box".to_owned(),
        width: Some(10.0),
        padding_left: 8.0,
        padding_right: 8.0,
        border_width: 1.0,
        ..ComputedStyle::default()
    };
    let scene = compute_layout(&scene_with(style));
    let g = &scene.nodes[0].geometry;
    assert_eq!(
        g.content_width,
        Some(0.0),
        "content width must not go negative"
    );
}

#[test]
fn none_width_produces_none_content_width() {
    let style = ComputedStyle {
        box_sizing: "border-box".to_owned(),
        width: None,
        ..ComputedStyle::default()
    };
    let scene = compute_layout(&scene_with(style));
    assert_eq!(scene.nodes[0].geometry.content_width, None);
}

// --- Margin and padding pass-through ---

#[test]
fn margin_and_padding_preserved_in_geometry() {
    let style = ComputedStyle {
        margin_top: 4.0,
        margin_right: 8.0,
        margin_bottom: 12.0,
        margin_left: 16.0,
        padding_top: 1.0,
        padding_right: 2.0,
        padding_bottom: 3.0,
        padding_left: 4.0,
        ..ComputedStyle::default()
    };
    let scene = compute_layout(&scene_with(style));
    let g = &scene.nodes[0].geometry;
    assert_eq!(g.margin, [4.0, 8.0, 12.0, 16.0]);
    assert_eq!(g.padding, [1.0, 2.0, 3.0, 4.0]);
}
