import { createServer } from "node:http";
import { existsSync, readFileSync, statSync } from "node:fs";
import { extname, join, normalize, resolve, sep } from "node:path";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { inferBuiltBasePath, normalizeBasePath, resolveBasePath } from "../base-path.shared.js";

const siteRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const outRoot = resolve(siteRoot, "out");
const port = Number.parseInt(process.env.PORT ?? "3000", 10);

if (!existsSync(outRoot)) {
  throw new Error("Missing site/out. Run `pnpm build` before `pnpm start`.");
}

const builtIndex = readFileSync(join(outRoot, "index.html"), "utf8");
const basePath =
  normalizeBasePath(process.env.NEXT_PUBLIC_BASE_PATH) ||
  inferBuiltBasePath(builtIndex) ||
  resolveBasePath();

const mimeTypes = {
  ".css": "text/css; charset=utf-8",
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".txt": "text/plain; charset=utf-8",
  ".wasm": "application/wasm",
};

function fileForUrl(urlPath) {
  let path;
  try {
    path = decodeURIComponent(urlPath);
  } catch {
    return null;
  }
  if (basePath) {
    if (path === "/") return { redirect: `${basePath}/` };
    if (path === basePath) return { redirect: `${basePath}/` };
    if (path.startsWith(`${basePath}/`)) {
      path = path.slice(basePath.length) || "/";
    } else if (
      !path.startsWith("/_next/") &&
      !path.startsWith("/docs-assets/") &&
      path !== "/api/search"
    ) {
      return null;
    }
  }

  const normalizedPath = normalize(path).replace(/^(\.\.(\/|\\|$))+/, "");
  const candidate = resolve(outRoot, `.${normalizedPath}`);
  if (!candidate.startsWith(outRoot + sep) && candidate !== outRoot) {
    return null;
  }

  if (existsSync(candidate) && statSync(candidate).isFile()) return { file: candidate };
  if (existsSync(candidate) && statSync(candidate).isDirectory()) {
    const index = join(candidate, "index.html");
    if (existsSync(index)) return { file: index };
  }

  const html = `${candidate}.html`;
  if (existsSync(html)) return { file: html };

  return null;
}

const server = createServer((request, response) => {
  const url = new URL(request.url ?? "/", `http://${request.headers.host ?? "localhost"}`);
  const result = fileForUrl(url.pathname);

  if (!result) {
    response.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
    response.end("Not found");
    return;
  }

  if ("redirect" in result) {
    response.writeHead(302, { location: result.redirect });
    response.end();
    return;
  }

  const type =
    url.pathname.endsWith("/api/search") || url.pathname.endsWith("/api/search/")
      ? "application/json; charset=utf-8"
      : (mimeTypes[extname(result.file)] ?? "application/octet-stream");
  response.writeHead(200, { "content-type": type });
  response.end(readFileSync(result.file));
});

server.listen(port, () => {
  const path = basePath || "";
  console.log(`Serving ${outRoot} at http://localhost:${port}${path}/`);
});
