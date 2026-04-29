import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const siteRoot = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(siteRoot, "..", "..");
const script = resolve(repoRoot, "scripts", "generate_fumadocs_site.py");

if (!existsSync(script)) {
  throw new Error(`Missing generator script: ${script}`);
}

for (const python of ["python3", "python"]) {
  const result = spawnSync(
    python,
    [
      script,
      "--repo-root",
      repoRoot,
      "--content-root",
      resolve(repoRoot, "site", "content", "docs"),
      "--public-root",
      resolve(repoRoot, "site", "public"),
    ],
    { stdio: "inherit" },
  );

  if (result.error && result.error.code === "ENOENT") {
    continue;
  }

  process.exit(result.status ?? 1);
}

throw new Error("Unable to find python3 or python on PATH.");
