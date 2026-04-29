import { readFileSync } from "node:fs";
import { createMDX } from "fumadocs-mdx/next";
import { resolveBasePath } from "./base-path.shared.js";

const withMDX = createMDX();
const basePath = resolveBasePath();
const cargoToml = readFileSync(new URL("../Cargo.toml", import.meta.url), "utf8");
const versionMatch = cargoToml.match(/^version = "([^"]+)"$/m);

if (!versionMatch) {
  throw new Error("Unable to read Signals Relay version from Cargo.toml");
}

/** @type {import("next").NextConfig} */
const config = {
  output: "export",
  reactStrictMode: true,
  trailingSlash: true,
  basePath,
  images: {
    unoptimized: true,
  },
  env: {
    NEXT_PUBLIC_BASE_PATH: basePath,
    NEXT_PUBLIC_SIGNALS_RELAY_VERSION: versionMatch[1],
  },
};

export default withMDX(config);
