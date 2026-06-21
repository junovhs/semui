# SEMUI v0.1 Fixture Corpus

This directory contains the concrete fixture inputs selected by `ACC-02` and materialized by `ACC-03`.

Layout rules:

- `manifest.toml` is the canonical machine-readable index
- each scene lives in its own directory
- each scene directory contains `source.html`, `source.css`, and expected artifacts
- each scene reserves `expected/scene.semui.json`, `expected/roundtrip.html`, and `expected/roundtrip.css`
- every scene declares exact diagnostics in `expected/diagnostics.txt` and all
  six expected gate states in `expected/gates.json`
- `coverage.json` maps every supported selector, property, and value family to
  a concrete fixture and names all negative diagnostic categories

Manifest tags define fixture roles. `browser` scenes participate in the full
browser proof. `negative` scenes prove unsupported selector, property, and
value diagnostics and intentionally remain outside browser-pass aggregation.
The six `canonical` scenes stay identifiable while microfixtures and minimized
real-world scenes expand executable coverage.

The `expected/` files are checked-in regression artifacts. Verification must
compare them without rewriting them.

## Deterministic browser capture

Install the exact locked tooling and browser, then run the capture harness:

```text
npm ci
npx playwright install chromium
npm test -- --output /tmp/semui-browser-evidence.json --artifacts /tmp/semui-browser-proof
```

`browser-policy.json` pins the Playwright/Chromium versions, viewport, DPR,
locale, timezone, motion/color policy, and locally bundled Inter font files.
The harness blocks all network requests, renders both source and emitted
artifacts twice, and fails unless each pair of captures has identical DOM and
PNG hashes. It compares source against emitted structure, computed styles,
geometry, and scene-cropped pixels for every `browser` fixture.

The harness also records browser observations for every Scene IR element using
stable preorder identities. Exact supported computed styles and numeric styles
within the `1px` loss budget are compared separately from DOM rectangle
geometry. Built-in mutation tests prove that style, geometry, text-shift,
border, and color drift fail with focused evidence. Visual failures optionally
write source, emitted, diff PNGs, and compact JSON outside the repository.
