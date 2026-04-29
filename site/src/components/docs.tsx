import type { ComponentPropsWithoutRef } from "react";
import { withBasePath } from "@/lib/shared";

export function DocsImage({
  alt = "",
  className,
  src,
  ...props
}: ComponentPropsWithoutRef<"img">) {
  const resolvedSrc = typeof src === "string" ? withBasePath(src) : src;
  const classes = ["h-auto max-w-full rounded-md border border-fd-border", className]
    .filter(Boolean)
    .join(" ");

  return <img alt={alt} className={classes} src={resolvedSrc} {...props} />;
}
