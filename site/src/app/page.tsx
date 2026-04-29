import Link from "next/link";
import { ArrowRight, Boxes, GitBranch, RadioTower } from "lucide-react";
import { HeroParallax } from "@/components/hero-parallax";
import { docsHref, signalsRelayVersion } from "@/lib/shared";

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
    title: "Keep deployment inputs explicit",
    body: "The docs cover source log group prerequisites, shared collector secrets, VPC egress, and upgrade flow.",
  },
  {
    title: "Understand the tradeoffs",
    body: "The architecture guide explains the partitioner, Kinesis stream, tumbling window, and export modes.",
  },
];

export default function HomePage() {
  return (
    <main className="min-h-screen">
      <section className="signals-hero relative isolate overflow-hidden border-b border-neutral-800 bg-neutral-950 text-white">
        <HeroParallax />
        <div className="relative mx-auto flex min-h-[inherit] w-full max-w-6xl flex-col justify-center px-6 py-20 md:px-10">
          <div className="max-w-3xl">
            <p className="mb-5 inline-flex rounded-md border border-white/15 bg-white/[0.08] px-3 py-1 text-sm text-neutral-300 shadow-sm shadow-black/20 backdrop-blur">
              Experimental AWS serverless OTLP relay - v{signalsRelayVersion}
            </p>
            <h1 className="text-4xl font-semibold leading-tight tracking-normal text-white md:text-6xl">
              Signals Relay
            </h1>
            <p className="mt-6 max-w-2xl text-lg leading-8 text-neutral-300 md:text-xl">
              Convert CloudWatch Application Signals aws/spans log records into OTLP trace
              payloads and export them to an OTLP/HTTP backend with an inspectable serverless
              pipeline.
            </p>
            <div className="mt-8 flex flex-col gap-3 sm:flex-row">
              <Link
                className="inline-flex items-center justify-center gap-2 rounded-md bg-sky-400 px-4 py-2.5 text-sm font-medium text-neutral-950 shadow-lg shadow-black/30 transition hover:bg-sky-300"
                href={docsHref("install")}
              >
                Install guide
                <ArrowRight aria-hidden="true" className="h-4 w-4" />
              </Link>
              <Link
                className="inline-flex items-center justify-center gap-2 rounded-md border border-white/15 bg-white/[0.08] px-4 py-2.5 text-sm font-medium text-white backdrop-blur transition hover:bg-white/[0.14]"
                href={docsHref("current-architecture")}
              >
                Architecture
              </Link>
            </div>
          </div>
        </div>
      </section>

      <section className="border-b border-fd-border bg-fd-muted/30">
        <div className="mx-auto grid w-full max-w-6xl gap-8 px-6 py-14 md:grid-cols-[0.9fr_1.1fr] md:px-10">
          <div>
            <p className="text-sm font-medium uppercase tracking-normal text-fd-muted-foreground">
              Runtime shape
            </p>
            <h2 className="mt-3 text-3xl font-semibold tracking-normal text-fd-foreground">
              Built around trace-aligned buffering.
            </h2>
            <p className="mt-4 leading-7 text-fd-muted-foreground">
              Signals Relay trades a little latency for more control over grouping, batching, and
              export behavior than raw CloudWatch Logs batches usually provide.
            </p>
          </div>
          <ol className="grid gap-3">
            {pipeline.map((item, index) => (
              <li
                className="grid grid-cols-[2.5rem_1fr] items-center gap-3 rounded-md border border-fd-border bg-fd-background p-4"
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
      </section>

      <section className="mx-auto w-full max-w-6xl px-6 py-14 md:px-10">
        <div className="grid gap-4 md:grid-cols-3">
          {fit.map(({ title, body }) => (
            <article className="rounded-md border border-fd-border bg-fd-card p-5" key={title}>
              <h3 className="text-base font-semibold text-fd-foreground">{title}</h3>
              <p className="mt-3 text-sm leading-6 text-fd-muted-foreground">{body}</p>
            </article>
          ))}
        </div>
        <div className="mt-10 grid gap-4 md:grid-cols-3">
          <Link
            className="group rounded-md border border-fd-border p-5 transition hover:bg-fd-muted"
            href={docsHref()}
          >
            <Boxes aria-hidden="true" className="h-5 w-5 text-[color:var(--signals-blue)]" />
            <h3 className="mt-3 font-semibold">Open the docs</h3>
            <p className="mt-2 text-sm leading-6 text-fd-muted-foreground">
              Start from the generated documentation map.
            </p>
          </Link>
          <Link
            className="group rounded-md border border-fd-border p-5 transition hover:bg-fd-muted"
            href={docsHref("release")}
          >
            <GitBranch aria-hidden="true" className="h-5 w-5 text-[color:var(--signals-green)]" />
            <h3 className="mt-3 font-semibold">Maintain releases</h3>
            <p className="mt-2 text-sm leading-6 text-fd-muted-foreground">
              Review the SAR, GitHub Release, and CloudFormation artifact flow.
            </p>
          </Link>
          <Link
            className="group rounded-md border border-fd-border p-5 transition hover:bg-fd-muted"
            href={docsHref("current-architecture")}
          >
            <RadioTower aria-hidden="true" className="h-5 w-5 text-[color:var(--signals-amber)]" />
            <h3 className="mt-3 font-semibold">Review architecture</h3>
            <p className="mt-2 text-sm leading-6 text-fd-muted-foreground">
              Understand the partitioner, stream, relay window, and export modes.
            </p>
          </Link>
        </div>
      </section>

      <section className="border-t border-fd-border bg-fd-muted/30">
        <div className="mx-auto flex w-full max-w-6xl flex-col gap-4 px-6 py-10 md:flex-row md:items-center md:justify-between md:px-10">
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
