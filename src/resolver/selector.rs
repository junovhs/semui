use crate::HtmlNode;

/// CSS specificity for the v0.1 subset: (class_count, type_count).
///
/// ID selectors are out of scope. Specificity is compared lexicographically:
/// higher class_count wins, then higher type_count, then source order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity(pub u32, pub u32);

/// Parse a single simple-or-compound selector string into type and class parts.
///
/// Handles the v0.1 subset:
/// - type selector: `"body"`, `"h1"`, `"button"`
/// - class selector: `".card"`, `".meta"`
/// - compound: `"button.primary"`, `".button.secondary"`
fn parse_simple_selector(selector: &str) -> (Option<&str>, Vec<&str>) {
    let mut parts = selector.splitn(2, '.');
    let first = parts.next().unwrap_or("").trim();
    let rest = &selector[first.len()..];

    let type_part = if first.is_empty() { None } else { Some(first) };
    let class_parts: Vec<&str> = rest.split('.').filter(|s| !s.is_empty()).collect();

    (type_part, class_parts)
}

/// Compute the CSS specificity of a single selector (no comma).
pub fn specificity(selector: &str) -> Specificity {
    let (type_part, class_parts) = parse_simple_selector(selector);
    let type_count = if type_part.is_some() { 1 } else { 0 };
    Specificity(class_parts.len() as u32, type_count)
}

/// Returns true if `selector` (a single non-comma selector) matches `node`.
pub fn selector_matches(selector: &str, node: &HtmlNode) -> bool {
    let (type_part, class_parts) = parse_simple_selector(selector);

    if let Some(type_name) = type_part
        && node.name.as_deref() != Some(type_name)
    {
        return false;
    }

    if !class_parts.is_empty() {
        let node_classes = node_class_set(node);
        if !class_parts.iter().all(|c| node_classes.contains(c)) {
            return false;
        }
    }

    true
}

fn node_class_set(node: &HtmlNode) -> Vec<&str> {
    node.attributes
        .iter()
        .find(|(k, _)| k == "class")
        .map(|(_, v)| v.split_whitespace().collect())
        .unwrap_or_default()
}
