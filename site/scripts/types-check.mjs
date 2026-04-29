import { rmSync } from "node:fs";
import { spawnSync } from "node:child_process";

for (const path of [".next/types", ".next/dev/types"]) {
  rmSync(path, { recursive: true, force: true });
}

const commands = [
  ["pnpm", ["run", "generate:content"]],
  ["pnpm", ["exec", "fumadocs-mdx"]],
  ["pnpm", ["exec", "next", "typegen"]],
  ["pnpm", ["exec", "tsc", "--noEmit"]],
];

for (const [command, args] of commands) {
  const result = spawnSync(command, args, { stdio: "inherit" });
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
