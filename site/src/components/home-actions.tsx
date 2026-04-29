"use client";

import { GitFork, Moon, Sun } from "lucide-react";
import { useEffect, useState } from "react";
import { useTheme } from "fumadocs-ui/provider/base";

export function HomeActions({ githubUrl }: { githubUrl: string }) {
  const { resolvedTheme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);
  const currentTheme = mounted ? resolvedTheme : "dark";

  useEffect(() => {
    setMounted(true);
  }, []);

  return (
    <div className="flex items-center gap-2">
      <a
        aria-label="Open Signals Relay on GitHub"
        className="inline-flex h-10 items-center justify-center gap-2 rounded-md border border-white/15 bg-white/[0.08] px-3 text-sm font-medium text-white shadow-sm shadow-black/20 backdrop-blur transition hover:bg-white/[0.14]"
        href={githubUrl}
        rel="noreferrer"
        target="_blank"
      >
        <GitFork aria-hidden="true" className="h-4 w-4" />
        GitHub
      </a>
      <div
        aria-label="Theme"
        className="inline-flex h-10 items-center gap-1 rounded-md border border-white/15 bg-white/[0.08] p-1 text-white shadow-sm shadow-black/20 backdrop-blur"
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
