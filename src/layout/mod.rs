//! Layout and geometry computation for the SEMUI v0.1 static subset.
//!
//! Takes a [`ResolvedScene`] (CSS cascade output from `CAS-01`) and produces
//! a [`LaidOutScene`]: every element node paired with its explicit geometry.
//!
//! # v0.1 scope
//!
//! All sizing in the v0.1 fixture corpus is stated in explicit `px` values.
//! There are no percentage widths, no `auto` on sized elements, and no
//! font-metric-dependent intrinsic sizing. Geometry computation therefore
//! reduces to two operations:
//!
//! 1. **Absolute coordinate extraction** — for `position: absolute` nodes,
//!    copy `left`/`top` from the computed style as explicit x/y.
//! 2. **Box-sizing normalization** — for `box-sizing: border-box` nodes,
//!    subtract padding and border from the declared width/height to obtain
//!    the inner content-box dimensions.
//!
//! Elements whose size cannot be determined without browser rendering (e.g.
//! auto-sized flex items with text content) produce `None` for the relevant
//! dimension fields. `DIA-01` reports these as unresolved geometry.

pub use model::{Geometry, LaidOutNode, LaidOutScene};

use crate::resolver::{ComputedStyle, ResolvedScene};

mod model;

#[cfg(test)]
mod tests;

/// Compute explicit geometry for every element node in `resolved`.
///
/// This operation is infallible: every node produces a [`Geometry`], though
/// some fields may be `None` when the dimension cannot be determined from
/// CSS alone.
pub fn compute_layout(resolved: &ResolvedScene) -> LaidOutScene {
    let nodes = resolved
        .nodes
        .iter()
        .map(|rn| LaidOutNode {
            node: rn.node.clone(),
            geometry: node_geometry(&rn.style),
            style: rn.style.clone(),
        })
        .collect();

    LaidOutScene {
        scene_id: resolved.scene_id.clone(),
        nodes,
    }
}

fn node_geometry(style: &ComputedStyle) -> Geometry {
    let (content_width, content_height) = content_box(style);

    Geometry {
        explicit_x: absolute_coord(&style.position, style.left),
        explicit_y: absolute_coord(&style.position, style.top),
        width: style.width,
        height: style.height,
        min_width: style.min_width,
        content_width,
        content_height,
        margin: [
            style.margin_top,
            style.margin_right,
            style.margin_bottom,
            style.margin_left,
        ],
        padding: [
            style.padding_top,
            style.padding_right,
            style.padding_bottom,
            style.padding_left,
        ],
        border_width: style.border_width,
        border_radius: style.border_radius,
    }
}

/// Return the explicit coordinate only for absolutely positioned elements.
/// Flow and flex elements have no resolved origin in v0.1.
fn absolute_coord(position: &str, coord: Option<f32>) -> Option<f32> {
    if position == "absolute" { coord } else { None }
}

/// Compute content-box width and height after subtracting padding and border
/// for `border-box` elements. Content-box elements return the declared value
/// unchanged.
fn content_box(style: &ComputedStyle) -> (Option<f32>, Option<f32>) {
    if style.box_sizing != "border-box" {
        return (style.width, style.height);
    }

    let h_inset = style.padding_left + style.padding_right + style.border_width * 2.0;
    let v_inset = style.padding_top + style.padding_bottom + style.border_width * 2.0;

    let cw = style.width.map(|w| (w - h_inset).max(0.0));
    let ch = style.height.map(|h| (h - v_inset).max(0.0));

    (cw, ch)
}
