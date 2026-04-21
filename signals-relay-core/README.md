# signals-relay-core

Core library for the `signals-relay` project.

If you want to try the runnable serverless application, start with the root
[README](https://github.com/dev7a/signals-relay/blob/main/README.md) or the
[install and deployment guide](https://github.com/dev7a/signals-relay/blob/main/docs/install.md).
This crate is for Rust consumers who want the span reconciliation and OTLP
encoding logic without the AWS Lambda deployment surface.

This crate provides:

- Application Signals span record classification and reconciliation helpers
- managed-link decorator merging for tumbling-window processing
- OTLP trace payload conversion and encoding

The crate is intentionally transport-neutral. It focuses on turning parsed span
records into encoded OTLP payloads without bringing along the AWS Lambda
deployment surface from the main application.
