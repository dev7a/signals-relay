import type { Metadata } from "next";
import { getMDXComponents } from "@/components/mdx";
import { getSourceRepoPath, source, type DocsSourcePage } from "@/lib/source";
import { gitConfig } from "@/lib/shared";
import {
  DocsBody,
  DocsDescription,
  DocsPage,
  DocsTitle,
} from "fumadocs-ui/layouts/docs/page";

export function findDocsPage(slug?: string[]) {
  if (!slug || slug.length === 0) {
    return source.getPages().find((page) => page.path === "index.mdx");
  }

  return source.getPage(slug);
}

export function renderDocsPage(page: DocsSourcePage) {
  const MDX = page.data.body;
  const sourceRepoPath = getSourceRepoPath(page);

  return (
    <DocsPage toc={page.data.toc} full={page.data.full}>
      <DocsTitle>{page.data.title}</DocsTitle>
      <DocsDescription>{page.data.description}</DocsDescription>
      <div className="flex flex-row items-center gap-2 border-b pb-6">
        <a
          className="text-sm text-fd-muted-foreground underline-offset-4 hover:text-fd-foreground hover:underline"
          href={`https://github.com/${gitConfig.user}/${gitConfig.repo}/blob/${gitConfig.branch}/${sourceRepoPath}`}
        >
          View source
        </a>
      </div>
      <DocsBody>
        <MDX components={getMDXComponents()} />
      </DocsBody>
    </DocsPage>
  );
}

export function buildDocsMetadata(page: DocsSourcePage): Metadata {
  return {
    title: page.data.title,
    description: page.data.description,
  };
}
