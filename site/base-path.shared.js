/**
 * @param {string | undefined} value
 * @returns {string}
 */
export function normalizeBasePath(value) {
  if (!value || value === "/") return "";
  const trimmed = value.replace(/\/+$/, "");
  return trimmed.startsWith("/") ? trimmed : `/${trimmed}`;
}

/**
 * @param {{
 *   explicit?: string | undefined;
 *   githubActions?: string | undefined;
 *   githubRepository?: string | undefined;
 * }} [options]
 * @returns {string}
 */
export function resolveBasePath({
  explicit = process.env.NEXT_PUBLIC_BASE_PATH,
  githubActions = process.env.GITHUB_ACTIONS,
  githubRepository = process.env.GITHUB_REPOSITORY,
} = {}) {
  if (explicit !== undefined) {
    return normalizeBasePath(explicit);
  }

  if (githubActions === "true" && githubRepository) {
    const repo = githubRepository.split("/")[1];
    if (repo) return `/${repo}`;
  }

  return "";
}

/**
 * @param {string} html
 * @returns {string}
 */
export function inferBuiltBasePath(html) {
  const match = html.match(/(?:href|src)="(\/[^"]*?)\/_next\//);
  return normalizeBasePath(match?.[1] ?? "");
}
