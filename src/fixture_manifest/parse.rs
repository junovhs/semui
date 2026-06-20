use super::{FixtureManifestError, FixtureScene};
use std::path::PathBuf;

#[derive(Default)]
pub(super) struct ParsedManifest {
    pub schema_version: u32,
    pub corpus: String,
    pub scenes: Vec<FixtureScene>,
}

#[derive(Default)]
struct SceneBuilder {
    id: Option<String>,
    priority: Option<u32>,
    dir: Option<PathBuf>,
    source_html: Option<PathBuf>,
    source_css: Option<PathBuf>,
    expected_semui: Option<PathBuf>,
    expected_roundtrip_html: Option<PathBuf>,
    expected_roundtrip_css: Option<PathBuf>,
    tags: Option<Vec<String>>,
}

impl ParsedManifest {
    pub(super) fn parse(contents: &str) -> Result<Self, FixtureManifestError> {
        ManifestParser::default().parse(contents)
    }
}

#[derive(Default)]
struct ManifestParser {
    schema_version: Option<u32>,
    corpus: Option<String>,
    scenes: Vec<FixtureScene>,
    current_scene: Option<SceneBuilder>,
}

impl ManifestParser {
    fn parse(mut self, contents: &str) -> Result<ParsedManifest, FixtureManifestError> {
        for (index, raw_line) in contents.lines().enumerate() {
            self.parse_line(raw_line, index + 1)?;
        }
        self.finish_scene(contents.lines().count())?;

        Ok(ParsedManifest {
            schema_version: self.schema_version.ok_or_else(|| {
                parse_error(1, "missing required `schema_version` field".to_owned())
            })?,
            corpus: self
                .corpus
                .ok_or_else(|| parse_error(1, "missing required `corpus` field".to_owned()))?,
            scenes: self.scenes,
        })
    }

    fn parse_line(
        &mut self,
        raw_line: &str,
        line_number: usize,
    ) -> Result<(), FixtureManifestError> {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            return Ok(());
        }

        if line == "[[scene]]" {
            self.finish_scene(line_number.saturating_sub(1))?;
            self.current_scene = Some(SceneBuilder::default());
            return Ok(());
        }

        let (key, value) = split_key_value(line, line_number)?;
        if let Some(scene) = self.current_scene.as_mut() {
            scene.apply(key, value, line_number)
        } else {
            self.apply_top_level(key, value, line_number)
        }
    }

    fn finish_scene(&mut self, line_number: usize) -> Result<(), FixtureManifestError> {
        if let Some(scene) = self.current_scene.take() {
            self.scenes.push(scene.finish(line_number)?);
        }
        Ok(())
    }

    fn apply_top_level(
        &mut self,
        key: &str,
        value: &str,
        line_number: usize,
    ) -> Result<(), FixtureManifestError> {
        match key {
            "schema_version" => self.schema_version = Some(parse_u32(value, line_number)?),
            "corpus" => self.corpus = Some(parse_string(value, line_number)?),
            _ => {
                return Err(parse_error(
                    line_number,
                    format!("unexpected top-level key `{key}`"),
                ));
            }
        }
        Ok(())
    }
}

impl SceneBuilder {
    fn apply(
        &mut self,
        key: &str,
        value: &str,
        line_number: usize,
    ) -> Result<(), FixtureManifestError> {
        match key {
            "id" => self.id = Some(parse_string(value, line_number)?),
            "priority" => self.priority = Some(parse_u32(value, line_number)?),
            "dir" => self.dir = Some(PathBuf::from(parse_string(value, line_number)?)),
            "source_html" => {
                self.source_html = Some(PathBuf::from(parse_string(value, line_number)?))
            }
            "source_css" => {
                self.source_css = Some(PathBuf::from(parse_string(value, line_number)?))
            }
            "expected_semui" => {
                self.expected_semui = Some(PathBuf::from(parse_string(value, line_number)?));
            }
            "expected_roundtrip_html" => {
                self.expected_roundtrip_html =
                    Some(PathBuf::from(parse_string(value, line_number)?));
            }
            "expected_roundtrip_css" => {
                self.expected_roundtrip_css =
                    Some(PathBuf::from(parse_string(value, line_number)?));
            }
            "tags" => self.tags = Some(parse_string_array(value, line_number)?),
            _ => {
                return Err(parse_error(
                    line_number,
                    format!("unexpected scene key `{key}`"),
                ));
            }
        }
        Ok(())
    }

    fn finish(self, line_number: usize) -> Result<FixtureScene, FixtureManifestError> {
        Ok(FixtureScene {
            id: required(self.id, "id", line_number)?,
            priority: required(self.priority, "priority", line_number)?,
            dir: required(self.dir, "dir", line_number)?,
            source_html: required(self.source_html, "source_html", line_number)?,
            source_css: required(self.source_css, "source_css", line_number)?,
            expected_semui: required(self.expected_semui, "expected_semui", line_number)?,
            expected_roundtrip_html: required(
                self.expected_roundtrip_html,
                "expected_roundtrip_html",
                line_number,
            )?,
            expected_roundtrip_css: required(
                self.expected_roundtrip_css,
                "expected_roundtrip_css",
                line_number,
            )?,
            tags: required(self.tags, "tags", line_number)?,
        })
    }
}

fn split_key_value(line: &str, line_number: usize) -> Result<(&str, &str), FixtureManifestError> {
    let Some(index) = line.find('=') else {
        return Err(parse_error(
            line_number,
            "expected `key = value` syntax".to_owned(),
        ));
    };
    let key = line[..index].trim();
    let value = line[index + 1..].trim();
    if key.is_empty() || value.is_empty() {
        return Err(parse_error(
            line_number,
            "expected both a key and a value".to_owned(),
        ));
    }
    Ok((key, value))
}

fn parse_string(value: &str, line_number: usize) -> Result<String, FixtureManifestError> {
    if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
        return Err(parse_error(
            line_number,
            format!("expected string literal, found `{value}`"),
        ));
    }
    Ok(value[1..value.len() - 1].to_owned())
}

fn parse_string_array(
    value: &str,
    line_number: usize,
) -> Result<Vec<String>, FixtureManifestError> {
    if !value.starts_with('[') || !value.ends_with(']') {
        return Err(parse_error(
            line_number,
            format!("expected string array, found `{value}`"),
        ));
    }
    let inner = value[1..value.len() - 1].trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    inner
        .split(',')
        .map(|item| parse_string(item.trim(), line_number))
        .collect()
}

fn parse_u32(value: &str, line_number: usize) -> Result<u32, FixtureManifestError> {
    value.parse::<u32>().map_err(|_| {
        parse_error(
            line_number,
            format!("expected unsigned integer, found `{value}`"),
        )
    })
}

fn required<T>(
    value: Option<T>,
    field: &str,
    line_number: usize,
) -> Result<T, FixtureManifestError> {
    value.ok_or_else(|| parse_error(line_number, format!("missing required `{field}` field")))
}

fn parse_error(line: usize, message: String) -> FixtureManifestError {
    FixtureManifestError::Parse { line, message }
}
