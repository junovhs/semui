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
use crate::ir::{IrNode, NodeKind, SceneIr};
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

    if ir1.nodes.len() != ir2.nodes.len() {
        drift.push(Drift {
            message: format!(
                "node count: pass1={} pass2={}",
                ir1.nodes.len(),
                ir2.nodes.len()
            ),
        });
        // Can't compare per-node if counts differ
        return drift;
    }

    for (idx, (n1, n2)) in ir1.nodes.iter().zip(ir2.nodes.iter()).enumerate() {
        compare_node(n1, n2, idx, &mut drift);
    }

    drift
}

fn compare_node(n1: &IrNode, n2: &IrNode, idx: usize, drift: &mut Vec<Drift>) {
    if n1.kind != n2.kind {
        drift.push(Drift {
            message: format!("node[{idx}]: kind {:?} → {:?}", n1.kind, n2.kind),
        });
    }

    if n1.kind == NodeKind::Text {
        if n1.text_content != n2.text_content {
            drift.push(Drift {
                message: format!(
                    "node[{idx}]: text_content {:?} → {:?}",
                    n1.text_content, n2.text_content
                ),
            });
        }
        return;
    }

    // Layout
    compare_opt_f32(n1.layout.width, n2.layout.width, idx, "layout.width", drift);
    compare_opt_f32(n1.layout.height, n2.layout.height, idx, "layout.height", drift);
    compare_opt_f32(n1.layout.top, n2.layout.top, idx, "layout.top", drift);
    compare_opt_f32(n1.layout.left, n2.layout.left, idx, "layout.left", drift);

    // Paint
    if n1.paint.background_color != n2.paint.background_color {
        drift.push(Drift {
            message: format!(
                "node[{idx}]: background_color {:?} → {:?}",
                n1.paint.background_color, n2.paint.background_color
            ),
        });
    }
    if n1.paint.border_radius != n2.paint.border_radius {
        drift.push(Drift {
            message: format!(
                "node[{idx}]: border_radius {:?} → {:?}",
                n1.paint.border_radius, n2.paint.border_radius
            ),
        });
    }

    // Typography
    if n1.typography != n2.typography {
        drift.push(Drift {
            message: format!("node[{idx}]: typography differs"),
        });
    }
}

fn compare_opt_f32(
    a: Option<f32>,
    b: Option<f32>,
    idx: usize,
    field: &str,
    drift: &mut Vec<Drift>,
) {
    if a != b {
        drift.push(Drift {
            message: format!("node[{idx}]: {field} {a:?} → {b:?}"),
        });
    }
}
