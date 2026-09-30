tmp=$(mktemp)

cat >"$tmp" <<EOF
#!/usr/bin/env bash
exec "$HOME/.local/share/omarchy/bin/omarchy-agent" "$@"
EOF

sudo install -m 0755 "$tmp" /usr/local/bin/omarchy-agent-shim
