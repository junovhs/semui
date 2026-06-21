//! Scene-IR geometry resolution for the WGPU reference target (`RET-05`).
//!
//! A target consumes only canonical [`SceneIr`] (per `DEC-04`), which carries
//! CSS layout *inputs* — `position`/`top`/`left`/`width`/`height`, the box-model
//! edges, and the v0.1 flex fields — but not resolved screen rectangles. This
//! module is the geometry pass that turns those inputs into an absolute,
//! pixel-space [`BoxRect`] per node under the `RET-01` conventions (top-left
//! origin, `y` growing downward, CSS pixels).
//!
//! Three layout families are resolved: `position: absolute` against the padding
//! box of the nearest positioned ancestor, block vertical flow, and the v0.1
//! flex subset (row/column with `gap`, `align-items: center`, and
//! `justify-content: center`). Each [`BoxRect`] is a node's **border box**, the
//! surface the box rasterizer fills, borders, and rounds.
//!
//! ## Intrinsic (content) sizing is out of scope
//!
//! A box whose width or height is `auto` is sized by its content, which for the
//! canonical scenes means measured text — and text rendering is a later child
//! (`RET-06`). Such a node has no determinable border box here and is simply
//! absent from the resolved map rather than guessed. Only nodes with explicit
//! geometry (directly, or as a block child inheriting its container's width)
//! resolve.

use std::collections::BTreeMap;

use crate::ir::{
    AlignItems, BoxSizing, Display, FlexDirection, IrNode, JustifyContent, Position, SceneIr,
};

/// An axis-aligned rectangle in target pixel space: a top-left origin with `y`
/// growing downward, lengths in CSS pixels — the `RET-01` conventions. A
/// resolved [`BoxRect`] is always a node's **border box**.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl BoxRect {
    /// The x coordinate of the right edge.
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// The y coordinate of the bottom edge.
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// This border box inset uniformly by `d` on every side (clamped to zero
    /// size). Used to derive the padding box (inset by the border) and the
    /// content box (inset by border + padding).
    fn inset(&self, top: f32, right: f32, bottom: f32, left: f32) -> BoxRect {
        BoxRect {
            x: self.x + left,
            y: self.y + top,
            width: (self.width - left - right).max(0.0),
            height: (self.height - top - bottom).max(0.0),
        }
    }
}

/// Resolve every node's absolute border box. Nodes whose size is content-driven
/// (and thus needs text measurement) are omitted. Keys are IR node ids.
pub fn resolve_geometry(scene: &SceneIr) -> BTreeMap<String, BoxRect> {
    let children = ChildIndex::build(scene);
    let mut out = BTreeMap::new();

    for root in scene.nodes.iter().filter(|n| n.parent_id.is_none()) {
        let Some(size) = explicit_border_box(root) else {
            // A root with no explicit size would need content measurement.
            continue;
        };
        let origin = root_origin(root);
        let border_box = BoxRect {
            x: origin.0,
            y: origin.1,
            width: size.0,
            height: size.1,
        };
        layout_subtree(
            root,
            border_box,
            padding_box(root, border_box),
            &children,
            &mut out,
        );
    }

    out
}

/// The smallest canvas (width, height) in whole pixels that contains every
/// resolved box, or `None` when nothing resolved.
pub fn canvas_extent(boxes: &BTreeMap<String, BoxRect>) -> Option<(u32, u32)> {
    let mut max_right = 0.0_f32;
    let mut max_bottom = 0.0_f32;
    for rect in boxes.values() {
        max_right = max_right.max(rect.right());
        max_bottom = max_bottom.max(rect.bottom());
    }
    if boxes.is_empty() {
        return None;
    }
    Some((
        max_right.ceil().max(1.0) as u32,
        max_bottom.ceil().max(1.0) as u32,
    ))
}

