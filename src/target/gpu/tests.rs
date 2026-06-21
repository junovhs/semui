//! Headless WGPU scaffold tests (`RET-04`).
//!
//! These render to a real offscreen GPU target and assert on read-back RGBA. If
//! no adapter is available (a headless box with neither hardware nor a software
//! Vulkan/GL driver) the GPU-dependent tests log and soft-skip so CI without a
//! device does not fail; locally and in a GPU-equipped CI they run for real.

use super::*;
use crate::ir::{
    BoxSizing, Color, ControlKind, Display, EdgeInset, ExecutionMode, IrNode, Layout, NodeKind,
    Paint, Position, SceneIr, SourceRef,
};

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
    // A flex container with a border plus a button child — none of which the
    // scaffold renders.
    let mut root = node("n0", None);
    root.layout.display = Display::Flex;
    root.paint = paint(Some("#ffffff"));
    root.paint.border = Some(crate::ir::Border {
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

    assert!(lost.contains(&Capability::FlexLayout));
    assert!(lost.contains(&Capability::Border));
    assert!(lost.contains(&Capability::ButtonControl));
    // Background is the one capability the scaffold honors, so it is never loss.
    assert!(!lost.contains(&Capability::Background));
    assert_eq!(target.target_id(), "wgpu");
}

#[test]
fn parse_hex_rejects_malformed_input() {
    assert_eq!(parse_hex("#aabbcc"), Some([0xaa, 0xbb, 0xcc]));
    assert_eq!(parse_hex("aabbcc"), None);
    assert_eq!(parse_hex("#abc"), None);
    assert_eq!(parse_hex("#gggggg"), None);
}
