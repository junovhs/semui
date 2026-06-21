//! Release proof tests — final acceptance gate for v0.1.

use std::path::PathBuf;

use crate::diagnostics::{Diagnostic, DiagnosticKind};
use crate::ir::SceneIr;
use crate::release::{
    GateEvidence, GateStatus, SceneGates, SceneProof, build_golden_artifacts, run_corpus_proof,
    write_golden_artifacts,
};
use crate::source_graph::load_scene_source_graph;
use crate::verification::verify_round_trip;

use super::super::validate_diagnostic_expectations;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Normalize line endings so comparisons are stable across platforms and the
/// committed golden's checkout encoding.
fn norm(s: &str) -> String {
    s.replace("\r\n", "\n")
}

fn fixture_scene_ids(
    required_tag: Option<&str>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let manifest = crate::load_fixture_manifest(
        repo_root()
            .join("fixtures")
            .join("v0.1")
            .join("manifest.toml"),
    )?;
    Ok(manifest
        .scenes
        .iter()
        .filter(|scene| {
            required_tag.is_none_or(|tag| scene.tags.iter().any(|candidate| candidate == tag))
        })
        .map(|scene| scene.id.clone())
        .collect())
}

// ---------------------------------------------------------------------------
// Corpus-wide acceptance gate
// ---------------------------------------------------------------------------

/// Every scene in the v0.1 manifest must pass the implemented internal gates.
#[test]
fn all_v01_scenes_pass_internal_gates() -> Result<(), Box<dyn std::error::Error>> {
    let proof = run_corpus_proof(repo_root())?;

    let failures: Vec<_> = proof
        .scenes
        .iter()
        .filter(|scene| {
            scene.gates.structural.status != GateStatus::Pass
                || scene.gates.semantic_ir.status != GateStatus::Pass
                || scene.gates.diagnostics.status != GateStatus::Pass
        })
        .collect();

    assert!(
        failures.is_empty(),
        "internal gates failed for {} scene(s): {failures:#?}",
        failures.len(),
    );
    Ok(())
}

#[test]
fn corpus_status_is_unavailable_until_browser_gates_exist() -> Result<(), Box<dyn std::error::Error>>
{
    let proof = run_corpus_proof(repo_root())?;
    assert_eq!(proof.status(), GateStatus::Unavailable);
    for scene in &proof.scenes {
        assert_eq!(scene.gates.computed_style.status, GateStatus::Unavailable);
        assert_eq!(scene.gates.geometry.status, GateStatus::Unavailable);
        assert_eq!(scene.gates.visual.status, GateStatus::Unavailable);
        assert_eq!(scene.status(), GateStatus::Unavailable);
    }
    Ok(())
}

#[test]
fn failing_gate_takes_precedence_over_unavailable_evidence() {
    let scene = SceneProof {
        scene_id: "test".to_owned(),
        ir_node_count: 1,
        diagnostic_count: 0,
        gates: SceneGates {
            structural: GateEvidence::pass(),
            semantic_ir: GateEvidence::pass(),
            diagnostics: GateEvidence::pass(),
            computed_style: GateEvidence::unavailable("not captured"),
            geometry: GateEvidence::unavailable("not captured"),
            visual: GateEvidence::fail(vec!["pixel drift".to_owned()]),
        },
    };

    assert_eq!(scene.status(), GateStatus::Fail);
}

#[test]
fn corpus_evidence_json_is_deterministic_and_names_every_gate()
-> Result<(), Box<dyn std::error::Error>> {
    let proof = run_corpus_proof(repo_root())?;
    let first = proof.to_json()?;
    let second = proof.to_json()?;
    assert_eq!(first, second);

    let value: serde_json::Value = serde_json::from_str(&first)?;
    assert_eq!(value["status"], "unavailable");
    assert_eq!(value["scenes"][0]["status"], "unavailable");
    let gates = &value["scenes"][0]["gates"];
    for name in [
        "structural",
        "semantic_ir",
        "diagnostics",
        "computed_style",
        "geometry",
        "visual",
    ] {
        assert!(gates.get(name).is_some(), "missing gate {name}: {gates}");
    }
    Ok(())
}

/// Corpus proof covers every fixture explicitly tagged for browser evidence.
#[test]
fn corpus_proof_covers_all_browser_scenes() -> Result<(), Box<dyn std::error::Error>> {
    let proof = run_corpus_proof(repo_root())?;
    assert_eq!(
        proof.scenes.len(),
        fixture_scene_ids(Some("browser"))?.len()
    );
    assert!(proof.scenes.len() >= 6, "canonical browser corpus shrank");
    Ok(())
}

/// Total IR node count must be positive (sanity check).
#[test]
fn corpus_has_nonzero_ir_nodes() -> Result<(), Box<dyn std::error::Error>> {
    let proof = run_corpus_proof(repo_root())?;
    assert!(proof.total_ir_nodes() > 0);
    Ok(())
}