/// Record `node` at `border_box`, then place its children. `nearest_positioned`
/// is the padding box of the nearest positioned ancestor — the containing block
/// for any in-flow descendant's absolutely positioned children inherit, while a
/// positioned node overrides it with its own padding box.
fn layout_subtree(
    node: &IrNode,
    border_box: BoxRect,
    nearest_positioned: BoxRect,
    children: &ChildIndex,
    out: &mut BTreeMap<String, BoxRect>,
) {
    out.insert(node.id.clone(), border_box);

    let content = content_box(node, border_box);
    // A positioned box is the containing block for its absolute children; an
    // in-flow box passes the inherited containing block through.
    let containing_block = if node.layout.position == Position::Absolute || node.parent_id.is_none()
    {
        padding_box(node, border_box)
    } else {
        nearest_positioned
    };

    let kids = children.of(&node.id);
    let in_flow: Vec<&IrNode> = kids
        .iter()
        .copied()
        .filter(|c| c.layout.position == Position::Static)
        .collect();
    let absolute: Vec<&IrNode> = kids
        .iter()
        .copied()
        .filter(|c| c.layout.position == Position::Absolute)
        .collect();

    match node.layout.display {
        Display::Block => layout_block(&in_flow, content, containing_block, children, out),
        Display::Flex | Display::InlineFlex => {
            layout_flex(node, &in_flow, content, containing_block, children, out)
        }
    }

    for child in absolute {
        let Some((w, h)) = explicit_border_box(child) else {
            continue;
        };
        let x = containing_block.x + child.layout.left.unwrap_or(0.0) + child.layout.margin.left;
        let y = containing_block.y + child.layout.top.unwrap_or(0.0) + child.layout.margin.top;
        let child_box = BoxRect {
            x,
            y,
            width: w,
            height: h,
        };
        layout_subtree(child, child_box, containing_block, children, out);
    }
}

/// Stack in-flow children vertically within `content`. A child with no explicit
/// width fills the content width; a child with no explicit height is
/// content-sized (text) and is skipped without advancing the flow cursor by an
/// unknown amount.
fn layout_block(
    in_flow: &[&IrNode],
    content: BoxRect,
    nearest_positioned: BoxRect,
    children: &ChildIndex,
    out: &mut BTreeMap<String, BoxRect>,
) {
    let mut cursor_y = content.y;
    for child in in_flow {
        let m = &child.layout.margin;
        let (explicit_w, explicit_h) = explicit_border_box_parts(child);
        let width = explicit_w.unwrap_or((content.width - m.left - m.right).max(0.0));
        let x = content.x + m.left;
        let y = cursor_y + m.top;
        match explicit_h {
            Some(height) => {
                let child_box = BoxRect {
                    x,
                    y,
                    width,
                    height,
                };
                layout_subtree(child, child_box, nearest_positioned, children, out);
                cursor_y = y + height + m.bottom;
            }
            None => {
                // Content-driven height: not resolvable without text. Advance by
                // the margins only so a following sibling is not double-counted.
                cursor_y = y + m.bottom;
            }
        }
    }
}

/// Lay out in-flow children of a flex container along its main axis with `gap`,
/// honoring the v0.1 subset: `align-items: center` on the cross axis and
/// `justify-content: center` on the main axis. Only children with explicit
/// width and height place; content-sized children are skipped.
fn layout_flex(
    container: &IrNode,
    in_flow: &[&IrNode],
    content: BoxRect,
    nearest_positioned: BoxRect,
    children: &ChildIndex,
    out: &mut BTreeMap<String, BoxRect>,
) {
    let direction = container
        .layout
        .flex_direction
        .clone()
        .unwrap_or(FlexDirection::Row);
    let gap = container.layout.gap.unwrap_or(0.0);
    let center_cross = container.layout.align_items == Some(AlignItems::Center);
    let center_main = container.layout.justify_content == Some(JustifyContent::Center);

    // Resolve sizes up front; a child needs both axes explicit to place.
    let sized: Vec<(&IrNode, f32, f32)> = in_flow
        .iter()
        .filter_map(|c| explicit_border_box(c).map(|(w, h)| (*c, w, h)))
        .collect();

    let total_main: f32 = sized
        .iter()
        .map(|(_, w, h)| main_of(&direction, *w, *h))
        .sum::<f32>()
        + gap * (sized.len().saturating_sub(1) as f32);

    let (content_main_start, content_main_size) = match direction {
        FlexDirection::Row => (content.x, content.width),
        FlexDirection::Column => (content.y, content.height),
    };
    let mut main = if center_main {
        content_main_start + (content_main_size - total_main) / 2.0
    } else {
        content_main_start
    };

    for (child, w, h) in sized {
        let main_size = main_of(&direction, w, h);
        let cross_size = cross_of(&direction, w, h);
        let (content_cross_start, content_cross_size) = match direction {
            FlexDirection::Row => (content.y, content.height),
            FlexDirection::Column => (content.x, content.width),
        };
        let cross = if center_cross {
            content_cross_start + (content_cross_size - cross_size) / 2.0
        } else {
            content_cross_start
        };
        let (x, y) = match direction {
            FlexDirection::Row => (main, cross),
            FlexDirection::Column => (cross, main),
        };
        let child_box = BoxRect {
            x,
            y,
            width: w,
            height: h,
        };
        layout_subtree(child, child_box, nearest_positioned, children, out);
        main += main_size + gap;
    }
}

