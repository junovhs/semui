use super::model::{CssDeclaration, CssRule, SourceGraphError, TraceSpan};

pub fn parse_css_document(
    document_id: usize,
    source: &str,
) -> Result<Vec<CssRule>, SourceGraphError> {
    let mut rules = Vec::new();
    let mut index = 0;
    let bytes = source.as_bytes();

    while index < bytes.len() {
        index = skip_css_ws(source, index);
        if index >= bytes.len() {
            break;
        }

        let selector_start = index;
        while index < bytes.len() && bytes[index] != b'{' {
            index += 1;
        }

        if index >= bytes.len() {
            return Err(SourceGraphError::UnsupportedCss {
                reason: "missing opening brace for CSS rule".to_owned(),
                offset: selector_start,
            });
        }

        let selector_text = source[selector_start..index].trim();
        if selector_text.starts_with('@') {
            return Err(SourceGraphError::UnsupportedCss {
                reason: "at-rules are outside the v0.1 CSS subset".to_owned(),
                offset: selector_start,
            });
        }

        index += 1;
        let body_start = index;
        while index < bytes.len() && bytes[index] != b'}' {
            index += 1;
        }

        if index >= bytes.len() {
            return Err(SourceGraphError::UnsupportedCss {
                reason: "missing closing brace for CSS rule".to_owned(),
                offset: body_start,
            });
        }

        let body_end = index;
        let selectors = parse_selectors(selector_text, selector_start)?;
        let declarations =
            parse_css_declarations(source, &source[body_start..body_end], body_start)?;
        rules.push(CssRule {
            id: rules.len(),
            selectors,
            declarations,
            span: span_for(source, selector_start, body_end + 1),
            document_id,
        });

        index += 1;
    }

    Ok(rules)
}

fn parse_selectors(selector_text: &str, offset: usize) -> Result<Vec<String>, SourceGraphError> {
    let selectors = selector_text
        .split(',')
        .map(str::trim)
        .filter(|selector| !selector.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    if selectors.is_empty() {
        return Err(SourceGraphError::UnsupportedCss {
            reason: "empty selector list".to_owned(),
            offset,
        });
    }

    Ok(selectors)
}

fn parse_css_declarations(
    full_source: &str,
    block: &str,
    block_offset: usize,
) -> Result<Vec<CssDeclaration>, SourceGraphError> {
    let mut declarations = Vec::new();
    let mut cursor = 0;

    for segment in block.split(';') {
        let segment_len = segment.len();
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            cursor += segment_len + 1;
            continue;
        }

        let Some(colon_index) = trimmed.find(':') else {
            return Err(SourceGraphError::UnsupportedCss {
                reason: "CSS declarations must use `property: value` syntax".to_owned(),
                offset: block_offset + cursor,
            });
        };

        let property = trimmed[..colon_index].trim();
        let value = trimmed[colon_index + 1..].trim();
        if property.is_empty() || value.is_empty() {
            return Err(SourceGraphError::UnsupportedCss {
                reason: "CSS declarations must include both a property and a value".to_owned(),
                offset: block_offset + cursor,
            });
        }

        declarations.push(CssDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
            span: span_for(
                full_source,
                block_offset + cursor,
                block_offset + cursor + segment_len,
            ),
        });
        cursor += segment_len + 1;
    }

    Ok(declarations)
}

fn skip_css_ws(source: &str, mut index: usize) -> usize {
    let bytes = source.as_bytes();
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    index
}

fn span_for(source: &str, start: usize, end: usize) -> TraceSpan {
    let mut line = 1;
    let mut line_start = 0;
    for (offset, ch) in source.char_indices() {
        if offset >= start {
            break;
        }
        if ch == '\n' {
            line += 1;
            line_start = offset + ch.len_utf8();
        }
    }

    TraceSpan {
        start,
        end,
        line,
        column: start.saturating_sub(line_start) + 1,
    }
}
