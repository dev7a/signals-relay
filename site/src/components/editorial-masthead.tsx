import Link from "next/link";
import type { ReactNode } from "react";
import { ThemeToggle } from "@/components/theme-toggle";
import { githubUrl, signalsRelayVersion } from "@/lib/shared";

const editionLabel = (() => {
  const d = new Date();
  const y = d.getUTCFullYear();
  const m = String(d.getUTCMonth() + 1).padStart(2, "0");
  const day = String(d.getUTCDate()).padStart(2, "0");
  return `${y}.${m}.${day}`;
})();

function SignalsMark() {
  return (
    <svg aria-hidden="true" className="masthead__mark" viewBox="0 0 64 64">
      <defs>
        <linearGradient id="masthead-mark-grad" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0%" stopColor="#dcb960" />
          <stop offset="100%" stopColor="#b88a35" />
        </linearGradient>
      </defs>
      <rect fill="#0e0d0a" height="64" rx="11" width="64" x="0" y="0" />
      <rect fill="none" height="63" rx="10.5" stroke="rgba(200,164,74,0.28)" width="63" x="0.5" y="0.5" />
      <path d="M 20.78 20.84 A 11.52 11.52 0 0 1 20.78 43.16" fill="none" opacity="0.40" stroke="url(#masthead-mark-grad)" strokeLinecap="round" strokeWidth="4.48" />
      <path d="M 22.85 12.78 A 19.84 19.84 0 0 1 22.85 51.22" fill="none" opacity="0.62" stroke="url(#masthead-mark-grad)" strokeLinecap="round" strokeWidth="4.48" />
      <path d="M 24.92 4.72 A 28.16 28.16 0 0 1 24.92 59.28" fill="none" opacity="0.84" stroke="url(#masthead-mark-grad)" strokeLinecap="round" strokeWidth="4.48" />
      <path d="M 48.32 32 L 55.04 32" fill="none" stroke="url(#masthead-mark-grad)" strokeLinecap="round" strokeWidth="4.48" />
      <path d="M 44.80 24.83 L 55.04 32 L 44.80 39.17" fill="none" stroke="url(#masthead-mark-grad)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="4.48" />
    </svg>
  );
}

function GitHubMark() {
  return (
    <svg aria-hidden="true" fill="currentColor" viewBox="0 0 24 24">
      <path d="M12 .3a12 12 0 0 0-3.8 23.4c.6.1.8-.3.8-.6v-2.1c-3.3.7-4-1.6-4-1.6-.6-1.4-1.4-1.8-1.4-1.8-1.1-.7.1-.7.1-.7 1.2.1 1.9 1.3 1.9 1.3 1.1 1.9 2.9 1.3 3.6 1 .1-.8.4-1.3.8-1.6-2.7-.3-5.5-1.3-5.5-5.9 0-1.3.5-2.4 1.3-3.2-.1-.4-.6-1.6.1-3.2 0 0 1-.3 3.3 1.2a11.5 11.5 0 0 1 6 0c2.3-1.5 3.3-1.2 3.3-1.2.7 1.6.2 2.8.1 3.2.8.8 1.3 1.9 1.3 3.2 0 4.6-2.8 5.6-5.5 5.9.4.4.8 1.1.8 2.2v3.3c0 .3.2.7.8.6A12 12 0 0 0 12 .3" />
    </svg>
  );
}

export function EditorialMasthead({
  navLinks,
  activeHref,
  trailing,
}: {
  navLinks: ReadonlyArray<{ href: string; label: string }>;
  activeHref?: string;
  trailing?: ReactNode;
}) {
  return (
    <header className="masthead">
      <div className="masthead__wrap">
        <div className="masthead__row">
          <div className="masthead__left">
            <span>signalsrelay.dev</span>
            <span aria-hidden="true" className="masthead__dot">·</span>
            <span>edition {editionLabel}</span>
          </div>
          <Link aria-label="Signals Relay home" className="masthead__title" href="/">
            <SignalsMark />
            <span><em>Signals</em> Relay</span>
          </Link>
          <div className="masthead__right">
            <span>v{signalsRelayVersion}</span>
            <span aria-hidden="true" className="masthead__dot">·</span>
            <span className="masthead__beta">● beta</span>
          </div>
        </div>
        <div className="masthead__divider" />
        <nav aria-label="Primary" className="masthead__nav">
          {navLinks.map(({ href, label }) => (
            <Link
              className={activeHref === href ? "masthead__navlink masthead__navlink--active" : "masthead__navlink"}
              href={href}
              key={href}
            >
              {label}
            </Link>
          ))}
          <div className="masthead__nav-trailing">
            {trailing}
            <ThemeToggle />
            <a className="masthead__github" href={githubUrl} rel="noopener noreferrer" target="_blank">
              <GitHubMark />
              <span>dev7a/signals-relay</span>
            </a>
          </div>
        </nav>
      </div>
    </header>
  );
}
