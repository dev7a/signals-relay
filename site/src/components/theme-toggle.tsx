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

  const currentTheme = mounted ? resolvedTheme : undefined;
  const next = currentTheme === "dark" ? "light" : "dark";
  const Icon = currentTheme === "dark" ? Sun : Moon;
  const label = currentTheme ? `Switch to ${next} theme` : "Toggle theme";

  return (
    <button
      aria-label={label}
      className="masthead__theme"
      disabled={!currentTheme}
      onClick={() => setTheme(next)}
      title={label}
      type="button"
    >
      <Icon aria-hidden="true" />
    </button>
  );
}
