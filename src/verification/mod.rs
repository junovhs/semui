//! Round-trip regression harness for the v0.1 fixture corpus.
//!
//! The canonical round-trip is:
//! ```text
//! Source HTML/CSS  → resolve → lay out → extract → SceneIr (pass 1)
//!                                                       ↓ emit
//!                                                 HTML/CSS strings
//!                                                       ↓ re-parse
//! Emitted HTML/CSS → resolve → lay out → extract → SceneIr (pass 2)
//! ```
//!
//! [`verify_round_trip`] runs both passes and returns a [`VerificationResult`]
//! describing structural and semantic equivalence.

#[cfg(test)]
mod tests;

use crate::emitter::emit;
use crate::extractor::extract_ir;
use std::fmt::Debug;

use crate::ir::{IrNode, SceneIr};
use crate::layout::compute_layout;
use crate::resolver::resolve_scene;
use crate::source_graph::SceneSourceGraph;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// A single structural or semantic difference between pass-1 and pass-2 IR.
#[derive(Debug, Clone, PartialEq)]
pub struct Drift {
    pub message: String,
}

/// The result of running a full round-trip on one scene.
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub scene_id: String,
    pub pass: bool,
    pub drift: Vec<Drift>,
    pub pass1_node_count: usize,
    pub pass2_node_count: usize,
}

/// Run the full round-trip on `graph` and return the comparison result.
///
/// Returns `Err` only if a pipeline step (resolve, extract, re-parse) fails.
/// Structural differences are captured as [`Drift`] items rather than errors.
pub fn verify_round_trip(
    graph: &SceneSourceGraph,
) -> Result<VerificationResult, Box<dyn std::error::Error>> {
    // Pass 1: source → IR
    let resolved1 = resolve_scene(graph)?;
    let laid_out1 = compute_layout(&resolved1);
    let ir1 = extract_ir(&laid_out1, graph)?;

    // Emit pass-1 IR to HTML + CSS strings
    let emitted = emit(&ir1);

    // Re-parse emitted output
    let graph2 = SceneSourceGraph::from_strings(&graph.scene_id, &emitted.html, &emitted.css)?;

    // Pass 2: emitted HTML/CSS → IR
    let resolved2 = resolve_scene(&graph2)?;
    let laid_out2 = compute_layout(&resolved2);
    let ir2 = extract_ir(&laid_out2, &graph2)?;

    let drift = compare_ir(&ir1, &ir2);
    let pass = drift.is_empty();

    Ok(VerificationResult {
        scene_id: graph.scene_id.clone(),
        pass,
        drift,
        pass1_node_count: ir1.nodes.len(),
        pass2_node_count: ir2.nodes.len(),
    })
}

// ---------------------------------------------------------------------------
// IR comparison
// ---------------------------------------------------------------------------

fn compare_ir(ir1: &SceneIr, ir2: &SceneIr) -> Vec<Drift> {
    let mut drift: Vec<Drift> = Vec::new();

    compare_value(
        &ir1.schema_version,
        &ir2.schema_version,
        "scene.schema_version",
        &mut drift,
    );
    compare_value(&ir1.scene_id, &ir2.scene_id, "scene.scene_id", &mut drift);
    compare_value(&ir1.corpus, &ir2.corpus, "scene.corpus", &mut drift);
    compare_value(
        &ir1.execution_mode,
        &ir2.execution_mode,
        "scene.execution_mode",
        &mut drift,
    );

    if ir1.nodes.len() != ir2.nodes.len() {
        drift.push(Drift {
            message: format!(
                "node count: pass1={} pass2={}",
                ir1.nodes.len(),
                ir2.nodes.len()
            ),
        });
    }

    for (idx, (n1, n2)) in ir1.nodes.iter().zip(ir2.nodes.iter()).enumerate() {
        compare_node(n1, n2, idx, &mut drift);
    }

    drift
}

