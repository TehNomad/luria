#!/usr/bin/env node
// Research-only capture tool. NOT Luria production code.
//
// Purpose: capture the JavaScript response bodies actually served by
// https://nebasechanger.neb.com/ using Playwright's response lifecycle, and
// write a sanitized manifest (URL, status, content type, byte length, SHA-256,
// capture timestamp) for every asset saved.
//
// Deliberately NOT done: no Cloudflare bypass, no challenge solving, no
// request-header replay, no cookie/credential reuse from outside the browser.
// The browser is used normally; if the site keeps the app behind a challenge,
// this tool reports that and exits non-zero.
//
// Usage:
//   node capture-js.mjs                                  # fresh headless browser
//   node capture-js.mjs --profile ./browser-profile      # persistent profile,
//                                                        # so a challenge the USER solved earlier persists
//   node capture-js.mjs --no-headless                    # visible window; the user completes
//                                                        # any challenge manually in that window
//   node capture-js.mjs --cdp http://127.0.0.1:9222      # attach to an existing
//                                                        # browser session already at the site
//
// Sanitization: response/request headers, cookies, clearance tokens, consent
// values and request POST bodies are never written. A final self-check aborts
// if any forbidden marker appears in the emitted manifest.

import process from "node:process";
import { mkdir, writeFile, readdir, readFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { chromium } from "playwright";

// ---------------------------------------------------------------- config ----

const args = process.argv.slice(2);
function flag(name) {
  const i = args.indexOf(name);
  return i === -1 ? null : args[i + 1];
}

const targetUrl = flag("--url") ?? "https://nebasechanger.neb.com/";
const outDir = flag("--out") ?? "live-capture";
const cdpUrl = flag("--cdp");
const profileDir = flag("--profile");
const noHeadless = args.includes("--no-headless");
const navTimeout = Number(flag("--timeout") ?? 90_000);
const maxBodyBytes = Number(flag("--max-body") ?? 8_000_000);
const maxDiscoveryRounds = Number(flag("--discovery-rounds") ?? 3);

const waitTimeout = Number(flag("--wait-timeout") ?? 0); // ms to wait for a human to clear a challenge

const target = new URL(targetUrl);

// Third-party and Cloudflare machinery: recorded in the manifest as excluded,
// bodies never saved. These are not app code, and saving them would store
// challenge material.
const EXCLUDE = [
  { re: /^blob:/, reason: "blob: URL, not a fetchable asset" },
  { re: /\/cdn-cgi\/challenge-platform\//, reason: "Cloudflare challenge machinery" },
  { re: /^https:\/\/challenges\.cloudflare\.com\//, reason: "Cloudflare Turnstile" },
  { re: /^https:\/\/static\.cloudflareinsights\.com\//, reason: "Cloudflare Insights beacon" },
  { re: /^https:\/\/cdn\.cookielaw\.org\//, reason: "OneTrust consent SDK (third party)" },
  { re: /^https:\/\/cdn\.onetrust\.com\//, reason: "OneTrust consent SDK (third party)" },
];

// Precise markers: cookie/header *assignments*, not the word "cookie" inside a
// public third-party URL such as cdn.cookielaw.org.
const FORBIDDEN = ["cf_clearance=", "optanonconsent=", "set-cookie", "authorization:", "bearer ", "cookie:", "cf-chl", "cf_clearance"];

function excluded(url) {
  for (const e of EXCLUDE) if (e.re.test(url)) return e.reason;
  return null;
}

function isJsish(url, contentType) {
  const ct = (contentType ?? "").toLowerCase();
  if (/javascript|ecmascript|x-javascript|js/.test(ct)) return true;
  if (/\.(js|mjs|json|map|wasm)(\?|$)/i.test(url)) return true;
  return false;
}

// ------------------------------------------------------- classification ----

const CHALLENGE_MARKERS = [
  "challenge-stage",
  "cf-chl",
  "_cf_chl",
  "Just a moment",
  "challenges.cloudflare.com",
  "turnstile",
  "CLOUDFLAREERROR",
];

function classify(buf, contentType, url) {
  const head = buf.slice(0, 4096).toString("utf8");
  const reasons = [];

  if (buf.length === 0) return { kind: "empty", ok: false, reasons: ["zero-length body"] };

  for (const m of CHALLENGE_MARKERS) {
    if (head.toLowerCase().includes(m.toLowerCase())) {
      reasons.push(`challenge marker: ${m}`);
      return { kind: "cloudflare-challenge", ok: false, reasons };
    }
  }

  // WebAssembly (a worker/module asset): magic 0x00 61 73 6d
  if (buf.length > 8 && buf[0] === 0x00 && buf[1] === 0x61 && buf[2] === 0x73 && buf[3] === 0x6d) {
    return { kind: "wasm", ok: true, reasons: ["wasm magic"] };
  }

  const trimmed = head.trimStart();
  if (/^<(!doctype|html|\/html|script|link|meta|svg)/i.test(trimmed) || /<html[\s>]/i.test(head)) {
    return { kind: "html", ok: false, reasons: ["starts with/contains HTML markup"] };
  }
  if (/^[[{]/.test(trimmed)) {
    // JSON. A .map asset is legitimately JSON (a source map); other JSON is
    // not JavaScript and must not be labelled as such.
    if (/\.map(\?|$)/i.test(url)) return { kind: "sourcemap", ok: true, reasons: ["JSON source map"] };
    return { kind: "json", ok: false, reasons: ["JSON body, not JavaScript"] };
  }

  const jsSignals = ["function", "=>", "var ", "let ", "const ", "return ", "//# sourceMappingURL", "/*!"];
  const hits = jsSignals.filter((s) => head.includes(s));
  const ctJs = /javascript|ecmascript|js/i.test((contentType ?? "").toLowerCase());

  if (ctJs) {
    reasons.push(`content-type is JS (${contentType})`);
    if (hits.length) reasons.push(`JS tokens: ${hits.slice(0, 4).join(", ")}`);
    return { kind: "javascript", ok: true, reasons };
  }
  if (hits.length >= 2) {
    reasons.push(`JS tokens without JS content-type: ${hits.slice(0, 4).join(", ")}`);
    return { kind: "javascript", ok: true, reasons };
  }
  return { kind: "unknown", ok: false, reasons: ["no JS content-type and too few JS tokens"] };
}

// ------------------------------------------------------- verify-only mode ---
// --verify DIR classifies files already on disk with the same rules the capture
// uses, so "saved file" and "verified file" are the same check.
if (args.includes("--verify")) {
  const dir = flag("--verify");
  const rows = [];
  let bad = 0;
  for (const name of await readdir(dir)) {
    if (name === "manifest.json" || name === "shell.html") continue; // tool artifacts, not assets
    if (!/\.(js|mjs|json|map|wasm)$/i.test(name)) continue;
    const buf = await readFile(`${dir}/${name}`);
    const cls = classify(buf, null, name);
    if (!cls.ok) bad += 1;
    rows.push({ name, bytes: buf.length, sha256: createHash("sha256").update(buf).digest("hex"), kind: cls.kind, ok: cls.ok, reasons: cls.reasons });
  }
  console.log(`verify ${dir}: ${rows.length} files, ${rows.length - bad} look like JavaScript, ${bad} do not`);
  for (const r of rows) console.log(`  ${r.bytes}B sha256=${r.sha256} ${r.kind} ok=${r.ok} ${r.name}${r.ok ? "" : "  <- " + r.reasons.join("; ")}`);
  process.exit(bad === 0 ? 0 : 2);
}

// ------------------------------------------------------------- capture ------

const discoverySeeds = args.reduce((acc, a, i) => a === "--seed" ? acc.concat([args[i + 1]]) : acc, []);

const captured = new Map(); // url -> record
const excludedLog = new Map(); // url -> reason
const failures = [];
const startedAt = new Date();
const perAssetAt = new Map(); // url -> ISO timestamp of body read

const launchOpts = {
  headless: !noHeadless,
  timeout: navTimeout,
  // Keep the browser's own defaults otherwise; no header/cookie overrides.
};

let browser;
let closeBrowser;
if (cdpUrl) {
  browser = await chromium.connectOverCDP(cdpUrl);
  closeBrowser = async () => await browser.close();
} else if (profileDir) {
  await mkdir(profileDir, { recursive: true });
  browser = await chromium.launchPersistentContext(profileDir, launchOpts);
  closeBrowser = async () => await browser.close();
} else {
  browser = await chromium.launch(launchOpts);
  closeBrowser = async () => await browser.close();
}

await mkdir(outDir, { recursive: true });
const page = await browser.newPage();

async function onResponse(response) {
  const url = response.url();
  const reason = excluded(url);
  if (reason) {
    excludedLog.set(url, reason);
    return;
  }

  // In this Playwright version headerValue() returns a Promise even inside an
  // event handler, so it must be awaited before any string operation.
  const contentType = await response.headerValue("content-type");
  if (!isJsish(url, contentType)) return; // not a JS-relevant asset

  const status = await response.status();
  if (status >= 300 && status <= 399) return; // redirect: no body available

  try {
    // response.body() awaits the request's completion promise internally
    // (Response2.internalBody does `await this._finishedPromise`), so the body
    // is read only after the response has finished streaming.
    const body = await response.body();
    perAssetAt.set(url, new Date().toISOString());

    if (body.length > maxBodyBytes) {
      failures.push({ url, error: `body ${body.length} bytes exceeds --max-body ${maxBodyBytes}; not saved` });
      return;
    }

    const sha256 = createHash("sha256").update(body).digest("hex");
    const cls = classify(body, contentType, url);
    const path = new URL(url).pathname;
    const base = path.split("/").pop() || "asset";
    const safeBase = base.replace(/[^a-zA-Z0-9._-]/g, "_");
    const filename = `${sha256.slice(0, 12)}-${safeBase}`;

    captured.set(url, {
      url,
      file: filename,
      status,
      contentType,
      resourceType: await response.request().resourceType(),
      bytes: body.length,
      sha256,
      capturedAt: perAssetAt.get(url),
      kind: cls.kind,
      looksLikeJavaScript: cls.ok,
      classifyReasons: cls.reasons,
      body,
    });
  } catch (e) {
    failures.push({ url, error: String(e).split("\n")[0] });
  }
}

const safeErrors = [];
async function safe(label, fn) {
  try {
    return await fn();
  } catch (e) {
    safeErrors.push(`${label}: ${String(e).split("\n")[0]}`);
    return null;
  }
}

page.on("response", onResponse);

let navigationError = null;
try {
  await page.goto(targetUrl, { waitUntil: "domcontentloaded", timeout: navTimeout });
} catch (e) {
  navigationError = String(e).split("\n")[0];
}

// With --wait-timeout the caller can solve a challenge in the open window; the
// tool waits until the app itself is populated instead of assuming the load
// finished.
if (waitTimeout > 0) {
  await safe(`waitForApp (up to ${waitTimeout}ms)`, () =>
    page.waitForFunction(
      "() => { const a = document.querySelector('#app'); return a && a.children.length > 0 }",
      { timeout: waitTimeout },
    ));
}
await safe("waitForLoadState after goto", () => page.waitForLoadState("networkidle", { timeout: navTimeout }));

// Did the app actually initialize, or did we stop at a Cloudflare challenge?
let appReady = false;
let challengeSeen = false;
try {
  appReady = await page.evaluate(
    `(() => { const a = document.querySelector('#app'); return !!a && a.children.length > 0 })()`,
  );
} catch { /* page may have navigated away */ }
try {
  const text = await page.evaluate("document.documentElement?.outerHTML ?? ''");
  challengeSeen = CHALLENGE_MARKERS.some((m) => text.toLowerCase().includes(m.toLowerCase()));
} catch { /* ignore */ }

// --------------------------------------------- discovery of more assets -----
// The SPA shell references its module bundle; the bundle can reference further
// /assets/ files and source maps. Fetch those in the SAME browser context
// (normal navigation, no header replay) so their bodies are captured too.

function refsFromText(text) {
  const found = new Set();
  for (const m of text.matchAll(/["'`](\/assets\/[A-Za-z0-9._-]+)["'`]/g)) found.add(m[1]);
  for (const m of text.matchAll(/sourceMappingURL=([^\s'"`)]+)/g)) found.add(m[1]);
  for (const m of text.matchAll(/["'`](\/[A-Za-z0-9._/-]*\.(?:js|mjs|json|wasm))["'`]/g)) found.add(m[1]);
  return [...found];
}

const discovered = new Set(discoverySeeds);

// The SPA shell is the authoritative source for which bundle the site serves
// right now. Save it separately from the JS assets: it is a discovery artifact,
// not a JavaScript asset, and must not be counted as one.
let shell = null;
const shellHtml = await safe("page.content (shell HTML)", () => page.content());
if (shellHtml) {
  const buf = Buffer.from(shellHtml, "utf8");
  const cls = classify(buf, "text/html", page.url());
  shell = {
    file: null,
    bytes: buf.length,
    sha256: createHash("sha256").update(buf).digest("hex"),
    capturedAt: new Date().toISOString(),
    url: page.url(),
    kind: cls.kind,
    saved: false,
    refs: [],
  };
  const challenged = CHALLENGE_MARKERS.some((m) => shellHtml.toLowerCase().includes(m.toLowerCase()));
  if (!challenged) {
    // A Cloudflare challenge document is session material, not the app shell:
    // it is hashed and reported, but never written to disk.
    shell.file = "shell.html";
    shell.refs = refsFromText(shellHtml);
    shell.saved = true;
    await writeFile(`${outDir}/shell.html`, buf);
  }
  for (const ref of shell.refs) {
    const abs = new URL(ref, page.url()).toString();
    if (new URL(abs).origin === target.origin) discovered.add(abs);
  }
}

for (let round = 0; round < maxDiscoveryRounds; round += 1) {
  for (const rec of [...captured.values()]) {
    if (rec.kind !== "javascript" && rec.kind !== "sourcemap") continue;
    for (const ref of refsFromText(rec.body.toString("utf8"))) {
      const abs = new URL(ref, target.origin + "/").toString();
      if (new URL(abs).origin !== target.origin) continue; // same-origin only
      discovered.add(abs);
    }
  }
  const todo = [...discovered].filter((u) => !captured.has(u) && !excluded(u));
  if (!todo.length) break;
  for (const url of todo) {
    await safe(`goto ${url}`, () => page.goto(url, { waitUntil: "domcontentloaded", timeout: navTimeout }));
    await safe(`waitForLoadState after ${url}`, () => page.waitForLoadState("networkidle", { timeout: navTimeout }));
  }
}

// --------------------------------------------------------------- write out --

const assets = [];
for (const rec of captured.values()) {
  const { body, ...meta } = rec;
  await writeFile(`${outDir}/${meta.file}`, body);
  assets.push(meta);
}

const appBundlePattern = /\/assets\/index\.[0-9a-f]{8}\.js/;
const appAssets = assets.filter((a) => appBundlePattern.test(a.url));
const missing = [...discovered].filter((u) => !captured.has(u) && !excluded(u));

const manifest = {
  capturedAt: new Date().toISOString(),
  startedAt: startedAt.toISOString(),
  page: targetUrl,
  browser: {
    mode: cdpUrl ? "connectOverCDP" : profileDir ? "persistent-profile" : "fresh-headless",
    headless: !noHeadless,
    cdpUrl: cdpUrl ?? null,
    profileDir: profileDir ?? null,
  },
  outcome: {
    navigationError,
    safeErrors,
    appReady,
    challengeSeen,
    shell,
    appBundleRequested: appAssets.length > 0,
    appBundles: appAssets.map((a) => a.url),
    assetCount: assets.length,
    missing,
  },
  assets,
  excluded: [...excludedLog.entries()].map(([url, reason]) => ({ url, reason })),
  failures,
  sanitization: {
    statement: "No request or response headers, cookies, clearance tokens, consent values, or request POST bodies are recorded.",
    fieldsRecorded: ["url", "file", "status", "contentType", "resourceType", "bytes", "sha256", "capturedAt", "kind", "looksLikeJavaScript", "classifyReasons"],
    checkedMarkers: FORBIDDEN,
    checkedBlob: "manifest minus this sanitization block",
  },
};

const serialized = JSON.stringify(manifest, null, 2);
const sanitizedForCheck = { ...manifest };
delete sanitizedForCheck.sanitization;
const serializedForCheck = JSON.stringify(sanitizedForCheck).toLowerCase();
const leaked = FORBIDDEN.filter((m) => serializedForCheck.includes(m));
if (leaked.length) {
  console.log(`SANITIZATION CHECK FAILED: forbidden markers present: ${leaked.join(", ")}`);
  await closeBrowser();
  process.exit(3);
}

await writeFile(`${outDir}/manifest.json`, serialized);

// ------------------------------------------------------------- report -------

const jsOk = assets.filter((a) => a.looksLikeJavaScript);
const jsBad = assets.filter((a) => !a.looksLikeJavaScript);

console.log(`page: ${targetUrl}`);
console.log(`shell: ${shell ? shell.bytes + "B sha256=" + shell.sha256 + " refs=" + shell.refs.length : "not captured"}`);
if (shell && shell.refs.length) {
  console.log("shell refs:");
  for (const r of shell.refs) console.log(`  ${new URL(r, page.url()).toString()}`);
}
console.log(`app bundle requested: ${appAssets.length > 0}`);
console.log(`assets saved: ${assets.length} (${jsOk.length} look like JavaScript)`);
for (const a of assets) {
  console.log(`  ${a.status} ${a.bytes}B  sha256=${a.sha256}  ${a.contentType ?? "-"}  ${a.kind}  ok=${a.looksLikeJavaScript}  ${a.url}`);
}
if (jsBad.length) {
  console.log("VERIFICATION FAILURES (saved but not JavaScript):");
  for (const a of jsBad) console.log(`  ${a.url}: ${a.kind} — ${a.classifyReasons.join("; ")}`);
}
if (missing.length) {
  console.log("DISCOVERED BUT NOT CAPTURED:");
  for (const u of missing) console.log(`  ${u}`);
}
if (failures.length) {
  console.log("FAILURES:");
  for (const f of failures) console.log(`  ${f.url}: ${f.error}`);
}
if (navigationError) console.log(`navigation: ${navigationError}`);
if (safeErrors.length) console.log("STEP ERRORS:");
for (const e of safeErrors) console.log(`  ${e}`);
console.log(`excluded (not saved): ${excludedLog.size}`);
console.log(`out: ${outDir}`);

await closeBrowser();

// Requirement: if the live app bundle could not be obtained, exit non-zero so
// the caller stops rather than substituting an archived build.
process.exit(appAssets.length > 0 && jsOk.length > 0 ? 0 : 2);
