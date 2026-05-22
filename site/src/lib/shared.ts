import { resolveBasePath } from "../../base-path.shared.js";

export const appName = "Signals Relay";
export const docsRoute = "/docs";

export const gitConfig = {
  user: "dev7a",
  repo: "signals-relay",
  branch: "main",
};

export const githubUrl = `https://github.com/${gitConfig.user}/${gitConfig.repo}`;

export const siteOrigin = (process.env.SITE_ORIGIN ?? "https://dev7a.github.io/signals-relay")
  .replace(/\/+$/, "");

export const basePath = resolveBasePath({
  explicit: process.env.NEXT_PUBLIC_BASE_PATH,
});

export const signalsRelayVersion = process.env.NEXT_PUBLIC_SIGNALS_RELAY_VERSION ?? "0.0.0";

function formatEditionDate(date: Date) {
  const y = date.getUTCFullYear();
  const m = String(date.getUTCMonth() + 1).padStart(2, "0");
  const day = String(date.getUTCDate()).padStart(2, "0");
  return `${y}.${m}.${day}`;
}

// Static export labels the site edition by build date.
export const siteEditionLabel = formatEditionDate(new Date());

export function withBasePath(path: string) {
  if (!path.startsWith("/") || !basePath || path.startsWith(`${basePath}/`) || path === basePath) {
    return path;
  }
  return `${basePath}${path}`;
}

export function docsHref(path = "") {
  if (!path) {
    return `${docsRoute}/`;
  }
  return `${docsRoute}/${path.replace(/^\/+|\/+$/g, "")}/`;
}
