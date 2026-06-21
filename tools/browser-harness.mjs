import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import { chromium } from "playwright";

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
  `;
  const style = `<style data-semui-browser-proof>${deterministicCss}\n${css}</style>`;
  const linked = /<link\b[^>]*rel=["']stylesheet["'][^>]*>/i;
  if (linked.test(html)) return html.replace(linked, style);
  return html.replace(/<\/head>/i, `${style}</head>`);
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
  const screenshot = await page.screenshot({
    animations: "disabled",
    caret: "hide",
    fullPage: false,
    type: "png",
  });
  const dom = await page.content();
  await page.close();
  assert.deepEqual(externalRequests, [], `network request escaped isolation: ${externalRequests}`);
  return {
    screenshot_sha256: sha256(screenshot),
    dom_sha256: sha256(dom),
  };
}

async function captureDeterministically(context, document, label) {
  const first = await capture(context, document);
  const second = await capture(context, document);
  assert.deepEqual(first, second, `${label} capture changed between identical renders`);
  return first;
}

assert.equal(packageJson.devDependencies.playwright, policy.playwright_version);
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

    captures.push({
      scene_id: scene.id,
      source: await captureDeterministically(
        context,
        inlineDocument(sourceHtml, sourceCss, fonts.css),
        `${scene.id} source`,
      ),
      emitted: await captureDeterministically(
        context,
        inlineDocument(emittedHtml, emittedCss, fonts.css),
        `${scene.id} emitted`,
      ),
    });
  }
  await context.close();

  const evidence = {
    schema_version: 1,
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
