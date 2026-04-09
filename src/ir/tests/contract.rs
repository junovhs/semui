use super::super::*;
use super::minimal_scene;

// --- JSON shape contract ---

#[test]
fn execution_mode_serializes_to_static_string() -> Result<(), serde_json::Error> {
    let json = serde_json::to_string(&ExecutionMode::Static)?;
    assert_eq!(json, r#""static""#);
    Ok(())
}

#[test]
fn node_kind_serializes_correctly() -> Result<(), serde_json::Error> {
    assert_eq!(serde_json::to_string(&NodeKind::Box)?, r#""box""#);
    assert_eq!(serde_json::to_string(&NodeKind::Text)?, r#""text""#);
    assert_eq!(serde_json::to_string(&NodeKind::Control)?, r#""control""#);
    Ok(())
}

#[test]
fn control_kind_serializes_to_button() -> Result<(), serde_json::Error> {
    assert_eq!(serde_json::to_string(&ControlKind::Button)?, r#""button""#);
    Ok(())
}

#[test]
fn line_height_normal_serializes_as_tagged_kind() -> Result<(), serde_json::Error> {
    let json = serde_json::to_string(&LineHeight::Normal)?;
    let v: serde_json::Value = serde_json::from_str(&json)?;
    assert_eq!(v["kind"], "normal", "expected kind=normal, got: {json}");
    assert!(v.get("value").is_none(), "normal must not carry a value field");
    Ok(())
}

#[test]
fn line_height_length_serializes_with_value() -> Result<(), serde_json::Error> {
    let json = serde_json::to_string(&LineHeight::Length { value: 24.0 })?;
    let v: serde_json::Value = serde_json::from_str(&json)?;
    assert_eq!(v["kind"], "length");
    assert_eq!(v["value"], 24.0_f64);
    Ok(())
}

#[test]
fn display_inline_flex_serializes_as_snake_case() -> Result<(), serde_json::Error> {
    let json = serde_json::to_string(&Display::InlineFlex)?;
    assert_eq!(json, r#""inline_flex""#);
    Ok(())
}

// --- Optional fields omitted in JSON ---

#[test]
fn none_optional_fields_absent_from_json() -> Result<(), serde_json::Error> {
    let ir = minimal_scene();
    let json = ir.to_json()?;
    let v: serde_json::Value = serde_json::from_str(&json)?;
    let node = &v["nodes"][0];
    assert!(node.get("parent_id").is_none(), "parent_id must be absent when None");
    assert!(node.get("control_kind").is_none(), "control_kind must be absent when None");
    assert!(node.get("text_content").is_none(), "text_content must be absent when None");
    assert!(node.get("typography").is_none(), "typography must be absent when None");
    assert!(node["source"].get("span").is_none(), "source.span must be absent when None");
    let layout = &node["layout"];
    for field in &["top", "left", "width", "height", "min_width"] {
        assert!(layout.get(field).is_none(), "{field} must be absent from layout when None");
    }
    Ok(())
}

// --- schema_version field ---

#[test]
fn schema_version_is_present_in_serialized_output() -> Result<(), serde_json::Error> {
    let ir = minimal_scene();
    let json = ir.to_json()?;
    let v: serde_json::Value = serde_json::from_str(&json)?;
    assert_eq!(v["schema_version"], 1);
    Ok(())
}

// --- Negative: malformed JSON is rejected ---

#[test]
fn from_json_rejects_malformed_input() {
    assert!(SceneIr::from_json("{not valid json}").is_err());
}

#[test]
fn from_json_rejects_missing_required_field() {
    // schema_version is required; omitting it must fail
    let json = r#"{
        "scene_id": "test",
        "corpus": "v0.1",
        "execution_mode": "static",
        "nodes": []
    }"#;
    assert!(
        SceneIr::from_json(json).is_err(),
        "expected error when schema_version is missing"
    );
}

// --- EdgeInset helpers ---

#[test]
fn edge_inset_zero_is_all_zeros() {
    let z = EdgeInset::zero();
    assert_eq!((z.top, z.right, z.bottom, z.left), (0.0, 0.0, 0.0, 0.0));
}

#[test]
fn edge_inset_uniform_sets_all_sides() {
    let u = EdgeInset::uniform(12.0);
    assert_eq!((u.top, u.right, u.bottom, u.left), (12.0, 12.0, 12.0, 12.0));
}
