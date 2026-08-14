# GPUI lifecycle ownership 功能验证 Case

## FC-01: PRD 边界清晰

```bash
rg -n "child entity subscriptions|非目标|_subscriptions" prd-spec/refactors/gpui-lifecycle-task-ownership/2026-08-14-gpui-lifecycle-task-ownership-design.md
```

预期：命令退出码 0。

## FC-02: RootView child subscriptions 不再 detach

```bash
python3 - <<'PY'
from pathlib import Path
text = Path('homie/crates/homie-app/src/root.rs').read_text()
targets = ['cx.subscribe(terminal', 'cx.subscribe_in(&sidebar', 'cx.subscribe_in(\n            &launcher', 'cx.subscribe(navigation', 'cx.subscribe(inspector']
for target in targets:
    idx = text.find(target)
    if idx == -1:
        raise SystemExit(f'missing subscription target: {target!r}')
    window = text[idx:idx+1200]
    if '.detach()' in window:
        raise SystemExit(f'subscription target still detaches: {target!r}')
print('root child subscriptions are held')
PY
```

预期：输出 `root child subscriptions are held`。

## FC-03: RootView 和 UtilitySurfaces targeted tests 通过

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-app root -- --nocapture
cargo test --manifest-path homie/Cargo.toml -p homie-app surface_shell::tests -- --nocapture
```

预期：命令退出码 0。

## FC-04: 静态门禁通过

```bash
(cd homie && cargo fmt --check)
git diff --check
git diff --name-only -- homie/crates/homie-ui homie/crates/homie-engine homie/crates/homie-client
```

预期：前两个命令退出码 0，最后一个命令无输出。
