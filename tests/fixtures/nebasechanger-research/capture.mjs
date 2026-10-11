
import { chromium } from "playwright";
import { mkdir, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";

const origin = "https://nebasechanger.neb.com/";
const out = "nebasechanger-js";

await mkdir(out, { recursive: true });

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();
const scripts = new Map();
const failures = [];

page.on("response", async (response) => {
  const url = response.url();
  const type = response.request().resourceType();

  if (type !== "script") return;

  try {
    const body = await response.body();
    const hash = createHash("sha256")
      .update(body).digest("hex");

    const path = new URL(url).pathname;
    const base = path.split("/").pop() || "script.js";
    const filename = `${hash.slice(0, 12)}-${base.replace(/[^a-zA-Z0-9._-]/g, "_")}`;

    scripts.set(url, {
      filename,
      sha256: hash,
      bytes: body.length,
      body,
      status: response.status(),
    });
  } catch (e) {
    failures.push({ url, error: String(e) });
  }
});

try {
  await page.goto(origin, {
    waitUntil: "networkidle",
    timeout: 60000,
  });
} catch (e) {
  console.log("Navigation did not reach network idle:", String(e));
}

for (const item of scripts.values()) {
  await writeFile(`${out}/${item.filename}`, item.body);
}

const manifest = [...scripts.entries()].map(([url, item]) => ({
  url,
  file: item.filename,
  sha256: item.sha256,
  bytes: item.bytes,
  status: item.status,
}));

await writeFile(
  `${out}/manifest.json`,
  JSON.stringify({ page: origin, scripts: manifest, failures }, null, 2)
);

console.log(`Captured ${scripts.size} JavaScript responses.`);
console.log(`Output directory: ${out}`);
console.log(`See ${out}/manifest.json for URLs and hashes.`);

await browser.close();

