import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { buildDocsMetadata, findDocsPage, renderDocsPage } from "@/lib/docs-page";
import { source } from "@/lib/source";

type DocsSlugProps = {
  params: Promise<{
    slug?: string[];
  }>;
};

export default async function Page(props: DocsSlugProps) {
  const params = await props.params;
  const page = findDocsPage(params.slug);
  if (!page) notFound();

  return renderDocsPage(page);
}

export async function generateStaticParams() {
  return source.generateParams();
}

export async function generateMetadata(props: DocsSlugProps): Promise<Metadata> {
  const params = await props.params;
  const page = findDocsPage(params.slug);
  if (!page) notFound();

  return buildDocsMetadata(page);
}
