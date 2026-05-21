import Link from "next/link";
import { ArrowRight } from "lucide-react";
import type { ReactNode } from "react";
import { EditorialMasthead } from "@/components/editorial-masthead";
import { SchematicDiagram } from "@/components/schematic-diagram";
import { docsHref, githubUrl, signalsRelayVersion, siteEditionLabel } from "@/lib/shared";

const navLinks: Array<{ href: string; label: string }> = [
  { href: "/", label: "Overview" },
  { href: docsHref("concepts"), label: "Concepts" },
  { href: docsHref("install"), label: "Install" },
  { href: docsHref("current-architecture"), label: "Architecture" },
  { href: docsHref("troubleshooting"), label: "Troubleshooting" },
  { href: docsHref(), label: "Docs" },
];

const indexEntries: Array<{ num: string; title: string; description: string; href: string }> = [
  {
    num: "¶ 01",
    title: "Concepts",
    description: "Glossary, export modes, secret contract, and inputs.",
    href: docsHref("concepts"),
  },
  {
    num: "¶ 02",
    title: "Install",
    description: "From a Release, via SAR, IaC, or built from source.",
    href: docsHref("install"),
  },
  {
    num: "¶ 03",
    title: "Architecture",
    description: "Partitioner, Kinesis, tumbling window, export modes.",
    href: docsHref("current-architecture"),
  },
  {
    num: "¶ 04",
    title: "Troubleshooting",
    description: "Verify deploys, diagnose missing spans, failure queues.",
    href: docsHref("troubleshooting"),
  },
];

const pipelineSteps: Array<{ num: string; label: ReactNode; tag: string }> = [
  {
    num: "¶ 01",
    label: (
      <>
        CloudWatch Logs subscription on <em>aws/spans</em>
      </>
    ),
    tag: "source",
  },
  {
    num: "¶ 02",
    label: (
      <>
        Partitioner λ republishes records with <em>traceId</em> keys
      </>
    ),
    tag: "lambda",
  },
  {
    num: "¶ 03",
    label: (
      <>
        Kinesis buffers spans into <em>trace-aligned</em> lanes
      </>
    ),
    tag: "stream",
  },
  {
    num: "¶ 04",
    label: (
      <>
        Relay λ exports OTLP — <em>direct</em> or <em>collector</em> mode
      </>
    ),
    tag: "export",
  },
];

const imprintBackends: Array<{ name: string; icon: ReactNode }> = [
  {
    name: "Honeycomb",
    icon: (
      <svg aria-hidden="true" fill="currentColor" viewBox="0 0 24 24">
        <path d="M12 2l8 4.6v8.8L12 20l-8-4.6V6.6L12 2zm0 2.3L6 7.7v6.6l6 3.4 6-3.4V7.7l-6-3.4z" />
      </svg>
    ),
  },
  {
    name: "Datadog",
    icon: (
      <svg aria-hidden="true" fill="currentColor" viewBox="0 0 24 24">
        <circle cx="12" cy="12" r="9" />
      </svg>
    ),
  },
  {
    name: "Grafana Tempo",
    icon: (
      <svg aria-hidden="true" fill="currentColor" viewBox="0 0 24 24">
        <circle cx="12" cy="12" r="9" />
        <path d="M12 6v6l4 2" fill="none" stroke="#0e0d0a" strokeLinecap="round" strokeWidth="2.2" />
      </svg>
    ),
  },
  {
    name: "New Relic",
    icon: (
      <svg aria-hidden="true" fill="currentColor" viewBox="0 0 24 24">
        <path d="M12 2L3 7v10l9 5 9-5V7l-9-5z" />
      </svg>
    ),
  },
  {
    name: "Any OTLP/HTTP",
    icon: (
      <svg aria-hidden="true" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
        <circle cx="12" cy="12" r="9" />
        <path d="M12 7v10M7 12h10" />
      </svg>
    ),
  },
];

const colophonProject: Array<{ href: string; label: string; external?: boolean }> = [
  { href: githubUrl, label: "GitHub", external: true },
  { href: `${githubUrl}/releases`, label: "Releases", external: true },
  { href: `${githubUrl}/blob/main/LICENSE`, label: "License (MIT)", external: true },
];

