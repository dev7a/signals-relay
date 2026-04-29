import { docs } from "collections/server";
import { type InferPageType, loader } from "fumadocs-core/source";
import { docsRoute } from "@/lib/shared";

export const source = loader({
  baseUrl: docsRoute,
  source: docs.toFumadocsSource(),
});

export type DocsSourcePage = InferPageType<typeof source>;

export function getSourceRepoPath(page: DocsSourcePage) {
  if (page.path === "index.mdx") return "docs/README.md";
  return `docs/${page.path.replace(/\.mdx$/, ".md")}`;
}
