# SEMUI v0.1 Fixture Corpus

This directory contains the concrete fixture inputs selected by `ACC-02` and materialized by `ACC-03`.

Layout rules:

- `manifest.toml` is the canonical machine-readable index
- each scene lives in its own directory
- each scene directory contains `source.html`, `source.css`, and expected artifacts
- each scene reserves `expected/scene.semui.json`, `expected/roundtrip.html`, and `expected/roundtrip.css`

The `expected/` files are checked-in regression artifacts. Verification must
compare them without rewriting them.

## Deterministic browser capture

Install the exact locked tooling and browser, then run the capture harness:

```text
npm ci
npx playwright install chromium
npm test -- --output /tmp/semui-browser-evidence.json
```

`browser-policy.json` pins the Playwright/Chromium versions, viewport, DPR,
locale, timezone, motion/color policy, and locally bundled Inter font files.
The harness blocks all network requests, renders both source and emitted
artifacts twice, and fails unless each pair of captures has identical DOM and
PNG hashes. It compares source against emitted structure, computed styles, and
geometry; source-versus-emitted pixel comparison belongs to later oracle work.

The harness also records browser observations for every Scene IR element using
stable preorder identities. Exact supported computed styles and numeric styles
within the `1px` loss budget are compared separately from DOM rectangle
geometry. A built-in mutation self-test proves that style and geometry drift
fail with node/property-specific evidence.
