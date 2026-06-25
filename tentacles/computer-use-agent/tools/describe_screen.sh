#!/usr/bin/env bash
set -euo pipefail

echo "== desktop =="
uname -a

echo "== visible process hints =="
ps -axo comm | sort | uniq | sed -n '1,80p'

