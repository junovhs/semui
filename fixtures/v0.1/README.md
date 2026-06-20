# SEMUI v0.1 Fixture Corpus

This directory contains the concrete fixture inputs selected by `ACC-02` and materialized by `ACC-03`.

Layout rules:

- `manifest.toml` is the canonical machine-readable index
- each scene lives in its own directory
- each scene directory contains `source.html`, `source.css`, and expected artifacts
- each scene reserves `expected/scene.semui.json`, `expected/roundtrip.html`, and `expected/roundtrip.css`

The `expected/` files are checked-in regression artifacts. Verification must
compare them without rewriting them.