#[test]
fn unexpected_diagnostic_fails_the_fixture_contract() {
    let diagnostic = Diagnostic {
        kind: DiagnosticKind::UnsupportedValue,
        message: "unsupported display".to_owned(),
        rule_id: 0,
        selector: Some(".hidden".to_owned()),
        property: Some("display".to_owned()),
        value: Some("none".to_owned()),
    };

    let error = validate_diagnostic_expectations("test", &[diagnostic], "")
        .expect_err("an undocumented diagnostic must fail release proof");
    assert!(error.contains("unsupported-value:display=none"));
}

#[test]
fn every_fixture_declares_and_meets_expected_gate_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::diagnostics::analyze;

    let root = repo_root();
    let manifest =
        crate::load_fixture_manifest(root.join("fixtures").join("v0.1").join("manifest.toml"))?;
    for scene in &manifest.scenes {
        let graph = load_scene_source_graph(&root, &scene.id)?;
        let expected_root = root
            .join("fixtures")
            .join("v0.1")
            .join(&scene.dir)
            .join("expected");
        let diagnostics = analyze(&graph);
        let expected_diagnostics = std::fs::read_to_string(expected_root.join("diagnostics.txt"))?;
        if let Err(error) =
            validate_diagnostic_expectations(&scene.id, &diagnostics, &expected_diagnostics)
        {
            panic!("{error}");
        }

        let expected: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(expected_root.join("gates.json"))?)?;
        let round_trip = verify_round_trip(&graph)?;
        let actual_internal = [
            ("structural", round_trip.structural.pass),
            ("semantic_ir", round_trip.semantic_ir.pass),
            ("diagnostics", true),
        ];
        for (gate, passed) in actual_internal {
            let actual = if passed { "pass" } else { "fail" };
            assert_eq!(expected[gate], actual, "{} expected {gate}", scene.id);
        }
        for gate in ["computed_style", "geometry", "visual"] {
            let required = if scene.tags.iter().any(|tag| tag == "browser") {
                "pass"
            } else {
                "unavailable"
            };
            assert_eq!(expected[gate], required, "{} expected {gate}", scene.id);
        }
        if scene.tags.iter().any(|tag| tag == "negative") {
            assert!(
                !diagnostics.is_empty(),
                "negative fixture {} must prove at least one diagnostic",
                scene.id
            );
        }
    }
    Ok(())
}

