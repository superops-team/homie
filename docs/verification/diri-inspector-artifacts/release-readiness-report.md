# Diri Inspector Artifacts Release Readiness

```yaml
change_id: diri-inspector-artifacts
beads: homie-3q0
status: ready_for_next_loopx_slice
```

## Delivered

- Client artifact scan API.
- Runtime-output based inspector artifact summary.
- App regression preventing static `Ports none` artifact rows.

## Parity Impact

| Row | Decision | Reason |
|-----|----------|--------|
| UI-004 | partial | Inspector Artifacts now reads real scan data; diff/Changes and full artifact E2E remain pending. |
| ART-001 | partial | Link/PR/preview scanner is wired to app; browser pool/preview E2E remains pending. |
| ART-002 | partial | Localhost ports are detected and shown as counts; forwarding/listing E2E remains pending. |

