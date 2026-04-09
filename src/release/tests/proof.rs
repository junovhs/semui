//! Release proof tests — final acceptance gate for v0.1.

use std::path::PathBuf;

use crate::ir::SceneIr;
use crate::release::{run_corpus_proof, write_golden_artifacts};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// ---------------------------------------------------------------------------
// Corpus-wide acceptance gate
// ---------------------------------------------------------------------------

/// Every scene in the v0.1 manifest must pass its round-trip with zero drift.
#[test]
fn all_v01_scenes_pass_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let proof = run_corpus_proof(repo_root())?;

    let failures: Vec<_> = proof
        .scenes
        .iter()
        .filter(|s| !s.round_trip_pass)
        .collect();

    assert!(
        failures.is_empty(),
        "round-trip failed for {} scene(s):\n{}",
        failures.len(),
        failures
            .iter()
            .flat_map(|s| {
                let id = &s.scene_id;
                s.drift.iter().map(move |d| format!("  [{id}] {d}"))
            })
            .collect::<Vec<_>>()
            .join("\n")
    );
    Ok(())
}

/// Corpus must cover all 5 scenes defined in the manifest.
#[test]
fn corpus_proof_covers_all_five_scenes() -> Result<(), Box<dyn std::error::Error>> {
    let proof = run_corpus_proof(repo_root())?;
    assert_eq!(proof.scenes.len(), 5, "expected 5 scenes in the v0.1 corpus");
    Ok(())
}

/// Total IR node count must be positive (sanity check).
#[test]
fn corpus_has_nonzero_ir_nodes() -> Result<(), Box<dyn std::error::Error>> {
    let proof = run_corpus_proof(repo_root())?;
    assert!(proof.total_ir_nodes() > 0);
    Ok(())
}

// ---------------------------------------------------------------------------
// Per-scene golden artifact validation
// ---------------------------------------------------------------------------

/// Golden scene.semui.json must be valid JSON that round-trips through serde.
#[test]
fn golden_semui_json_is_deserializable_for_all_scenes() -> Result<(), Box<dyn std::error::Error>> {
    let scene_ids = [
        "profile_card_absolute",
        "stacked_info_card",
        "action_row_variants",
        "nested_panel_inset",
        "typography_specimen",
    ];
    for scene_id in scene_ids {
        let ir = write_golden_artifacts(repo_root(), scene_id)?;
        // Serialize → deserialize round-trip is lossless
        let json = ir.to_json()?;
        let ir2 = SceneIr::from_json(&json)?;
        assert_eq!(ir.schema_version, ir2.schema_version);
        assert_eq!(ir.scene_id, ir2.scene_id);
        assert_eq!(ir.nodes.len(), ir2.nodes.len());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Supported subset assertions
// ---------------------------------------------------------------------------

/// Verify that the supported property set is non-empty and the expected
/// high-value properties are in it.
#[test]
fn supported_subset_includes_core_properties() {
    use crate::diagnostics::analyze;
    use crate::source_graph::{CssDeclaration, CssRule, SceneSourceGraph, TraceSpan};
    use std::path::PathBuf;

    fn zero_span() -> TraceSpan {
        TraceSpan { start: 0, end: 0, line: 1, column: 1 }
    }

    // Check that a rule with only core properties produces no diagnostics.
    let core_props = [
        ("display", "flex"),
        ("position", "absolute"),
        ("width", "100px"),
        ("height", "50px"),
        ("background-color", "#fff"),
        ("border-radius", "8px"),
        ("font-size", "16px"),
        ("font-weight", "400"),
        ("color", "#000"),
    ];
    let decls: Vec<CssDeclaration> = core_props
        .iter()
        .map(|(p, v)| CssDeclaration {
            property: p.to_string(),
            value: v.to_string(),
            span: zero_span(),
        })
        .collect();
    let graph = SceneSourceGraph {
        scene_id: "test".to_owned(),
        scene_root: PathBuf::from("."),
        html: crate::source_graph::SourceDocument {
            id: 0,
            kind: crate::source_graph::SourceDocumentKind::Html,
            path: PathBuf::from("test.html"),
            contents: String::new(),
        },
        css: crate::source_graph::SourceDocument {
            id: 1,
            kind: crate::source_graph::SourceDocumentKind::Css,
            path: PathBuf::from("test.css"),
            contents: String::new(),
        },
        html_nodes: Vec::new(),
        css_rules: vec![CssRule {
            id: 0,
            selectors: vec![".foo".to_owned()],
            declarations: decls,
            span: zero_span(),
            document_id: 1,
        }],
    };

    let diags = analyze(&graph);
    assert!(
        diags.is_empty(),
        "core supported properties must not produce diagnostics: {diags:?}"
    );
}