fn compare_node(n1: &IrNode, n2: &IrNode, idx: usize, drift: &mut Vec<Drift>) {
    let node = format!("node[{idx}]");
    macro_rules! field {
        ($path:literal, $left:expr, $right:expr) => {
            compare_value(&$left, &$right, &format!("{node}.{}", $path), drift)
        };
    }

    field!("id", n1.id, n2.id);
    field!("kind", n1.kind, n2.kind);
    field!("parent_id", n1.parent_id, n2.parent_id);
    field!("control_kind", n1.control_kind, n2.control_kind);
    field!("text_content", n1.text_content, n2.text_content);

    // Layout
    field!("layout.position", n1.layout.position, n2.layout.position);
    field!("layout.display", n1.layout.display, n2.layout.display);
    field!(
        "layout.box_sizing",
        n1.layout.box_sizing,
        n2.layout.box_sizing
    );
    field!("layout.top", n1.layout.top, n2.layout.top);
    field!("layout.left", n1.layout.left, n2.layout.left);
    field!("layout.width", n1.layout.width, n2.layout.width);
    field!("layout.height", n1.layout.height, n2.layout.height);
    field!("layout.min_width", n1.layout.min_width, n2.layout.min_width);
    field!(
        "layout.margin.top",
        n1.layout.margin.top,
        n2.layout.margin.top
    );
    field!(
        "layout.margin.right",
        n1.layout.margin.right,
        n2.layout.margin.right
    );
    field!(
        "layout.margin.bottom",
        n1.layout.margin.bottom,
        n2.layout.margin.bottom
    );
    field!(
        "layout.margin.left",
        n1.layout.margin.left,
        n2.layout.margin.left
    );
    field!(
        "layout.padding.top",
        n1.layout.padding.top,
        n2.layout.padding.top
    );
    field!(
        "layout.padding.right",
        n1.layout.padding.right,
        n2.layout.padding.right
    );
    field!(
        "layout.padding.bottom",
        n1.layout.padding.bottom,
        n2.layout.padding.bottom
    );
    field!(
        "layout.padding.left",
        n1.layout.padding.left,
        n2.layout.padding.left
    );
    field!(
        "layout.flex_direction",
        n1.layout.flex_direction,
        n2.layout.flex_direction
    );
    field!(
        "layout.align_items",
        n1.layout.align_items,
        n2.layout.align_items
    );
    field!(
        "layout.justify_content",
        n1.layout.justify_content,
        n2.layout.justify_content
    );
    field!(
        "layout.align_self",
        n1.layout.align_self,
        n2.layout.align_self
    );
    field!("layout.gap", n1.layout.gap, n2.layout.gap);

    // Paint
    field!(
        "paint.background_color",
        n1.paint.background_color,
        n2.paint.background_color
    );
    match (&n1.paint.border, &n2.paint.border) {
        (Some(b1), Some(b2)) => {
            field!("paint.border.width", b1.width, b2.width);
            field!("paint.border.color", b1.color, b2.color);
        }
        _ => field!("paint.border", n1.paint.border, n2.paint.border),
    }
    field!(
        "paint.border_radius",
        n1.paint.border_radius,
        n2.paint.border_radius
    );
    field!("paint.cursor", n1.paint.cursor, n2.paint.cursor);

    // Typography
    match (&n1.typography, &n2.typography) {
        (Some(t1), Some(t2)) => {
            field!("typography.font_family", t1.font_family, t2.font_family);
            field!("typography.font_size", t1.font_size, t2.font_size);
            field!("typography.font_weight", t1.font_weight, t2.font_weight);
            field!("typography.line_height", t1.line_height, t2.line_height);
            field!("typography.color", t1.color, t2.color);
        }
        _ => field!("typography", n1.typography, n2.typography),
    }

    // `source` is deliberately excluded: it records parser provenance and is
    // expected to change when emitted HTML/CSS is parsed for the second pass.
}

fn compare_value<T: Debug + PartialEq>(a: &T, b: &T, field: &str, drift: &mut Vec<Drift>) {
    if a != b {
        drift.push(Drift {
            message: format!("{field}: {a:?} → {b:?}"),
        });
    }
}