fn main_of(direction: &FlexDirection, width: f32, height: f32) -> f32 {
    match direction {
        FlexDirection::Row => width,
        FlexDirection::Column => height,
    }
}

fn cross_of(direction: &FlexDirection, width: f32, height: f32) -> f32 {
    match direction {
        FlexDirection::Row => height,
        FlexDirection::Column => width,
    }
}

/// The top-left of a root's border box: its absolute offset (if positioned) plus
/// its top/left margin.
fn root_origin(root: &IrNode) -> (f32, f32) {
    let m = &root.layout.margin;
    match root.layout.position {
        Position::Absolute => (
            root.layout.left.unwrap_or(0.0) + m.left,
            root.layout.top.unwrap_or(0.0) + m.top,
        ),
        Position::Static => (m.left, m.top),
    }
}

/// The uniform border width of `node`, or zero.
fn border_width(node: &IrNode) -> f32 {
    node.paint.border.as_ref().map(|b| b.width).unwrap_or(0.0)
}

/// `node`'s padding box: its border box inset by the border on all sides.
fn padding_box(node: &IrNode, border_box: BoxRect) -> BoxRect {
    let b = border_width(node);
    border_box.inset(b, b, b, b)
}

/// `node`'s content box: its border box inset by border + padding per side.
fn content_box(node: &IrNode, border_box: BoxRect) -> BoxRect {
    let b = border_width(node);
    let p = &node.layout.padding;
    border_box.inset(b + p.top, b + p.right, b + p.bottom, b + p.left)
}

/// `node`'s explicit border-box size, requiring both axes. `None` if either is
/// content-driven.
fn explicit_border_box(node: &IrNode) -> Option<(f32, f32)> {
    match explicit_border_box_parts(node) {
        (Some(w), Some(h)) => Some((w, h)),
        _ => None,
    }
}

/// Each axis of `node`'s explicit border-box size independently, converting from
/// the content box when `box-sizing: content-box`.
fn explicit_border_box_parts(node: &IrNode) -> (Option<f32>, Option<f32>) {
    let l = &node.layout;
    let b = border_width(node);
    let (extra_w, extra_h) = match l.box_sizing {
        BoxSizing::BorderBox => (0.0, 0.0),
        BoxSizing::ContentBox => (
            l.padding.left + l.padding.right + 2.0 * b,
            l.padding.top + l.padding.bottom + 2.0 * b,
        ),
    };
    (l.width.map(|w| w + extra_w), l.height.map(|h| h + extra_h))
}

/// Child lookup preserving the IR's pre-order sibling order.
struct ChildIndex<'a> {
    by_parent: BTreeMap<&'a str, Vec<&'a IrNode>>,
}

impl<'a> ChildIndex<'a> {
    fn build(scene: &'a SceneIr) -> Self {
        let mut by_parent: BTreeMap<&'a str, Vec<&'a IrNode>> = BTreeMap::new();
        for node in &scene.nodes {
            if let Some(parent) = node.parent_id.as_deref() {
                by_parent.entry(parent).or_default().push(node);
            }
        }
        Self { by_parent }
    }

    fn of(&self, id: &str) -> &[&'a IrNode] {
        self.by_parent.get(id).map(Vec::as_slice).unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests;
