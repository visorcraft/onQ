# ADR 0001: Plugin embedder selection hook

## Status
Accepted (scaffold)

## Context
Plugins may provide alternate embedding models. Core currently uses MiniLM via ONNX.

## Decision
- Built-in MiniLM remains the default embedder (`MODEL_ID`).
- Host loads plugins via existing install path; plugins that advertise embedding
  capability can later replace `AppState.embedder` through a host registration
  path (HostApi already exposes `embedding_embed` for reverse calls).
- No automatic download of arbitrary model weights without explicit user action.

## Consequences
- Settings can show installed plugins; selecting an alternate embedder is
  optional and gated on capability metadata.
- Full multi-model management remains iterative.
