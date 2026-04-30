import Link from "next/link";
import { ArrowRight, Boxes, LifeBuoy, RadioTower } from "lucide-react";
import { HeroParallax } from "@/components/hero-parallax";
import { HomeActions } from "@/components/home-actions";
import { RelayFlowDiagram } from "@/components/relay-flow-diagram";
import { SectionKicker } from "@/components/section-kicker";
import { docsHref, githubUrl, signalsRelayVersion } from "@/lib/shared";

const pipeline = [
  "CloudWatch Logs subscription on aws/spans",
  "Partitioner Lambda republishes records with traceId keys",
  "Kinesis buffers spans into trace-aligned lanes",
  "Relay Lambda exports OTLP over direct or collector mode",
];

const fit = [
  {
    title: "Evaluate Application Signals export",
    body: "Use the SAR app or quick-launch CloudFormation wrappers to test a standalone AWS path.",
  },
  {
    title: "Follow a broader operator path",
    body: "Concepts, install, verification, and troubleshooting docs cover the full evaluation loop.",
  },
  {
    title: "Understand the tradeoffs",
    body: "The architecture guide explains the partitioner, Kinesis stream, tumbling window, and export modes.",
  },
];

export default function HomePage() {
  return (
    <main className="min-h-screen">
      <div className="home-topbar">
        <Link aria-label="Signals Relay home" className="home-topbar__brand" href="/">
          Signals Relay
        </Link>
        <nav aria-label="Primary" className="home-topbar__links">
          <Link href={docsHref("concepts")}>Concepts</Link>
          <Link href={docsHref("install")}>Install</Link>
          <Link href={docsHref("current-architecture")}>Architecture</Link>
          <Link href={docsHref("troubleshooting")}>Troubleshooting</Link>
          <Link href={docsHref()}>Docs</Link>
        </nav>
        <HomeActions githubUrl={githubUrl} />
      </div>

      <section className="signals-hero relative isolate overflow-hidden border-b border-neutral-800 bg-neutral-950 text-white">
        <HeroParallax />
        <div className="relative mx-auto flex min-h-[inherit] w-full max-w-7xl items-center px-6 pb-20 pt-32 md:px-10 md:pb-24 md:pt-36 lg:px-12">
          <div className="max-w-3xl space-y-6">
            <p className="inline-flex rounded-md border border-white/15 bg-white/[0.08] px-3 py-1.5 text-sm font-semibold text-neutral-300 shadow-sm shadow-black/20 backdrop-blur">
              AWS serverless OTLP relay - v{signalsRelayVersion}
            </p>
            <h1 className="max-w-3xl text-5xl font-semibold leading-[0.98] tracking-[-0.05em] text-white md:text-7xl">
              Signals Relay
            </h1>
            <p className="max-w-2xl text-lg leading-8 text-neutral-300 lg:text-xl">
              Convert CloudWatch Application Signals aws/spans log records into OTLP trace
              payloads and export them to an OTLP/HTTP backend with an inspectable serverless
              pipeline.
            </p>
            <p className="max-w-2xl rounded-md border border-amber-300/20 bg-amber-300/[0.08] px-3 py-2 text-sm leading-6 text-amber-100">
              Use this as an evaluation path and review the hardening checklist before
              production.
            </p>
            <div className="flex flex-col gap-3 sm:flex-row">
              <Link
                className="inline-flex items-center justify-center gap-2 rounded-md bg-white px-5 py-2.5 text-sm font-semibold text-neutral-950 shadow-lg shadow-black/30 transition hover:bg-neutral-100"
                href={docsHref("install")}
              >
                Install guide
                <ArrowRight aria-hidden="true" className="h-4 w-4" />
              </Link>
              <Link
                className="inline-flex items-center justify-center gap-2 rounded-md border border-white/15 bg-white/[0.08] px-5 py-2.5 text-sm font-semibold text-white backdrop-blur transition hover:bg-white/[0.14]"
                href={docsHref("troubleshooting")}
              >
                Troubleshooting
              </Link>
            </div>
            <div className="grid gap-3 pt-2 sm:grid-cols-3">
              {fit.map(({ title, body }) => (
                <article className="rounded-xl border border-white/10 bg-black/[0.18] px-4 py-4 backdrop-blur-sm" key={title}>
                  <h2 className="text-sm font-semibold text-white">{title}</h2>
                  <p className="mt-1 text-sm leading-6 text-slate-200/75">{body}</p>
                </article>
              ))}
            </div>
          </div>
        </div>
      </section>

      <section className="border-b border-fd-border bg-fd-background">
        <div className="mx-auto grid w-full max-w-7xl gap-10 px-6 py-16 md:px-10 md:py-20 lg:grid-cols-[0.86fr_1.14fr] lg:items-center lg:px-12">
          <div>
            <SectionKicker>Runtime shape</SectionKicker>
            <h2 className="mt-3 text-3xl font-semibold tracking-[-0.04em] text-fd-foreground md:text-4xl">
              Built around trace-aligned buffering.
            </h2>
            <p className="mt-4 text-lg leading-8 text-fd-muted-foreground">
              Signals Relay trades a little latency for more control over grouping, batching, and
              export behavior than raw CloudWatch Logs batches usually provide.
            </p>
            <ol className="mt-8 grid gap-3">
              {pipeline.map((item, index) => (
                <li
                  className="grid grid-cols-[2.5rem_1fr] items-center gap-3 rounded-xl border border-fd-border bg-fd-card p-4"
                  key={item}
                >
                  <span className="flex h-9 w-9 items-center justify-center rounded-md border border-fd-border bg-fd-muted font-mono text-sm">
                    {index + 1}
                  </span>
                  <span className="text-sm leading-6 text-fd-foreground">{item}</span>
                </li>
              ))}
            </ol>
          </div>
          <RelayFlowDiagram />
        </div>
      </section>

      <section className="border-b border-fd-border bg-fd-muted/30">
        <div className="mx-auto w-full max-w-7xl px-6 py-16 md:px-10 md:py-20 lg:px-12">
          <div className="mb-8 max-w-2xl">
            <SectionKicker>Where to next</SectionKicker>
            <h2 className="mt-3 text-3xl font-semibold tracking-[-0.04em] text-fd-foreground md:text-4xl">
              Move from evaluation to operation.
            </h2>
          </div>
        <div className="grid gap-4 md:grid-cols-3">
          <Link
            className="group rounded-2xl border border-fd-border bg-fd-card p-6 transition hover:-translate-y-0.5 hover:border-[color:var(--signals-cyan-soft)] hover:shadow-sm"
            href={docsHref()}
          >
            <Boxes aria-hidden="true" className="h-5 w-5 text-[color:var(--signals-blue)]" />
            <h3 className="mt-3 font-semibold">Open the docs</h3>
            <p className="mt-2 text-sm leading-6 text-fd-muted-foreground">
              Choose the guide that matches your task.
            </p>
          </Link>
          <Link
            className="group rounded-2xl border border-fd-border bg-fd-card p-6 transition hover:-translate-y-0.5 hover:border-[color:var(--signals-cyan-soft)] hover:shadow-sm"
            href={docsHref("troubleshooting")}
          >
            <LifeBuoy aria-hidden="true" className="h-5 w-5 text-[color:var(--signals-green)]" />
            <h3 className="mt-3 font-semibold">Troubleshoot an install</h3>
            <p className="mt-2 text-sm leading-6 text-fd-muted-foreground">
              Verify stack outputs, span flow, export behavior, and failure queues.
            </p>
          </Link>
          <Link
            className="group rounded-2xl border border-fd-border bg-fd-card p-6 transition hover:-translate-y-0.5 hover:border-[color:var(--signals-cyan-soft)] hover:shadow-sm"
            href={docsHref("current-architecture")}
          >
            <RadioTower aria-hidden="true" className="h-5 w-5 text-[color:var(--signals-amber)]" />
            <h3 className="mt-3 font-semibold">Review architecture</h3>
            <p className="mt-2 text-sm leading-6 text-fd-muted-foreground">
              Understand the partitioner, stream, relay window, and export modes.
            </p>
          </Link>
        </div>
        </div>
      </section>

      <section className="border-t border-fd-border bg-fd-muted/30">
        <div className="mx-auto flex w-full max-w-7xl flex-col gap-4 px-6 py-10 md:flex-row md:items-center md:justify-between md:px-10 lg:px-12">
          <div>
            <h2 className="text-xl font-semibold tracking-normal">Ready to evaluate the relay?</h2>
            <p className="mt-2 text-sm text-fd-muted-foreground">
              Start with the install guide, then verify the architecture assumptions for your
              account.
            </p>
          </div>
          <Link
            className="inline-flex items-center justify-center gap-2 rounded-md border border-fd-border bg-fd-background px-4 py-2.5 text-sm font-medium transition hover:bg-fd-muted"
            href={docsHref("install")}
          >
            <RadioTower aria-hidden="true" className="h-4 w-4" />
            Deploy from a release
          </Link>
        </div>
      </section>
    </main>
  );
}
