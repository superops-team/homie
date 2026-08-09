# OpenSpec Plan: Diri Ports List CLI Runtime

```yaml
change_id: diri-ports-list-cli-runtime
beads: homie-979
prd: prd-spec/features/diri-ports-list-cli-runtime/2026-08-08-diri-ports-list-cli-runtime-design.md
```

## Scope

Implement `homie ports` as a runtime-backed list command over existing session outputs.

## Boundaries

- `homie-client`: aggregate session output ports.
- `homie-cli`: expose `ports` command.
- No TCP forwarding implementation.

