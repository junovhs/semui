//! HTML generation from IR nodes.

use crate::ir::{ControlKind, IrNode, NodeKind, SceneIr};

/// Emit a full HTML document (with an embedded `<link>` to `styles.css`).
pub fn build_html(ir: &SceneIr) -> String {
    let body = render_children(None, &ir.nodes, 2);
    format!(
        "<!doctype html>\n\
         <html lang=\"en\">\n\
         <head>\n\
         <meta charset=\"utf-8\">\n\
         <link rel=\"stylesheet\" href=\"styles.css\">\n\
         </head>\n\
         <body>\n\
         {body}\n\
         </body>\n\
         </html>"
    )
}

/// Render all children of `parent_id` as indented HTML.
fn render_children(parent_id: Option<&str>, nodes: &[IrNode], depth: usize) -> String {
    let parts: Vec<String> = nodes
        .iter()
        .filter(|n| n.parent_id.as_deref() == parent_id)
        .map(|n| render_node(n, nodes, depth))
        .collect();
    parts.join("\n")
}

fn render_node(node: &IrNode, all_nodes: &[IrNode], depth: usize) -> String {
    let indent = "  ".repeat(depth);
    match node.kind {
        NodeKind::Text => {
            let text = node.text_content.as_deref().unwrap_or("");
            format!("{indent}{}", escape_html(text))
        }
        NodeKind::Box | NodeKind::Control => {
            let tag = element_tag(node);
            let class = &node.id;
            let inner = render_children(Some(&node.id), all_nodes, depth + 1);
            if inner.is_empty() {
                format!("{indent}<{tag} class=\"{class}\"></{tag}>")
            } else {
                format!("{indent}<{tag} class=\"{class}\">\n{inner}\n{indent}</{tag}>")
            }
        }
    }
}

fn element_tag(node: &IrNode) -> &'static str {
    match (&node.kind, &node.control_kind) {
        (NodeKind::Control, Some(ControlKind::Button)) => "button",
        _ => "div",
    }
}

fn escape_html(s: &str) -> String {
    // Only escape the characters that are unsafe in HTML text content.
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}
