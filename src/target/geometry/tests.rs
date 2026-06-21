//! Geometry-resolution tests (`RET-05`). These are pure (no GPU): they assert
//! the resolved border boxes for absolute, block, and flex layout directly.

use super::*;
use crate::ir::{
    Border, BoxSizing, Color, Display, EdgeInset, ExecutionMode, FlexDirection, IrNode, Layout,
    NodeKind, Paint, Position, SceneIr, SourceRef,
};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn base_layout() -> Layout {
    Layout {
        position: Position::Static,
        display: Display::Block,
        box_sizing: BoxSizing::BorderBox,
        top: None,
        left: None,
        width: None,
        height: None,
        min_width: None,
        margin: EdgeInset::zero(),
        padding: EdgeInset::zero(),
        flex_direction: None,
        align_items: None,
        justify_content: None,
        align_self: None,
        gap: None,
    }
}

fn node(id: &str, parent: Option<&str>) -> IrNode {
    IrNode {
        id: id.to_string(),
        kind: NodeKind::Box,
        parent_id: parent.map(str::to_string),
        control_kind: None,
        text_content: None,
        layout: base_layout(),
        paint: Paint {
            background_color: None,
            border: None,
            border_radius: None,
            cursor: None,
        },
        typography: None,
        source: SourceRef {
            doc_id: 0,
            dom_path: id.to_string(),
            span: None,
        },
    }
}

fn scene(nodes: Vec<IrNode>) -> SceneIr {
    SceneIr {
        schema_version: 1,
        scene_id: "t".to_string(),
        corpus: "v0.1".to_string(),
        execution_mode: ExecutionMode::Static,
        nodes,
    }
}

#[test]
fn absolute_child_is_placed_against_the_containing_block_padding_box() {
    // Root absolute at (24,20) 320x180 with a 1px border, so its padding box
    // origin is (25,21). An absolute child at top=20,left=20 lands at (45,41).
    let mut root = node("n0", None);
    root.layout.position = Position::Absolute;
    root.layout.left = Some(24.0);
    root.layout.top = Some(20.0);
    root.layout.width = Some(320.0);
    root.layout.height = Some(180.0);
    root.paint.border = Some(Border {
        width: 1.0,
        color: Color("#000000".to_string()),
    });

    let mut child = node("n1", Some("n0"));
    child.layout.position = Position::Absolute;
    child.layout.left = Some(20.0);
    child.layout.top = Some(20.0);
    child.layout.width = Some(48.0);
    child.layout.height = Some(48.0);

    let g = resolve_geometry(&scene(vec![root, child]));
    assert_eq!(
        g["n0"],
        BoxRect {
            x: 24.0,
            y: 20.0,
            width: 320.0,
            height: 180.0
        }
    );
    assert_eq!(
        g["n1"],
        BoxRect {
            x: 45.0,
            y: 41.0,
            width: 48.0,
            height: 48.0
        }
    );
}

#[test]
fn content_box_sizing_expands_the_border_box_by_padding_and_border() {
    // content-box 100x40 with 8px padding all round and a 2px border:
    // border box = 100 + 8 + 8 + 2 + 2 = 120 wide, 40 + 8 + 8 + 4 = 60 tall.
    let mut root = node("n0", None);
    root.layout.position = Position::Absolute;
    root.layout.left = Some(0.0);
    root.layout.top = Some(0.0);
    root.layout.box_sizing = BoxSizing::ContentBox;
    root.layout.width = Some(100.0);
    root.layout.height = Some(40.0);
    root.layout.padding = EdgeInset::uniform(8.0);
    root.paint.border = Some(Border {
        width: 2.0,
        color: Color("#000000".to_string()),
    });

    let g = resolve_geometry(&scene(vec![root]));
    assert_eq!(g["n0"].width, 120.0);
    assert_eq!(g["n0"].height, 60.0);
}

#[test]
fn block_children_stack_vertically_with_margins() {
    let mut root = node("n0", None);
    root.layout.position = Position::Absolute;
    root.layout.left = Some(0.0);
    root.layout.top = Some(0.0);
    root.layout.width = Some(200.0);
    root.layout.height = Some(300.0);
    root.layout.padding = EdgeInset::uniform(10.0);

    let mut a = node("a", Some("n0"));
    a.layout.height = Some(30.0);
    a.layout.margin = EdgeInset {
        top: 5.0,
        right: 0.0,
        bottom: 7.0,
        left: 0.0,
    };
    let mut b = node("b", Some("n0"));
    b.layout.height = Some(40.0);

    let g = resolve_geometry(&scene(vec![root, a, b]));
    // content box origin is (10,10), width 180.
    assert_eq!(
        g["a"],
        BoxRect {
            x: 10.0,
            y: 15.0,
            width: 180.0,
            height: 30.0
        }
    );
    // next cursor = 15 + 30 + 7 = 52.
    assert_eq!(
        g["b"],
        BoxRect {
            x: 10.0,
            y: 52.0,
            width: 180.0,
            height: 40.0
        }
    );
}

