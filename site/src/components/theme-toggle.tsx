"use client";

import { Moon, Sun } from "lucide-react";
import { useEffect, useState } from "react";
import { useTheme } from "fumadocs-ui/provider/base";

export function ThemeToggle() {
  const { resolvedTheme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const next = resolvedTheme === "dark" ? "light" : "dark";
  const Icon = mounted && resolvedTheme === "dark" ? Sun : Moon;
  const label = mounted ? `Switch to ${next} theme` : "Toggle theme";

  return (
    <button
      aria-label={label}
      className="masthead__theme"
      onClick={() => setTheme(next)}
      title={label}
      type="button"
    >
      <Icon aria-hidden="true" />
    </button>
  );
}
