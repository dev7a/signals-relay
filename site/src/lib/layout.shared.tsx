import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";
import { githubUrl } from "@/lib/shared";

function NavMark() {
  const gold = "#c8a44a";
  return (
    <svg aria-hidden="true" height="22" viewBox="0 0 64 64" width="22">
      <rect fill="#0e0d0a" height="64" rx="11" width="64" x="0" y="0" />
      <rect fill="none" height="63" rx="10.5" stroke="rgba(200,164,74,0.28)" width="63" x="0.5" y="0.5" />
      <path d="M 20.78 20.84 A 11.52 11.52 0 0 1 20.78 43.16" fill="none" opacity="0.40" stroke={gold} strokeLinecap="round" strokeWidth="4.48" />
      <path d="M 22.85 12.78 A 19.84 19.84 0 0 1 22.85 51.22" fill="none" opacity="0.62" stroke={gold} strokeLinecap="round" strokeWidth="4.48" />
      <path d="M 24.92 4.72 A 28.16 28.16 0 0 1 24.92 59.28" fill="none" opacity="0.84" stroke={gold} strokeLinecap="round" strokeWidth="4.48" />
      <path d="M 48.32 32 L 55.04 32" fill="none" stroke={gold} strokeLinecap="round" strokeWidth="4.48" />
      <path d="M 44.80 24.83 L 55.04 32 L 44.80 39.17" fill="none" stroke={gold} strokeLinecap="round" strokeLinejoin="round" strokeWidth="4.48" />
    </svg>
  );
}

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <span style={{ display: "inline-flex", alignItems: "center", gap: "8px", fontFamily: "var(--font-serif), 'Instrument Serif', serif", fontSize: "1.2rem", letterSpacing: "-0.02em" }}>
          <NavMark />
          <span><em style={{ fontStyle: "italic", color: "var(--signals-gold)" }}>Signals</em> Relay</span>
        </span>
      ),
    },
    githubUrl,
  };
}
