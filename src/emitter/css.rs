//! CSS generation from IR nodes.

use crate::ir::layout::{AlignItems, AlignSelf, BoxSizing, Cursor, Display,
    EdgeInset, FlexDirection, JustifyContent, Layout, Position};
use crate::ir::paint::Paint;
use crate::ir::typography::{LineHeight, Typography};
use crate::ir::{IrNode, NodeKind};

/// Emit a complete `<style>` block for all non-Text nodes in `nodes`.
pub fn build_css(nodes: &[IrNode]) -> String {
    let rules: Vec<String> = nodes
        .iter()
        .filter(|n| n.kind != NodeKind::Text)
        .filter_map(emit_rule)
        .collect();
    rules.join("\n\n")
}

fn emit_rule(node: &IrNode) -> Option<String> {
    let mut decls: Vec<String> = Vec::new();
    // Reset native browser chrome for interactive controls before any layout
    // or paint declarations. `appearance` is not in the v0.1 cascade subset so
    // it is never stored in the IR, but every button must opt out of it to avoid
    // OS-level gradients, inset borders, and text-baseline drift on round-trip.
    if node.kind == NodeKind::Control {
        decls.push("appearance: none".to_owned());
    }
    decls.extend(layout_decls(&node.layout));
    decls.extend(paint_decls(&node.paint));
    if let Some(ref typo) = node.typography {
        decls.extend(typography_decls(typo));
    }
    if decls.is_empty() {
        return None;
    }
    let body = decls.iter().map(|d| format!("  {d};")).collect::<Vec<_>>().join("\n");
    Some(format!(".{} {{\n{body}\n}}", node.id))
}

fn layout_decls(l: &Layout) -> Vec<String> {
    let mut d: Vec<String> = Vec::new();
    if l.position == Position::Absolute {
        d.push("position: absolute".to_owned());
    }
    match l.display {
        Display::Flex => d.push("display: flex".to_owned()),
        Display::InlineFlex => d.push("display: inline-flex".to_owned()),
        Display::Block => {}
    }
    if l.box_sizing == BoxSizing::BorderBox {
        d.push("box-sizing: border-box".to_owned());
    }
    if let Some(v) = l.top    { d.push(format!("top: {}", px(v))); }
    if let Some(v) = l.left   { d.push(format!("left: {}", px(v))); }
    if let Some(v) = l.width  { d.push(format!("width: {}", px(v))); }
    if let Some(v) = l.height { d.push(format!("height: {}", px(v))); }
    if let Some(v) = l.min_width { d.push(format!("min-width: {}", px(v))); }
    if let Some(s) = edge_shorthand("margin", &l.margin) { d.push(s); }
    if let Some(s) = edge_shorthand("padding", &l.padding) { d.push(s); }
    if let Some(ref fd) = l.flex_direction {
        d.push(format!("flex-direction: {}", flex_dir(fd)));
    }
    if let Some(ref ai) = l.align_items {
        d.push(format!("align-items: {}", align_items_val(ai)));
    }
    if let Some(ref jc) = l.justify_content {
        d.push(format!("justify-content: {}", justify_val(jc)));
    }
    if let Some(ref ase) = l.align_self {
        d.push(format!("align-self: {}", align_self_val(ase)));
    }
    if let Some(v) = l.gap { d.push(format!("gap: {}", px(v))); }
    d
}

fn paint_decls(p: &Paint) -> Vec<String> {
    let mut d: Vec<String> = Vec::new();
    if let Some(ref bg) = p.background_color {
        d.push(format!("background-color: {}", bg.0));
    }
    if let Some(ref b) = p.border {
        d.push(format!("border: {} solid {}", px(b.width), b.color.0));
    }
    if let Some(r) = p.border_radius {
        d.push(format!("border-radius: {}", px(r)));
    }
    if let Some(ref c) = p.cursor {
        d.push(format!("cursor: {}", cursor_val(c)));
    }
    d
}

fn typography_decls(t: &Typography) -> Vec<String> {
    let mut d: Vec<String> = Vec::new();
    d.push(format!("font-family: {}", t.font_family.join(", ")));
    d.push(format!("font-size: {}", px(t.font_size)));
    d.push(format!("font-weight: {}", t.font_weight));
    d.push(format!("line-height: {}", line_height_val(&t.line_height)));
    d.push(format!("color: {}", t.color.0));
    d
}

// ---------------------------------------------------------------------------
// Value helpers
// ---------------------------------------------------------------------------

pub fn px(v: f32) -> String {
    if v.fract() == 0.0 { format!("{}px", v as i64) } else { format!("{v}px") }
}

fn edge_shorthand(name: &str, e: &EdgeInset) -> Option<String> {
    if e.top == 0.0 && e.right == 0.0 && e.bottom == 0.0 && e.left == 0.0 {
        return None;
    }
    let s = if e.top == e.right && e.right == e.bottom && e.bottom == e.left {
        format!("{name}: {}", px(e.top))
    } else if e.top == e.bottom && e.left == e.right {
        format!("{name}: {} {}", px(e.top), px(e.right))
    } else {
        format!("{name}: {} {} {} {}", px(e.top), px(e.right), px(e.bottom), px(e.left))
    };
    Some(s)
}

fn flex_dir(fd: &FlexDirection) -> &'static str {
    match fd { FlexDirection::Row => "row", FlexDirection::Column => "column" }
}

fn align_items_val(ai: &AlignItems) -> &'static str {
    match ai { AlignItems::Center => "center" }
}

fn justify_val(jc: &JustifyContent) -> &'static str {
    match jc { JustifyContent::Center => "center" }
}

fn align_self_val(ase: &AlignSelf) -> &'static str {
    match ase { AlignSelf::Auto => "auto", AlignSelf::FlexStart => "flex-start" }
}

fn cursor_val(c: &Cursor) -> &'static str {
    match c { Cursor::Auto => "auto", Cursor::Pointer => "pointer" }
}

fn line_height_val(lh: &LineHeight) -> String {
    match lh {
        LineHeight::Normal => "normal".to_owned(),
        LineHeight::Length { value } => px(*value),
    }
}
