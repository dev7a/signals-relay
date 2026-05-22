import { createRequire } from "node:module";
import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const siteRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const htmlToImageRoot = dirname(require.resolve("html-to-image/package.json"));
const localPublicRoot = resolve(siteRoot, "public", "local");

mkdirSync(localPublicRoot, { recursive: true });
copyFileSync(resolve(siteRoot, "local", "social-preview.html"), resolve(localPublicRoot, "social-preview.html"));
copyFileSync(resolve(htmlToImageRoot, "dist", "html-to-image.js"), resolve(localPublicRoot, "html-to-image.js"));
copyFileSync(resolve(htmlToImageRoot, "dist", "html-to-image.js.map"), resolve(localPublicRoot, "html-to-image.js.map"));

console.log("Local social preview helper: /local/social-preview.html");
