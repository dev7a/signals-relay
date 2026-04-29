# Signals Relay Docs Site

This directory contains the static Fumadocs site for Signals Relay.

The authored documentation stays in the repository root under `docs/`. The
`site/content/docs` and `site/public/docs-assets` directories are generated and
should not be edited by hand.

## Local Development

```bash
pnpm --dir site install
pnpm --dir site dev
```

## Static Preview

```bash
GITHUB_ACTIONS=true GITHUB_REPOSITORY=dev7a/signals-relay pnpm --dir site build
pnpm --dir site start
```

The static preview serves the exported site under `/signals-relay/`, matching
the GitHub Pages project-site path.
