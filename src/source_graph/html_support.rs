use super::model::SourceGraphError;

pub struct StartTag {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub self_closing: bool,
}

pub fn find_tag_end(source: &str, start: usize) -> Result<usize, SourceGraphError> {
    let bytes = source.as_bytes();
    let mut index = start + 1;
    let mut quote = None;

    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(active) = quote {
            if byte == active {
                quote = None;
            }
        } else if byte == b'"' || byte == b'\'' {
            quote = Some(byte);
        } else if byte == b'>' {
            return Ok(index);
        }
        index += 1;
    }

    Err(SourceGraphError::UnsupportedHtml {
        reason: "unterminated HTML tag".to_owned(),
        offset: start,
    })
}

pub fn parse_start_tag(tag_body: &str, offset: usize) -> Result<StartTag, SourceGraphError> {
    let self_closing = tag_body.ends_with('/') || is_void_element(tag_body);
    let trimmed = tag_body.trim_end_matches('/').trim();
    let name_end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
    let name = &trimmed[..name_end];
    if name.is_empty() {
        return Err(SourceGraphError::UnsupportedHtml {
            reason: "HTML element tag names must not be empty".to_owned(),
            offset,
        });
    }

    Ok(StartTag {
        name: name.to_owned(),
        attributes: parse_attributes(trimmed[name_end..].trim(), offset)?,
        self_closing,
    })
}

fn parse_attributes(
    input: &str,
    offset: usize,
) -> Result<Vec<(String, String)>, SourceGraphError> {
    let mut attributes = Vec::new();
    let mut index = 0;

    while let Some(name) = next_attribute_name(input, &mut index) {
        let value = parse_attribute_value(input, &mut index, &name, offset)?;
        attributes.push((name, value));
    }

    Ok(attributes)
}

fn next_attribute_name(input: &str, index: &mut usize) -> Option<String> {
    let bytes = input.as_bytes();
    while *index < bytes.len() && bytes[*index].is_ascii_whitespace() {
        *index += 1;
    }
    if *index >= bytes.len() {
        return None;
    }

    let start = *index;
    while *index < bytes.len() && !bytes[*index].is_ascii_whitespace() && bytes[*index] != b'=' {
        *index += 1;
    }
    Some(input[start..*index].trim().to_owned())
}

fn parse_attribute_value(
    input: &str,
    index: &mut usize,
    name: &str,
    offset: usize,
) -> Result<String, SourceGraphError> {
    let bytes = input.as_bytes();
    while *index < bytes.len() && bytes[*index].is_ascii_whitespace() {
        *index += 1;
    }
    if *index >= bytes.len() || bytes[*index] != b'=' {
        return Err(SourceGraphError::UnsupportedHtml {
            reason: format!("attribute `{name}` must use quoted `name=\"value\"` syntax"),
            offset,
        });
    }
    *index += 1;
    while *index < bytes.len() && bytes[*index].is_ascii_whitespace() {
        *index += 1;
    }
    let quote = read_quote(bytes, *index, name, offset)?;
    *index += 1;
    let value_start = *index;
    while *index < bytes.len() && bytes[*index] != quote {
        *index += 1;
    }
    if *index >= bytes.len() {
        return Err(SourceGraphError::UnsupportedHtml {
            reason: format!("unterminated value for attribute `{name}`"),
            offset,
        });
    }
    let value = input[value_start..*index].to_owned();
    *index += 1;
    Ok(value)
}

fn read_quote(
    bytes: &[u8],
    index: usize,
    name: &str,
    offset: usize,
) -> Result<u8, SourceGraphError> {
    if index >= bytes.len() || (bytes[index] != b'"' && bytes[index] != b'\'') {
        return Err(SourceGraphError::UnsupportedHtml {
            reason: format!("attribute `{name}` must use quoted values"),
            offset,
        });
    }
    Ok(bytes[index])
}

fn is_void_element(tag_body: &str) -> bool {
    let name = tag_body
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_end_matches('/');
    matches!(name, "meta" | "link" | "img" | "br" | "hr" | "input")
}