#[test]
fn flex_row_packs_along_the_main_axis_with_gap_and_centers_the_cross_axis() {
    let mut root = node("n0", None);
    root.layout.position = Position::Absolute;
    root.layout.left = Some(0.0);
    root.layout.top = Some(0.0);
    root.layout.width = Some(200.0);
    root.layout.height = Some(100.0);
    root.layout.display = Display::Flex;
    root.layout.flex_direction = Some(FlexDirection::Row);
    root.layout.align_items = Some(crate::ir::AlignItems::Center);
    root.layout.gap = Some(12.0);

    let mut a = node("a", Some("n0"));
    a.layout.width = Some(30.0);
    a.layout.height = Some(20.0);
    let mut b = node("b", Some("n0"));
    b.layout.width = Some(40.0);
    b.layout.height = Some(60.0);

    let g = resolve_geometry(&scene(vec![root, a, b]));
    // a: main x=0, cross centered in 100 → (100-20)/2 = 40.
    assert_eq!(
        g["a"],
        BoxRect {
            x: 0.0,
            y: 40.0,
            width: 30.0,
            height: 20.0
        }
    );
    // b: main x = 30 + 12 = 42, cross centered → (100-60)/2 = 20.
    assert_eq!(
        g["b"],
        BoxRect {
            x: 42.0,
            y: 20.0,
            width: 40.0,
            height: 60.0
        }
    );
}

#[test]
fn content_sized_nodes_are_omitted_not_guessed() {
    // An absolute box with no explicit height cannot resolve without content.
    let mut root = node("n0", None);
    root.layout.position = Position::Absolute;
    root.layout.left = Some(0.0);
    root.layout.top = Some(0.0);
    root.layout.width = Some(100.0);
    root.layout.height = Some(100.0);

    let mut badge = node("badge", Some("n0"));
    badge.layout.position = Position::Absolute;
    badge.layout.left = Some(10.0);
    badge.layout.top = Some(10.0);
    badge.layout.height = Some(24.0); // width is content-driven → omitted

    let g = resolve_geometry(&scene(vec![root, badge]));
    assert!(g.contains_key("n0"));
    assert!(!g.contains_key("badge"));
}

#[test]
fn canvas_extent_bounds_every_resolved_box() {
    let mut root = node("n0", None);
    root.layout.position = Position::Absolute;
    root.layout.left = Some(24.0);
    root.layout.top = Some(20.0);
    root.layout.width = Some(320.0);
    root.layout.height = Some(180.0);

    let g = resolve_geometry(&scene(vec![root]));
    assert_eq!(canvas_extent(&g), Some((344, 200)));
    assert_eq!(canvas_extent(&BTreeMap::new()), None);
}

#[test]
fn profile_card_absolute_resolves_its_explicit_boxes() {
    let json = std::fs::read_to_string(
        repo_root().join("fixtures/v0.1/profile_card_absolute/expected/scene.semui.json"),
    )
    .expect("fixture readable");
    let ir = SceneIr::from_json(&json).expect("fixture parses");
    let g = resolve_geometry(&ir);

    // Root card at its own absolute offset.
    assert_eq!(
        g["n0"],
        BoxRect {
            x: 24.0,
            y: 20.0,
            width: 320.0,
            height: 180.0
        }
    );
    // n0 has a 1px border, so its padding box origin is (25,21). The avatar n1
    // is absolute at top=20,left=20 → (45,41), 48x48.
    assert_eq!(
        g["n1"],
        BoxRect {
            x: 45.0,
            y: 41.0,
            width: 48.0,
            height: 48.0
        }
    );
    // The dark primary button n14: top=124,left=212 → (237,145), 88x36.
    assert_eq!(
        g["n14"],
        BoxRect {
            x: 237.0,
            y: 145.0,
            width: 88.0,
            height: 36.0
        }
    );
    // The badge n7 has no explicit width (content-sized) → omitted.
    assert!(!g.contains_key("n7"));
    // Text nodes never resolve to a box.
    assert!(!g.contains_key("n2"));
}