export default function HomePage() {
  return (
    <div className="editorial">
      <EditorialMasthead activeHref="/" navLinks={navLinks} />

      <section className="editorial-hero">
        <div className="editorial-hero__bp" aria-hidden="true" />
        <div className="editorial-wrap">
          <div className="editorial-hero__row">
            <div className="editorial-hero__col">
              <div className="editorial-hero__meta">
                <span>edition</span>
                <b>{siteEditionLabel}</b>
                <span className="editorial-hero__pill">in beta</span>
              </div>
              <h1 className="editorial-hero__title">
                Send <em>any</em> AWS Application Signal traces{" "}
                <span className="editorial-hero__amp">&amp;</span> ship it to any OTLP backend.
              </h1>
              <p className="editorial-hero__sub">
                A serverless AWS pipeline that turns <code>aws/spans</code> log records into OTLP
                traces and delivers them to Honeycomb, Datadog, Grafana Tempo, New Relic — or any
                OTLP/HTTP endpoint of your choosing.
              </p>
              <div className="editorial-hero__cta">
                <Link className="editorial-btn-paper" href={docsHref("install")}>
                  Install guide
                  <ArrowRight aria-hidden="true" />
                </Link>
                <Link className="editorial-btn-link" href={docsHref("current-architecture")}>
                  Read the architecture
                  <ArrowRight aria-hidden="true" />
                </Link>
              </div>
            </div>
            <SchematicDiagram />
          </div>
        </div>
      </section>

      <section className="editorial-index">
        <div className="editorial-wrap">
          <div className="editorial-index__row">
            {indexEntries.map((entry) => (
              <Link className="editorial-index__cell" href={entry.href} key={entry.num}>
                <span className="editorial-index__num">{entry.num}</span>
                <h3 className="editorial-index__title">{entry.title}</h3>
                <p className="editorial-index__desc">{entry.description}</p>
              </Link>
            ))}
          </div>
        </div>
      </section>

      <section className="editorial-article">
        <div className="editorial-wrap">
          <div className="editorial-article__row">
            <aside className="editorial-article__kicker">
              <small>§ Runtime shape</small>
              <h3>Trace-aligned buffering, instead of log-batched chaos.</h3>
            </aside>
            <div>
              <p className="editorial-article__lede">
                Signals Relay trades a little latency for control over <em>grouping</em>,{" "}
                <em>batching</em>, and <em>export</em> — the things raw CloudWatch Logs batches
                won&apos;t give you.
              </p>
              <div className="editorial-article__body">
                <p>
                  Application Signals lands as <code>aws/spans</code> log records, in batches
                  CloudWatch decides for you. That&apos;s fine if you only need raw data in
                  CloudWatch; it falls apart the moment you want trace-aware export. The relay
                  reshapes that flow into trace-keyed lanes, then exports each lane on a predictable
                  cadence.
                </p>
                <p>
                  Managed-link decorators reconcile <em>inside</em> the active tumbling window
                  before OTLP export. No cross-window state, no surprise reordering, no batch sizes
                  that depend on whatever CloudWatch felt like sending.
                </p>
              </div>
              <ol className="editorial-pipeline">
                {pipelineSteps.map((step) => (
                  <li key={step.num}>
                    <span className="editorial-pipeline__num">{step.num}</span>
                    <span className="editorial-pipeline__label">{step.label}</span>
                    <span className="editorial-pipeline__tag">{step.tag}</span>
                  </li>
                ))}
              </ol>
            </div>
          </div>
        </div>
      </section>

      <section className="editorial-imprint">
        <div className="editorial-wrap">
          <div className="editorial-imprint__row">
            <span className="editorial-imprint__label">
              <em>Ships</em> to
            </span>
            <div className="editorial-imprint__logos">
              {imprintBackends.map((backend) => (
                <div className="editorial-imprint__logo" key={backend.name}>
                  {backend.icon}
                  {backend.name}
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>

      <footer className="editorial-colophon">
        <div className="editorial-wrap">
          <div className="editorial-colophon__row">
            <div>
              <p className="editorial-colophon__brand">
                <em>Signals</em> Relay
              </p>
              <p className="editorial-colophon__lede">
                Beta — run in dev or staging today, then review the{" "}
                <Link
                  className="editorial-colophon__inline"
                  href={`${docsHref("install")}#production-hardening-checklist`}
                >
                  hardening checklist
                </Link>{" "}
                before production.
              </p>
              <p className="editorial-colophon__version">
                v{signalsRelayVersion} · MIT licensed
              </p>
            </div>
            <div>
              <h4>Docs</h4>
              <ul>
                <li>
                  <Link href={docsHref("concepts")}>Concepts</Link>
                </li>
                <li>
                  <Link href={docsHref("install")}>Install</Link>
                </li>
                <li>
                  <Link href={docsHref("current-architecture")}>Architecture</Link>
                </li>
                <li>
                  <Link href={docsHref("troubleshooting")}>Troubleshooting</Link>
                </li>
              </ul>
            </div>
            <div>
              <h4>Project</h4>
              <ul>
                {colophonProject.map((entry) => (
                  <li key={entry.label}>
                    <a
                      href={entry.href}
                      rel={entry.external ? "noopener noreferrer" : undefined}
                      target={entry.external ? "_blank" : undefined}
                    >
                      {entry.label}
                    </a>
                  </li>
                ))}
              </ul>
            </div>
            <div>
              <h4>Set in</h4>
              <ul>
                <li>Instrument Serif</li>
                <li>Inter</li>
                <li>JetBrains Mono</li>
              </ul>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}
