#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MANIFEST="$REPO_ROOT/docs/literature/provenance_manifest.csv"

export REPO_ROOT
python3 - <<'PY'
import csv, hashlib, sys
from pathlib import Path

import os
repo = Path(os.environ['REPO_ROOT'])
manifest = repo / 'docs/literature/provenance_manifest.csv'

ok = True
with manifest.open(newline='', encoding='utf-8') as f:
    reader = csv.DictReader(f)
    for row in reader:
        rel = row['local_file']
        expected = row['sha256']
        p = repo / rel
        if not p.exists():
            print(f'MISSING: {rel}')
            ok = False
            continue
        got = hashlib.sha256(p.read_bytes()).hexdigest()
        if got != expected:
            print(f'SHA MISMATCH: {rel}\n  expected {expected}\n  got      {got}')
            ok = False

if ok:
    print('All corpus files exist and match manifest SHA256 values.')
    sys.exit(0)
sys.exit(1)
PY
