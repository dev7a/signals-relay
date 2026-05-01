"use client";

import { Moon, Sun } from "lucide-react";
import { useEffect, useState } from "react";
import { useTheme } from "fumadocs-ui/provider/base";

function GitHubMark({ className }: { className?: string }) {
  return (
    <svg
      aria-hidden="true"
      className={className}
      fill="currentColor"
      viewBox="0 0 24 24"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path d="M12 .5C5.65.5.5 5.65.5 12c0 5.08 3.29 9.39 7.86 10.91.58.11.79-.25.79-.55 0-.27-.01-1.16-.02-2.11-3.2.7-3.87-1.37-3.87-1.37-.52-1.32-1.27-1.67-1.27-1.67-1.04-.71.08-.7.08-.7 1.15.08 1.76 1.18 1.76 1.18 1.02 1.76 2.69 1.25 3.34.96.1-.74.4-1.25.72-1.54-2.55-.29-5.24-1.27-5.24-5.66 0-1.25.45-2.27 1.18-3.07-.12-.29-.51-1.46.11-3.04 0 0 .96-.31 3.15 1.18.92-.26 1.9-.39 2.88-.39.98 0 1.96.13 2.88.39 2.19-1.49 3.15-1.18 3.15-1.18.62 1.58.23 2.75.11 3.04.74.8 1.18 1.82 1.18 3.07 0 4.4-2.69 5.37-5.25 5.65.41.36.78 1.07.78 2.16 0 1.56-.01 2.81-.01 3.19 0 .31.21.67.8.55C20.21 21.39 23.5 17.08 23.5 12 23.5 5.65 18.35.5 12 .5Z" />
    </svg>
  );
}

export function HomeActions({ githubUrl }: { githubUrl: string }) {
  const { resolvedTheme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);
  const currentTheme = mounted ? resolvedTheme : undefined;

  useEffect(() => {
    setMounted(true);
  }, []);

  return (
    <div className="flex items-center gap-2">
      <a
        aria-label="Open Signals Relay on GitHub"
        className="inline-flex h-10 items-center justify-center gap-2 rounded-md border border-white/15 bg-white/[0.08] px-3 text-sm font-medium text-white shadow-sm shadow-black/20 backdrop-blur transition hover:bg-white/[0.14]"
        href={githubUrl}
        rel="noopener noreferrer"
        target="_blank"
      >
        <GitHubMark className="h-4 w-4" />
        <span className="home-actions__github-label">GitHub</span>
      </a>
      <div
        aria-label="Theme"
        className="inline-flex h-10 items-center gap-1 rounded-md border border-white/15 bg-white/[0.08] p-1 text-white shadow-sm shadow-black/20 backdrop-blur"
        role="group"
      >
        <button
          aria-label="Use light theme"
          aria-pressed={currentTheme === "light"}
          className="flex h-8 w-8 items-center justify-center rounded-[5px] text-white/70 transition hover:bg-white/[0.12] hover:text-white aria-pressed:bg-white aria-pressed:text-neutral-950"
          onClick={() => setTheme("light")}
          title="Light theme"
          type="button"
        >
          <Sun aria-hidden="true" className="h-4 w-4" />
        </button>
        <button
          aria-label="Use dark theme"
          aria-pressed={currentTheme === "dark"}
          className="flex h-8 w-8 items-center justify-center rounded-[5px] text-white/70 transition hover:bg-white/[0.12] hover:text-white aria-pressed:bg-white aria-pressed:text-neutral-950"
          onClick={() => setTheme("dark")}
          title="Dark theme"
          type="button"
        >
          <Moon aria-hidden="true" className="h-4 w-4" />
        </button>
      </div>
    </div>
  );
}
