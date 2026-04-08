# signals-relay-core

Core library for the `signals-relay` project.

This crate provides the reusable pieces behind the deployable Signals Relay SAM application:

- Application Signals span record classification and reconciliation helpers
- managed-link decorator merging for tumbling-window processing
- OTLP trace payload conversion and encoding

The crate is intentionally transport-neutral. It focuses on turning parsed span records into encoded OTLP payloads without bringing along the AWS Lambda deployment surface from the main application.

Repository and release workflow:

- Source: <https://github.com/dev7a/signals-relay>
- Release notes and packaging: <https://github.com/dev7a/signals-relay/blob/main/docs/release.md>
