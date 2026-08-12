#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PROJECT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
CARGO_BIN=${CARGO_BIN:-cargo}
INSTALL_BIN=${INSTALL_BIN:-"$HOME/.local/bin"}
SYSTEMD_USER_DIR=${SYSTEMD_USER_DIR:-"$HOME/.config/systemd/user"}

cd "$PROJECT_DIR"
"$CARGO_BIN" build --release -p homie-node
mkdir -p \
    "$INSTALL_BIN" \
    "$SYSTEMD_USER_DIR" \
    "$HOME/.config/homie" \
    "$HOME/.local/share/homie/node"
install -m 0755 "target/release/homie-node" "$INSTALL_BIN/homie-node"
install -m 0644 "infra/homie-node.service" "$SYSTEMD_USER_DIR/homie-node.service"
chmod 700 "$HOME/.config/homie"
chmod 700 "$HOME/.local/share/homie/node"

if command -v systemctl >/dev/null 2>&1; then
    systemctl --user daemon-reload
fi

printf '%s\n' "Installed $INSTALL_BIN/homie-node"
printf '%s\n' "Next: set HOMIE_NODE_LISTEN in ~/.config/homie/node.env, then run:"
printf '%s\n' "  systemctl --user enable --now homie-node"
printf '%s\n' "  homie-node init"
