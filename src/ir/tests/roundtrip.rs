use super::super::*;
use super::{minimal_layout, minimal_paint};

#[test]
fn scene_ir_json_round_trip_minimal() -> Result<(), serde_json::Error> {
    let ir = super::minimal_scene();
    let json = ir.to_json()?;
    let restored: SceneIr = SceneIr::from_json(&json)?;
    assert_eq!(ir, restored);
    Ok(())
}

/// Exercises every field in IrNode: box, control, and text nodes with full
/// layout, paint, typography, and provenance spans.
#[test]
fn full_scene_round_trip_with_text_and_control() -> Result<(), serde_json::Error> {
    let ir = SceneIr {
        schema_version: 1,
        scene_id: "action_row_variants".to_string(),
        corpus: "v0.1".to_string(),
        execution_mode: ExecutionMode::Static,
        nodes: vec![
            IrNode {
                id: "n0".to_string(),
                kind: NodeKind::Box,
                parent_id: None,
                control_kind: None,
                text_content: None,
                layout: Layout {
                    position: Position::Static,
                    display: Display::Flex,
                    box_sizing: BoxSizing::BorderBox,
                    top: None,
                    left: None,
                    width: Some(320.0),
                    height: Some(48.0),
                    min_width: None,
                    margin: EdgeInset::zero(),
                    padding: EdgeInset::uniform(8.0),
                    flex_direction: Some(FlexDirection::Row),
                    align_items: Some(AlignItems::Center),
                    justify_content: Some(JustifyContent::Center),
                    align_self: None,
                    gap: Some(8.0),
                },
                paint: Paint {
                    background_color: Some(Color("#f5f5f5".to_string())),
                    border: None,
                    border_radius: None,
                    cursor: None,
                },
                typography: None,
                source: SourceRef {
                    doc_id: 0,
                    dom_path: "body > div.row".to_string(),
                    span: Some(SourceSpan { start: 0, end: 42, line: 1, column: 1 }),
                },
            },
            IrNode {
                id: "n1".to_string(),
                kind: NodeKind::Control,
                parent_id: Some("n0".to_string()),
                control_kind: Some(ControlKind::Button),
                text_content: None,
                layout: Layout {
                    position: Position::Static,
                    display: Display::InlineFlex,
                    box_sizing: BoxSizing::BorderBox,
                    top: None,
                    left: None,
                    width: Some(80.0),
                    height: Some(32.0),
                    min_width: None,
                    margin: EdgeInset::zero(),
                    padding: EdgeInset { top: 0.0, right: 16.0, bottom: 0.0, left: 16.0 },
                    flex_direction: None,
                    align_items: Some(AlignItems::Center),
                    justify_content: Some(JustifyContent::Center),
                    align_self: None,
                    gap: None,
                },
                paint: Paint {
                    background_color: Some(Color("#0066cc".to_string())),
                    border: Some(Border { width: 1.0, color: Color("#0055aa".to_string()) }),
                    border_radius: Some(4.0),
                    cursor: Some(Cursor::Pointer),
                },
                typography: Some(Typography {
                    font_family: vec!["Inter".to_string(), "sans-serif".to_string()],
                    font_size: 14.0,
                    font_weight: 600,
                    line_height: LineHeight::Length { value: 20.0 },
                    color: Color("#ffffff".to_string()),
                }),
                source: SourceRef {
                    doc_id: 0,
                    dom_path: "body > div.row > button.primary".to_string(),
                    span: None,
                },
            },
            IrNode {
                id: "n2".to_string(),
                kind: NodeKind::Text,
                parent_id: Some("n1".to_string()),
                control_kind: None,
                text_content: Some("Save".to_string()),
                layout: minimal_layout(),
                paint: minimal_paint(),
                typography: Some(Typography {
                    font_family: vec!["Inter".to_string(), "sans-serif".to_string()],
                    font_size: 14.0,
                    font_weight: 600,
                    line_height: LineHeight::Normal,
                    color: Color("#ffffff".to_string()),
                }),
                source: SourceRef {
                    doc_id: 0,
                    dom_path: "body > div.row > button.primary > #text".to_string(),
                    span: None,
                },
            },
        ],
    };

    let json = ir.to_json()?;
    let restored: SceneIr = SceneIr::from_json(&json)?;
    assert_eq!(ir, restored);

    // Pre-order invariant: parents appear before children
    assert_eq!(restored.nodes[0].id, "n0");
    assert_eq!(restored.nodes[1].id, "n1");
    assert_eq!(restored.nodes[2].id, "n2");
    assert_eq!(restored.nodes[2].parent_id.as_deref(), Some("n1"));

    // Kind-specific fields preserved through JSON
    assert_eq!(restored.nodes[2].text_content.as_deref(), Some("Save"));
    assert_eq!(restored.nodes[1].control_kind, Some(ControlKind::Button));
    Ok(())
}
