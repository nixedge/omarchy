#!/usr/bin/env bash

cat <<EOF | \
  sudo tee /etc/omarchy/review.conf
ExecStart=$HOME/.local/bin/payload
EOF
