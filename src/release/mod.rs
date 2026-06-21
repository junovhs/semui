//! v0.1 release proof: run the full corpus, generate golden artifacts, and
//! capture the evidence that round-trip fidelity is within budget.
//!
//! The main entry point is [`run_corpus_proof`], which returns a
//! [`CorpusProof`] summarising every scene in the manifest.

#[cfg(test)]
mod tests;

use std::path::Path;

use crate::diagnostics::{Diagnostic, DiagnosticKind, analyze};
use crate::emitter::{EmittedScene, emit};
use crate::extractor::extract_ir;
use crate::ir::SceneIr;
use crate::layout::compute_layout;
use crate::load_scene_source_graph;
use crate::resolver::resolve_scene;
use crate::verification::verify_round_trip;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Per-scene proof evidence.
#[derive(Debug, Clone)]
pub struct SceneProof {
    pub scene_id: String,
    /// Number of nodes in the extracted IR.
    pub ir_node_count: usize,
    /// Number of diagnostics produced by the static analyzer.
    pub diagnostic_count: usize,
    /// Whether the HTML/CSS → IR → HTML/CSS round-trip was lossless.
    pub round_trip_pass: bool,
    /// Drift messages when `round_trip_pass` is false.
    pub drift: Vec<String>,
}

/// Aggregate proof for the entire v0.1 corpus.
#[derive(Debug, Clone)]
pub struct CorpusProof {
    pub corpus: String,
    pub scenes: Vec<SceneProof>,
}

impl CorpusProof {
    /// `true` when every scene passes its round-trip check.
    pub fn all_pass(&self) -> bool {
        self.scenes.iter().all(|s| s.round_trip_pass)
    }

    /// Total IR nodes across all scenes.
    pub fn total_ir_nodes(&self) -> usize {
        self.scenes.iter().map(|s| s.ir_node_count).sum()
    }
}

// ---------------------------------------------------------------------------
// Proof runner
// ---------------------------------------------------------------------------

/// Run the full pipeline for every scene in the manifest and collect evidence.
pub fn run_corpus_proof(
    repo_root: impl AsRef<Path>,
) -> Result<CorpusProof, Box<dyn std::error::Error>> {
    let manifest_path = repo_root
        .as_ref()
        .join("fixtures")
        .join("v0.1")
        .join("manifest.toml");
    let manifest = crate::load_fixture_manifest(&manifest_path)?;

    let mut scenes: Vec<SceneProof> = Vec::new();

    for scene in &manifest.scenes {
        let scene_id = scene.id.clone();
        let graph = load_scene_source_graph(repo_root.as_ref(), &scene_id)?;

        let resolved = resolve_scene(&graph)?;
        let laid_out = compute_layout(&resolved);
        let ir = extract_ir(&laid_out, &graph)?;

        let diagnostics = analyze(&graph);
        let expected_diagnostics_path = repo_root
            .as_ref()
            .join("fixtures")
            .join("v0.1")
            .join(&scene.dir)
            .join("expected")
            .join("diagnostics.txt");
        let expected_diagnostics = std::fs::read_to_string(&expected_diagnostics_path)?;
        validate_diagnostic_expectations(&scene_id, &diagnostics, &expected_diagnostics)
            .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidData, message))?;
        let diagnostic_count = diagnostics.len();

        let vr = verify_round_trip(&graph)?;

        scenes.push(SceneProof {
            scene_id,
            ir_node_count: ir.nodes.len(),
            diagnostic_count,
            round_trip_pass: vr.pass,
            drift: vr.drift.iter().map(|d| d.message.clone()).collect(),
        });
    }

    Ok(CorpusProof {
        corpus: "v0.1".to_owned(),
        scenes,
    })
}

fn diagnostic_key(diagnostic: &Diagnostic) -> String {
    match diagnostic.kind {
        DiagnosticKind::UnsupportedProperty => format!(
            "unsupported-property:{}",
            diagnostic.property.as_deref().unwrap_or("<missing>")
        ),
        DiagnosticKind::UnsupportedValue => format!(
            "unsupported-value:{}={}",
            diagnostic.property.as_deref().unwrap_or("<missing>"),
            diagnostic.value.as_deref().unwrap_or("<missing>")
        ),
        DiagnosticKind::UnsupportedSelector => format!(
            "unsupported-selector:{}",
            diagnostic.selector.as_deref().unwrap_or("<missing>")
        ),
    }
}

fn validate_diagnostic_expectations(
    scene_id: &str,
    diagnostics: &[Diagnostic],
    expected_file: &str,
) -> Result<(), String> {
    let mut expected: Vec<String> = expected_file
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect();
    let mut actual: Vec<String> = diagnostics.iter().map(diagnostic_key).collect();
    expected.sort();
    actual.sort();

    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "diagnostic contract failed for '{scene_id}': expected {expected:?}, actual {actual:?}"
        ))
    }
}

// ---------------------------------------------------------------------------
// Golden artifact generation
// ---------------------------------------------------------------------------

/// Run the pipeline for `scene_id` and return its IR plus emitted artifacts
/// **without touching the filesystem**. This is the read-only core shared by
/// verification (which compares against committed goldens) and the explicit
/// golden-writing maintenance step.
pub fn build_golden_artifacts(
    repo_root: impl AsRef<Path>,
    scene_id: &str,
) -> Result<(SceneIr, EmittedScene), Box<dyn std::error::Error>> {
    let root = repo_root.as_ref();
    let graph = load_scene_source_graph(root, scene_id)?;
    let resolved = resolve_scene(&graph)?;
    let laid_out = compute_layout(&resolved);
    let ir = extract_ir(&laid_out, &graph)?;
    let emitted = emit(&ir);
    Ok((ir, emitted))
}

/// Generate and write golden artifacts for `scene_id` to the fixture's
/// `expected/` directory.  Overwrites existing files.
///
/// This is an explicit maintenance operation, separate from verification, and
/// must never run as part of a normal `cargo test`. See
/// `docs/v0.1-acceptance-gate.md`.
pub fn write_golden_artifacts(
    repo_root: impl AsRef<Path>,
    scene_id: &str,
) -> Result<SceneIr, Box<dyn std::error::Error>> {
    let root = repo_root.as_ref();
    let (ir, emitted) = build_golden_artifacts(root, scene_id)?;

    let expected_dir = root
        .join("fixtures")
        .join("v0.1")
        .join(scene_id)
        .join("expected");

    std::fs::write(expected_dir.join("scene.semui.json"), ir.to_json()?)?;
    std::fs::write(expected_dir.join("roundtrip.html"), &emitted.html)?;
    std::fs::write(expected_dir.join("roundtrip.css"), &emitted.css)?;

    Ok(ir)
}
