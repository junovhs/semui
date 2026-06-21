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

/// Evidence for one internal round-trip gate.
#[derive(Debug, Clone)]
pub struct VerificationGateResult {
    pub pass: bool,
    pub drift: Vec<Drift>,
}

/// The result of running a full round-trip on one scene.
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub scene_id: String,
    pub structural: VerificationGateResult,
    pub semantic_ir: VerificationGateResult,
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

    let comparison = compare_ir(&ir1, &ir2);

    Ok(VerificationResult {
        scene_id: graph.scene_id.clone(),
        structural: VerificationGateResult {
            pass: comparison.structural_drift.is_empty(),
            drift: comparison.structural_drift,
        },
        semantic_ir: VerificationGateResult {
            pass: comparison.semantic_ir_drift.is_empty(),
            drift: comparison.semantic_ir_drift,
        },
        pass1_node_count: ir1.nodes.len(),
        pass2_node_count: ir2.nodes.len(),
    })
}

// ---------------------------------------------------------------------------
// IR comparison
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct IrComparison {
    structural_drift: Vec<Drift>,
    semantic_ir_drift: Vec<Drift>,
}

fn compare_ir(ir1: &SceneIr, ir2: &SceneIr) -> IrComparison {
    let mut comparison = IrComparison {
        structural_drift: Vec::new(),
        semantic_ir_drift: Vec::new(),
    };

    compare_value(
        &ir1.schema_version,
        &ir2.schema_version,
        "scene.schema_version",
        &mut comparison.semantic_ir_drift,
    );
    compare_value(
        &ir1.scene_id,
        &ir2.scene_id,
        "scene.scene_id",
        &mut comparison.structural_drift,
    );
    compare_value(
        &ir1.corpus,
        &ir2.corpus,
        "scene.corpus",
        &mut comparison.semantic_ir_drift,
    );
    compare_value(
        &ir1.execution_mode,
        &ir2.execution_mode,
        "scene.execution_mode",
        &mut comparison.semantic_ir_drift,
    );

    if ir1.nodes.len() != ir2.nodes.len() {
        comparison.structural_drift.push(Drift {
            message: format!(
                "node count: pass1={} pass2={}",
                ir1.nodes.len(),
                ir2.nodes.len()
            ),
        });
    }

    for (idx, (n1, n2)) in ir1.nodes.iter().zip(ir2.nodes.iter()).enumerate() {
        compare_node(n1, n2, idx, &mut comparison);
    }

    comparison
}

fn compare_node(n1: &IrNode, n2: &IrNode, idx: usize, comparison: &mut IrComparison) {
    let node = format!("node[{idx}]");
    macro_rules! structural {
        ($path:literal, $left:expr, $right:expr) => {
            compare_value(
                &$left,
                &$right,
                &format!("{node}.{}", $path),
                &mut comparison.structural_drift,
            )
        };
    }
    macro_rules! semantic {
        ($path:literal, $left:expr, $right:expr) => {
            compare_value(
                &$left,
                &$right,
                &format!("{node}.{}", $path),
                &mut comparison.semantic_ir_drift,
            )
        };
    }

    structural!("id", n1.id, n2.id);
    structural!("kind", n1.kind, n2.kind);
    structural!("parent_id", n1.parent_id, n2.parent_id);
    structural!("control_kind", n1.control_kind, n2.control_kind);
    structural!("text_content", n1.text_content, n2.text_content);

    // Layout
    semantic!("layout.position", n1.layout.position, n2.layout.position);
    semantic!("layout.display", n1.layout.display, n2.layout.display);
    semantic!(
        "layout.box_sizing",
        n1.layout.box_sizing,
        n2.layout.box_sizing
    );
    semantic!("layout.top", n1.layout.top, n2.layout.top);
    semantic!("layout.left", n1.layout.left, n2.layout.left);
    semantic!("layout.width", n1.layout.width, n2.layout.width);
    semantic!("layout.height", n1.layout.height, n2.layout.height);
    semantic!("layout.min_width", n1.layout.min_width, n2.layout.min_width);
    semantic!(
        "layout.margin.top",
        n1.layout.margin.top,
        n2.layout.margin.top
    );
    semantic!(
        "layout.margin.right",
        n1.layout.margin.right,
        n2.layout.margin.right
    );
    semantic!(
        "layout.margin.bottom",
        n1.layout.margin.bottom,
        n2.layout.margin.bottom
    );
    semantic!(
        "layout.margin.left",
        n1.layout.margin.left,
        n2.layout.margin.left
    );
    semantic!(
        "layout.padding.top",
        n1.layout.padding.top,
        n2.layout.padding.top
    );
    semantic!(
        "layout.padding.right",
        n1.layout.padding.right,
        n2.layout.padding.right
    );
    semantic!(
        "layout.padding.bottom",
        n1.layout.padding.bottom,
        n2.layout.padding.bottom
    );
    semantic!(
        "layout.padding.left",
        n1.layout.padding.left,
        n2.layout.padding.left
    );
    semantic!(
        "layout.flex_direction",
        n1.layout.flex_direction,
        n2.layout.flex_direction
    );
    semantic!(
        "layout.align_items",
        n1.layout.align_items,
        n2.layout.align_items
    );
    semantic!(
        "layout.justify_content",
        n1.layout.justify_content,
        n2.layout.justify_content
    );
    semantic!(
        "layout.align_self",
        n1.layout.align_self,
        n2.layout.align_self
    );
    semantic!("layout.gap", n1.layout.gap, n2.layout.gap);

    // Paint
    semantic!(
        "paint.background_color",
        n1.paint.background_color,
        n2.paint.background_color
    );
    match (&n1.paint.border, &n2.paint.border) {
        (Some(b1), Some(b2)) => {
            semantic!("paint.border.width", b1.width, b2.width);
            semantic!("paint.border.color", b1.color, b2.color);
        }
        _ => semantic!("paint.border", n1.paint.border, n2.paint.border),
    }
    semantic!(
        "paint.border_radius",
        n1.paint.border_radius,
        n2.paint.border_radius
    );
    semantic!("paint.cursor", n1.paint.cursor, n2.paint.cursor);

    // Typography
    match (&n1.typography, &n2.typography) {
        (Some(t1), Some(t2)) => {
            semantic!("typography.font_family", t1.font_family, t2.font_family);
            semantic!("typography.font_size", t1.font_size, t2.font_size);
            semantic!("typography.font_weight", t1.font_weight, t2.font_weight);
            semantic!("typography.line_height", t1.line_height, t2.line_height);
            semantic!("typography.color", t1.color, t2.color);
        }
        _ => semantic!("typography", n1.typography, n2.typography),
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
