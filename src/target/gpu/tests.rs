//! Headless WGPU scaffold tests (`RET-04`).
//!
//! These render to a real offscreen GPU target and assert on read-back RGBA. If
//! no adapter is available (a headless box with neither hardware nor a software
//! Vulkan/GL driver) the GPU-dependent tests log and soft-skip so CI without a
//! device does not fail; locally and in a GPU-equipped CI they run for real.

use super::*;
use crate::ir::{
    Border, BoxSizing, Color, ControlKind, Display, EdgeInset, ExecutionMode, IrNode, Layout,
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

fn paint(bg: Option<&str>) -> Paint {
    Paint {
        background_color: bg.map(|hex| Color(hex.to_string())),
        border: None,
        border_radius: None,
        cursor: None,
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
        paint: paint(None),
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

/// Acquire a target, or soft-skip the test when no adapter exists.
fn target_or_skip(test: &str) -> Option<WgpuTarget> {
    match WgpuTarget::new() {
        Ok(target) => {
            eprintln!(
                "[{test}] backend={} adapter={}",
                target.backend(),
                target.adapter_name()
            );
            Some(target)
        }
        Err(err) => {
            eprintln!("[{test}] SKIP: no GPU adapter available: {err}");
            None
        }
    }
}

#[test]
fn headless_device_initializes_and_reports_a_backend() {
    let Some(target) = target_or_skip("headless_device_initializes") else {
        return;
    };
    assert_ne!(target.backend(), "Empty");
    assert!(!target.adapter_name().is_empty());
}

#[test]
fn renders_root_background_to_exact_rgba() {
    let Some(target) = target_or_skip("renders_root_background") else {
        return;
    };
    let mut root = node("n0", None);
    root.paint = paint(Some("#336699"));
    let emission = target.emit(&scene(vec![root]));

    assert_eq!(emission.artifact.width, 64);
    assert_eq!(emission.artifact.height, 64);
    // Every pixel is the cleared background; #336699 = (51, 102, 153).
    assert_eq!(emission.artifact.pixel(0, 0), [0x33, 0x66, 0x99, 0xff]);
    assert_eq!(emission.artifact.pixel(32, 32), [0x33, 0x66, 0x99, 0xff]);
    assert_eq!(emission.artifact.pixel(63, 63), [0x33, 0x66, 0x99, 0xff]);
}

#[test]
fn root_without_background_clears_to_white() {
    let Some(target) = target_or_skip("root_without_background") else {
        return;
    };
    let emission = target.emit(&scene(vec![node("n0", None)]));
    assert_eq!(emission.artifact.pixel(10, 10), [0xff, 0xff, 0xff, 0xff]);
}

#[test]
fn render_is_deterministic_across_two_runs() {
    let Some(target) = target_or_skip("render_is_deterministic") else {
        return;
    };
    let mut root = node("n0", None);
    root.paint = paint(Some("#1e293b"));
    let scene = scene(vec![root]);
    assert_eq!(target.emit(&scene).artifact, target.emit(&scene).artifact);
}

#[test]
fn unsupported_capabilities_are_declared_loss_not_silent() {
    let Some(target) = target_or_skip("unsupported_capabilities") else {
        return;
    };
    // A flex container with a border (now rendered) plus a native button child
    // (still declared loss) and a typographic text node (still declared loss).
    let mut root = node("n0", None);
    root.layout.display = Display::Flex;
    root.paint = paint(Some("#ffffff"));
    root.paint.border = Some(Border {
        width: 1.0,
        color: Color("#000000".to_string()),
    });
    let mut button = node("n1", Some("n0"));
    button.kind = NodeKind::Control;
    button.control_kind = Some(ControlKind::Button);

    let emission = target.emit(&scene(vec![root, button]));
    let lost: Vec<Capability> = emission
        .declared_loss
        .iter()
        .map(|gap| gap.capability)
        .collect();

    // The native control is the unsupported family here.
    assert!(lost.contains(&Capability::ButtonControl));
    // Layout and box paint are honored now, so they are never loss.
    assert!(!lost.contains(&Capability::FlexLayout));
    assert!(!lost.contains(&Capability::Border));
    assert!(!lost.contains(&Capability::Background));
    assert!(!lost.contains(&Capability::BlockLayout));
    assert_eq!(target.target_id(), "wgpu");
}

/// Load a canonical fixture's Scene IR by id.
fn fixture_scene(id: &str) -> SceneIr {
    let json = std::fs::read_to_string(
        repo_root().join(format!("fixtures/v0.1/{id}/expected/scene.semui.json")),
    )
    .expect("fixture readable");
    SceneIr::from_json(&json).expect("fixture parses")
}

#[test]
fn profile_card_boxes_rasterize_at_their_resolved_geometry() {
    let Some(target) = target_or_skip("profile_card_boxes") else {
        return;
    };
    let frame = target
        .emit(&fixture_scene("profile_card_absolute"))
        .artifact;

    // Canvas is sized to the resolved extent: the card is 320x180 at (24,20).
    assert_eq!((frame.width, frame.height), (344, 200));

    // Root card fill (#ffffff) at its center.
    assert_eq!(frame.pixel(184, 110), [0xff, 0xff, 0xff, 0xff]);
    // The card's 1px left border (#e5e7eb) at mid-height.
    assert_eq!(frame.pixel(24, 110), [0xe5, 0xe7, 0xeb, 0xff]);

    // Dark primary button (#111827) center: (237,145) 88x36 → (281,163).
    assert_eq!(frame.pixel(281, 163), [0x11, 0x18, 0x27, 0xff]);

    // Green status dot (#10b981) center: (45,147) 8x8 → (49,151).
    assert_eq!(frame.pixel(49, 151), [0x10, 0xb9, 0x81, 0xff]);
}

#[test]
fn border_radius_rounds_corners_so_the_background_shows_through() {
    let Some(target) = target_or_skip("border_radius_rounds") else {
        return;
    };
    let frame = target
        .emit(&fixture_scene("profile_card_absolute"))
        .artifact;

    // The avatar n1 (#dbeafe) is 48x48 at (45,41) with radius 999 → a full
    // circle of radius 24 centered at (69,65).
    // Center is inside the circle: avatar fill.
    assert_eq!(frame.pixel(69, 65), [0xdb, 0xea, 0xfe, 0xff]);
    // The bounding-box corner is outside the circle, so the card behind
    // (#ffffff) shows through rather than the avatar fill.
    assert_eq!(frame.pixel(46, 42), [0xff, 0xff, 0xff, 0xff]);
}

#[test]
fn box_raster_is_deterministic_across_two_runs() {
    let Some(target) = target_or_skip("box_raster_deterministic") else {
        return;
    };
    let scene = fixture_scene("profile_card_absolute");
    assert_eq!(target.emit(&scene).artifact, target.emit(&scene).artifact);
}

#[test]
fn parse_hex_rejects_malformed_input() {
    assert_eq!(parse_hex("#aabbcc"), Some([0xaa, 0xbb, 0xcc]));
    assert_eq!(parse_hex("aabbcc"), None);
    assert_eq!(parse_hex("#abc"), None);
    assert_eq!(parse_hex("#gggggg"), None);
}