#[test]
fn coverage_matrix_names_every_supported_property_and_real_fixture()
-> Result<(), Box<dyn std::error::Error>> {
    const PROPERTIES: &[&str] = &[
        "margin",
        "padding",
        "background",
        "border",
        "appearance",
        "position",
        "display",
        "box-sizing",
        "top",
        "left",
        "width",
        "height",
        "min-width",
        "margin-top",
        "margin-right",
        "margin-bottom",
        "margin-left",
        "padding-top",
        "padding-right",
        "padding-bottom",
        "padding-left",
        "border-width",
        "border-color",
        "border-radius",
        "background-color",
        "flex-direction",
        "align-items",
        "justify-content",
        "align-self",
        "gap",
        "cursor",
        "color",
        "font-family",
        "font-size",
        "font-weight",
        "line-height",
    ];
    let root = repo_root();
    let coverage: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
        root.join("fixtures").join("v0.1").join("coverage.json"),
    )?)?;
    let properties = coverage["supported_properties"]
        .as_object()
        .expect("supported_properties must be an object");
    assert_eq!(properties.len(), PROPERTIES.len());
    for property in PROPERTIES {
        let scene_id = properties[*property]
            .as_str()
            .unwrap_or_else(|| panic!("coverage missing supported property {property}"));
        let graph = load_scene_source_graph(&root, scene_id)?;
        assert!(
            graph.css_rules.iter().any(|rule| rule
                .declarations
                .iter()
                .any(|declaration| declaration.property == *property)),
            "coverage target {scene_id} does not declare {property}"
        );
    }

    let selectors = coverage["supported_selectors"]
        .as_object()
        .expect("supported_selectors must be an object");
    for selector in ["body", "h1", "p", "button", "button.primary"] {
        let scene_id = selectors[selector]
            .as_str()
            .unwrap_or_else(|| panic!("coverage missing supported selector {selector}"));
        let graph = load_scene_source_graph(&root, scene_id)?;
        assert!(
            graph
                .css_rules
                .iter()
                .any(|rule| rule.selectors.iter().any(|candidate| candidate == selector)),
            "coverage target {scene_id} does not use selector {selector}"
        );
    }
    let class_scene = selectors[".class"].as_str().expect("class coverage scene");
    assert!(
        load_scene_source_graph(&root, class_scene)?
            .css_rules
            .iter()
            .flat_map(|rule| &rule.selectors)
            .any(|selector| selector.starts_with('.')
                && selector[1..]
                    .chars()
                    .all(|character| character.is_alphanumeric() || character == '-'))
    );
    let group_scene = selectors["comma-separated group"]
        .as_str()
        .expect("group coverage scene");
    assert!(
        load_scene_source_graph(&root, group_scene)?
            .css_rules
            .iter()
            .any(|rule| rule.selectors.len() > 1)
    );

    for (value_name, specification) in coverage["supported_values"]
        .as_object()
        .expect("supported_values must be an object")
    {
        let scene_id = specification["scene"].as_str().expect("value scene");
        let property = specification["property"].as_str().expect("value property");
        let value = specification["value"].as_str().expect("value spelling");
        let graph = load_scene_source_graph(&root, scene_id)?;
        assert!(
            graph.css_rules.iter().any(|rule| rule
                .declarations
                .iter()
                .any(|declaration| declaration.property == property && declaration.value == value)),
            "coverage target {scene_id} does not exercise {value_name}: {property}: {value}"
        );
    }

    let manifest =
        crate::load_fixture_manifest(root.join("fixtures").join("v0.1").join("manifest.toml"))?;
    for scene_id in coverage["real_world_scenes"]
        .as_array()
        .expect("real_world_scenes must be an array")
    {
        let scene_id = scene_id.as_str().expect("real-world id must be a string");
        let scene = manifest.scene(scene_id)?;
        assert!(scene.scene.tags.iter().any(|tag| tag == "real-world"));
    }
    assert_eq!(
        coverage["negative_categories"]
            .as_object()
            .expect("negative_categories must be an object")
            .len(),
        3
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Per-scene golden artifact validation
// ---------------------------------------------------------------------------

/// Golden scene.semui.json must be valid JSON that round-trips through serde.
#[test]
fn golden_semui_json_is_deserializable_for_all_scenes() -> Result<(), Box<dyn std::error::Error>> {
    for scene_id in fixture_scene_ids(None)? {
        let (ir, _) = build_golden_artifacts(repo_root(), &scene_id)?;
        // Serialize → deserialize round-trip is lossless
        let json = ir.to_json()?;
        let ir2 = SceneIr::from_json(&json)?;
        assert_eq!(ir.schema_version, ir2.schema_version);
        assert_eq!(ir.scene_id, ir2.scene_id);
        assert_eq!(ir.nodes.len(), ir2.nodes.len());
    }
    Ok(())
}

/// Freshly generated artifacts must match the committed goldens exactly.
/// This is the read-only acceptance gate: it never writes to the repo, so a
/// normal `cargo test` leaves the working tree clean. If this fails, the
/// goldens are stale and must be regenerated via the explicit `regenerate_goldens`
/// maintenance step (run with `--ignored`).
#[test]
fn generated_artifacts_match_committed_goldens() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::read_to_string;

    for scene_id in fixture_scene_ids(None)? {
        let (ir, emitted) = build_golden_artifacts(repo_root(), &scene_id)?;
        let dir = repo_root()
            .join("fixtures")
            .join("v0.1")
            .join(&scene_id)
            .join("expected");

        let cases = [
            ("scene.semui.json", ir.to_json()?),
            ("roundtrip.html", emitted.html),
            ("roundtrip.css", emitted.css),
        ];
        for (name, generated) in cases {
            let committed = read_to_string(dir.join(name))?;
            assert_eq!(
                norm(&committed),
                norm(&generated),
                "stale golden: {scene_id}/expected/{name} differs from generated output"
            );
        }
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
        TraceSpan {
            start: 0,
            end: 0,
            line: 1,
            column: 1,
        }
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

// ---------------------------------------------------------------------------
// Explicit golden-regeneration maintenance path
// ---------------------------------------------------------------------------

/// Regenerate the committed golden artifacts for every scene.
///
/// This is the **explicit maintenance path** referenced by
/// `generated_artifacts_match_committed_goldens`. It is the only thing that
/// writes to `fixtures/v0.1/*/expected/`, and it is `#[ignore]`d so a normal
/// `cargo test` never runs it — keeping verification read-only. Run it
/// deliberately after an intentional pipeline change:
///
/// ```text
/// cargo test --lib release::tests::proof::regenerate_goldens -- --ignored
/// ```
///
/// Then review the resulting diff and commit the refreshed goldens.
#[test]
#[ignore = "maintenance only: rewrites committed goldens; run with --ignored"]
fn regenerate_goldens() -> Result<(), Box<dyn std::error::Error>> {
    for scene_id in fixture_scene_ids(None)? {
        write_golden_artifacts(repo_root(), &scene_id)?;
    }
    Ok(())
}
