import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import { chromium } from "playwright";
import { PNG } from "pngjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const policy = JSON.parse(await readFile(path.join(repoRoot, "browser-policy.json"), "utf8"));
const packageJson = JSON.parse(await readFile(path.join(repoRoot, "package.json"), "utf8"));

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function outputPath() {
  const index = process.argv.indexOf("--output");
  if (index === -1) return null;
  assert(process.argv[index + 1], "--output requires a path");
  return path.resolve(process.argv[index + 1]);
}

function optionPath(name) {
  const index = process.argv.indexOf(name);
  if (index === -1) return null;
  assert(process.argv[index + 1], `${name} requires a path`);
  return path.resolve(process.argv[index + 1]);
}

async function fontEnvironment() {
  const rules = [];
  const hashes = {};
  for (const weight of policy.font_weights) {
    const file = path.join(
      repoRoot,
      "node_modules",
      "@fontsource",
      "inter",
      "files",
      `inter-latin-${weight}-normal.woff2`,
    );
    const bytes = await readFile(file);
    hashes[String(weight)] = sha256(bytes);
    rules.push(`
      @font-face {
        font-family: "Inter";
        font-style: normal;
        font-display: block;
        font-weight: ${weight};
        src: url(data:font/woff2;base64,${bytes.toString("base64")}) format("woff2");
      }
    `);
  }
  return { css: rules.join("\n"), hashes };
}

function sceneEntries(manifest) {
  const entries = [];
  for (const block of manifest.split("[[scene]]").slice(1)) {
    const id = block.match(/^\s*id\s*=\s*"([^"]+)"/m)?.[1];
    const dir = block.match(/^\s*dir\s*=\s*"([^"]+)"/m)?.[1];
    assert(id && dir, `invalid fixture manifest scene block: ${block}`);
    entries.push({ id, dir });
  }
  assert.equal(entries.length, 6, "browser proof must cover all six v0.1 fixtures");
  return entries;
}

function inlineDocument(html, css, fontCss) {
  const deterministicCss = `
    ${fontCss}
    *, *::before, *::after {
      animation: none !important;
      caret-color: transparent !important;
      transition: none !important;
    }
    body {
      background: #ffffff !important;
    }
  `;
  const style = `<style data-semui-browser-proof>${deterministicCss}\n${css}</style>`;
  const linked = /<link\b[^>]*rel=["']stylesheet["'][^>]*>/i;
  if (linked.test(html)) return html.replace(linked, style);
  if (/<\/head>/i.test(html)) return html.replace(/<\/head>/i, `${style}</head>`);
  return `<!doctype html><html lang="en"><head>${style}</head><body>${html}</body></html>`;
}

async function capture(context, document) {
  const page = await context.newPage();
  const externalRequests = [];
  page.on("request", (request) => {
    if (!request.url().startsWith("data:") && request.url() !== "about:blank") {
      externalRequests.push(request.url());
    }
  });
  await page.setContent(document, { waitUntil: "load" });
  await page.evaluate(() => document.fonts.ready);
  const observations = await page.locator("body *").evaluateAll(
    (elements, styleProperties) =>
      elements.map((element, index) => {
        const style = getComputedStyle(element);
        const rect = element.getBoundingClientRect();
        let current = element;
        const path = [];
        while (current !== document.body) {
          path.push(Array.prototype.indexOf.call(current.parentElement.children, current));
          current = current.parentElement;
        }
        return {
          id: `node[${index}]`,
          path: path.reverse().join("/"),
          tag: element.tagName.toLowerCase(),
          kind: element.tagName.toLowerCase() === "button" ? "control" : "box",
          has_text: Array.from(element.childNodes).some(
            (node) => node.nodeType === Node.TEXT_NODE && node.textContent.trim() !== "",
          ),
          styles: Object.fromEntries(
            styleProperties.map((property) => [property, style.getPropertyValue(property)]),
          ),
          geometry: {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
          },
        };
      }),
    [...policy.exact_computed_styles, ...policy.numeric_computed_styles],
  );
  const bounds = observations.reduce(
    (result, observation) => ({
      left: Math.min(result.left, observation.geometry.x),
      top: Math.min(result.top, observation.geometry.y),
      right: Math.max(result.right, observation.geometry.x + observation.geometry.width),
      bottom: Math.max(result.bottom, observation.geometry.y + observation.geometry.height),
    }),
    { left: Infinity, top: Infinity, right: -Infinity, bottom: -Infinity },
  );
  assert(Number.isFinite(bounds.left), "scene must contain at least one visible element");
  const clipX = Math.max(0, Math.floor(bounds.left));
  const clipY = Math.max(0, Math.floor(bounds.top));
  const screenshot = await page.screenshot({
    animations: "disabled",
    caret: "hide",
    clip: {
      x: clipX,
      y: clipY,
      width: Math.min(policy.viewport.width, Math.ceil(bounds.right)) - clipX,
      height: Math.min(policy.viewport.height, Math.ceil(bounds.bottom)) - clipY,
    },
    type: "png",
  });
  const dom = await page.content();
  await page.close();
  assert.deepEqual(externalRequests, [], `network request escaped isolation: ${externalRequests}`);
  return {
    evidence: {
      screenshot_sha256: sha256(screenshot),
      dom_sha256: sha256(dom),
      observations,
    },
    screenshot,
  };
}

async function captureDeterministically(context, document, label) {
  const first = await capture(context, document);
  const second = await capture(context, document);
  assert.deepEqual(first.evidence, second.evidence, `${label} capture changed between identical renders`);
  assert(first.screenshot.equals(second.screenshot), `${label} screenshot bytes changed between identical renders`);
  return first;
}

function gate(details) {
  return { status: details.length === 0 ? "pass" : "fail", details };
}

function numericValue(value) {
  const parsed = Number.parseFloat(value);
  return Number.isFinite(parsed) ? parsed : null;
}

function compareObservations(source, emitted) {
  const identity = [];
  const computedStyle = [];
  const geometry = [];
  if (source.length !== emitted.length) {
    identity.push(`node count: source=${source.length} emitted=${emitted.length}`);
  }
  for (let index = 0; index < Math.min(source.length, emitted.length); index += 1) {
    const left = source[index];
    const right = emitted[index];
    for (const field of ["id", "path", "kind"]) {
      if (left[field] !== right[field]) {
        identity.push(`${left.id}.${field}: ${JSON.stringify(left[field])} -> ${JSON.stringify(right[field])}`);
      }
    }
    for (const property of policy.exact_computed_styles) {
      if (["font-family", "font-weight", "color"].includes(property) && !left.has_text && !right.has_text) {
        continue;
      }
      if (left.styles[property] !== right.styles[property]) {
        computedStyle.push(
          `${left.id}.style.${property}: ${JSON.stringify(left.styles[property])} -> ${JSON.stringify(right.styles[property])}`,
        );
      }
    }
    for (const property of policy.numeric_computed_styles) {
      if (["font-size", "line-height"].includes(property) && !left.has_text && !right.has_text) {
        continue;
      }
      const a = numericValue(left.styles[property]);
      const b = numericValue(right.styles[property]);
      const equalKeywords = a === null && b === null && left.styles[property] === right.styles[property];
      if (!equalKeywords && (a === null || b === null || Math.abs(a - b) > policy.numeric_tolerance_px)) {
        computedStyle.push(
          `${left.id}.style.${property}: ${JSON.stringify(left.styles[property])} -> ${JSON.stringify(right.styles[property])}`,
        );
      }
    }
    for (const field of ["x", "y", "width", "height"]) {
      const a = left.geometry[field];
      const b = right.geometry[field];
      if (Math.abs(a - b) > policy.numeric_tolerance_px) {
        geometry.push(`${left.id}.geometry.${field}: ${a} -> ${b}`);
      }
    }
  }
  return {
    identity: gate(identity),
    computed_style: gate(computedStyle),
    geometry: gate(geometry),
  };
}

function comparePixels(sourceBytes, emittedBytes) {
  const source = PNG.sync.read(sourceBytes);
  const emitted = PNG.sync.read(emittedBytes);
  assert.equal(source.width, emitted.width, "screenshot widths differ");
  assert.equal(source.height, emitted.height, "screenshot heights differ");

  const width = source.width;
  const height = source.height;
  const changed = new Uint8Array(width * height);
  const diff = new PNG({ width, height });
  let differingPixels = 0;
  for (let pixel = 0; pixel < width * height; pixel += 1) {
    const offset = pixel * 4;
    let delta = 0;
    for (let channel = 0; channel < 4; channel += 1) {
      delta = Math.max(delta, Math.abs(source.data[offset + channel] - emitted.data[offset + channel]));
    }
    const differs = delta > policy.visual.channel_delta_threshold;
    changed[pixel] = differs ? 1 : 0;
    if (differs) differingPixels += 1;
    diff.data[offset] = differs ? 255 : Math.round(source.data[offset] * 0.25);
    diff.data[offset + 1] = differs ? 0 : Math.round(source.data[offset + 1] * 0.25);
    diff.data[offset + 2] = differs ? 255 : Math.round(source.data[offset + 2] * 0.25);
    diff.data[offset + 3] = 255;
  }

  const regions = [];
  const queue = [];
  for (let start = 0; start < changed.length; start += 1) {
    if (changed[start] !== 1) continue;
    changed[start] = 2;
    queue.push(start);
    let cursor = 0;
    let minX = start % width;
    let maxX = minX;
    let minY = Math.floor(start / width);
    let maxY = minY;
    let pixels = 0;
    while (cursor < queue.length) {
      const pixel = queue[cursor];
      cursor += 1;
      pixels += 1;
      const x = pixel % width;
      const y = Math.floor(pixel / width);
      minX = Math.min(minX, x);
      maxX = Math.max(maxX, x);
      minY = Math.min(minY, y);
      maxY = Math.max(maxY, y);
      for (const [nextX, nextY] of [[x - 1, y], [x + 1, y], [x, y - 1], [x, y + 1]]) {
        if (nextX < 0 || nextX >= width || nextY < 0 || nextY >= height) continue;
        const next = nextY * width + nextX;
        if (changed[next] === 1) {
          changed[next] = 2;
          queue.push(next);
        }
      }
    }
    queue.length = 0;
    regions.push({ x: minX, y: minY, width: maxX - minX + 1, height: maxY - minY + 1, pixels });
  }
  regions.sort((a, b) => b.pixels - a.pixels || a.y - b.y || a.x - b.x);

  const differingPixelRatio = differingPixels / (width * height);
  const details = [];
  if (differingPixelRatio > policy.visual.max_differing_pixel_ratio) {
    details.push(
      `differing pixel ratio ${differingPixelRatio.toFixed(6)} exceeds ${policy.visual.max_differing_pixel_ratio}`,
    );
  }
  const oversized = regions.filter(
    (region) =>
      region.width > policy.visual.max_diff_region_width ||
      region.height > policy.visual.max_diff_region_height,
  );
  if (oversized.length > 0) {
    details.push(
      `${oversized.length} diff region(s) exceed ${policy.visual.max_diff_region_width}x${policy.visual.max_diff_region_height}px; largest ${oversized[0].width}x${oversized[0].height}px at ${oversized[0].x},${oversized[0].y}`,
    );
  }
  return {
    gate: gate(details),
    metrics: {
      width,
      height,
      differing_pixels: differingPixels,
      differing_pixel_ratio: differingPixelRatio,
      diff_region_count: regions.length,
      largest_regions: regions.slice(0, 5),
    },
    diff: PNG.sync.write(diff),
  };
}

function syntheticPng(width, height, paint) {
  const png = new PNG({ width, height });
  png.data.fill(255);
  paint(png.data, width, height);
  return PNG.sync.write(png);
}

function setPixel(data, width, x, y, rgba = [0, 0, 0, 255]) {
  const offset = (y * width + x) * 4;
  data.set(rgba, offset);
}

function visualComparatorSelfTest() {
  const baseline = syntheticPng(100, 100, () => {});
  const onePixel = syntheticPng(100, 100, (data, width) => setPixel(data, width, 10, 10));
  assert.equal(comparePixels(baseline, onePixel).gate.status, "pass", "isolated one-pixel noise must pass");

  const textShiftSource = syntheticPng(100, 100, (data, width) => {
    for (let y = 20; y < 32; y += 1) for (let x = 20; x < 60; x += 1) setPixel(data, width, x, y);
  });
  const textShiftEmitted = syntheticPng(100, 100, (data, width) => {
    for (let y = 20; y < 32; y += 1) for (let x = 22; x < 62; x += 1) setPixel(data, width, x, y);
  });
  assert.equal(comparePixels(textShiftSource, textShiftEmitted).gate.status, "fail", "2px text shift must fail");

  const border = syntheticPng(100, 100, (data, width) => {
    for (let x = 20; x < 50; x += 1) {
      setPixel(data, width, x, 20);
      setPixel(data, width, x, 49);
    }
    for (let y = 20; y < 50; y += 1) {
      setPixel(data, width, 20, y);
      setPixel(data, width, 49, y);
    }
  });
  assert.equal(comparePixels(baseline, border).gate.status, "fail", "border regression must fail");

  const color = syntheticPng(100, 100, (data, width) => {
    for (let y = 20; y < 50; y += 1) {
      for (let x = 20; x < 50; x += 1) setPixel(data, width, x, y, [254, 254, 254, 255]);
    }
  });
  assert.equal(comparePixels(baseline, color).gate.status, "fail", "color regression must fail");
}

function comparatorSelfTest() {
  const baseline = [
    {
      id: "node[0]",
      path: "",
      tag: "body",
      kind: "box",
      has_text: false,
      styles: Object.fromEntries(
        [...policy.exact_computed_styles, ...policy.numeric_computed_styles].map((key) => [
          key,
          policy.numeric_computed_styles.includes(key) ? "10px" : "same",
        ]),
      ),
      geometry: { x: 0, y: 0, width: 100, height: 100 },
    },
  ];
  const styleMutation = structuredClone(baseline);
  styleMutation[0].styles.display = "grid";
  const styleResult = compareObservations(baseline, styleMutation);
  assert.equal(styleResult.computed_style.status, "fail");
  assert.match(styleResult.computed_style.details[0], /node\[0\]\.style\.display/);

  const geometryMutation = structuredClone(baseline);
  geometryMutation[0].geometry.width += policy.numeric_tolerance_px + 0.1;
  const geometryResult = compareObservations(baseline, geometryMutation);
  assert.equal(geometryResult.geometry.status, "fail");
  assert.match(geometryResult.geometry.details[0], /node\[0\]\.geometry\.width/);

  const toleratedMutation = structuredClone(baseline);
  toleratedMutation[0].styles.width = `${10 + policy.numeric_tolerance_px}px`;
  assert.equal(compareObservations(baseline, toleratedMutation).computed_style.status, "pass");
}

assert.equal(packageJson.devDependencies.playwright, policy.playwright_version);
comparatorSelfTest();
visualComparatorSelfTest();
const fonts = await fontEnvironment();
const manifest = await readFile(path.join(repoRoot, "fixtures", "v0.1", "manifest.toml"), "utf8");
const scenes = sceneEntries(manifest);

const browser = await chromium.launch({
  headless: true,
  args: [
    "--disable-background-networking",
    "--disable-default-apps",
    "--disable-extensions",
    "--disable-renderer-backgrounding",
    "--font-render-hinting=none",
    "--force-color-profile=srgb",
    "--hide-scrollbars",
  ],
});

try {
  assert.equal(
    browser.version(),
    policy.browser_version,
    "browser revision differs from browser-policy.json; run the pinned Playwright browser",
  );
  const context = await browser.newContext({
    viewport: policy.viewport,
    deviceScaleFactor: policy.device_scale_factor,
    locale: policy.locale,
    timezoneId: policy.timezone,
    colorScheme: policy.color_scheme,
    reducedMotion: policy.reduced_motion,
    javaScriptEnabled: false,
    serviceWorkers: "block",
  });
  await context.route("**/*", (route) => route.abort("blockedbyclient"));

  const captures = [];
  for (const scene of scenes) {
    const sceneRoot = path.join(repoRoot, "fixtures", "v0.1", scene.dir);
    const sourceHtml = await readFile(path.join(sceneRoot, "source.html"), "utf8");
    const sourceCss = await readFile(path.join(sceneRoot, "source.css"), "utf8");
    const emittedHtml = await readFile(path.join(sceneRoot, "expected", "roundtrip.html"), "utf8");
    const emittedCss = await readFile(path.join(sceneRoot, "expected", "roundtrip.css"), "utf8");

    const source = await captureDeterministically(
        context,
        inlineDocument(sourceHtml, sourceCss, fonts.css),
        `${scene.id} source`,
      );
    const emitted = await captureDeterministically(
        context,
        inlineDocument(emittedHtml, emittedCss, fonts.css),
        `${scene.id} emitted`,
      );
    const comparison = compareObservations(source.evidence.observations, emitted.evidence.observations);
    const visual = comparePixels(source.screenshot, emitted.screenshot);
    comparison.visual = visual.gate;
    comparison.visual.metrics = visual.metrics;
    assert.equal(comparison.identity.status, "pass", `${scene.id}: ${comparison.identity.details.join("\n")}`);
    assert.equal(
      comparison.computed_style.status,
      "pass",
      `${scene.id}: ${comparison.computed_style.details.join("\n")}`,
    );
    assert.equal(comparison.geometry.status, "pass", `${scene.id}: ${comparison.geometry.details.join("\n")}`);

    if (comparison.visual.status === "fail") {
      const artifactRoot = optionPath("--artifacts");
      if (artifactRoot) {
        const sceneArtifacts = path.join(artifactRoot, scene.id);
        await mkdir(sceneArtifacts, { recursive: true });
        await Promise.all([
          writeFile(path.join(sceneArtifacts, "source.png"), source.screenshot),
          writeFile(path.join(sceneArtifacts, "emitted.png"), emitted.screenshot),
          writeFile(path.join(sceneArtifacts, "diff.png"), visual.diff),
          writeFile(
            path.join(sceneArtifacts, "failure.json"),
            `${JSON.stringify({ scene_id: scene.id, visual: comparison.visual }, null, 2)}\n`,
          ),
        ]);
      }
    }
    assert.equal(comparison.visual.status, "pass", `${scene.id}: ${comparison.visual.details.join("\n")}`);

    captures.push({
      scene_id: scene.id,
      source: source.evidence,
      emitted: emitted.evidence,
      comparison,
    });
  }
  await context.close();

  const evidence = {
    schema_version: 2,
    environment: {
      ...policy,
      browser_version: browser.version(),
      font_sha256: fonts.hashes,
    },
    scenes: captures,
  };
  const serialized = `${JSON.stringify(evidence, null, 2)}\n`;
  const destination = outputPath();
  if (destination) {
    await writeFile(destination, serialized);
  } else {
    process.stdout.write(serialized);
  }
} finally {
  await browser.close();
}
