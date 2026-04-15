#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SRC_CSV="$REPO_ROOT/docs/literature/sources.csv"
OUT_CSV="$REPO_ROOT/docs/literature/provenance_manifest.csv"

export REPO_ROOT
python3 - <<'PY'
import csv, hashlib, subprocess
from pathlib import Path

import os
repo = Path(os.environ['REPO_ROOT'])
src_csv = repo / 'docs/literature/sources.csv'
out_csv = repo / 'docs/literature/provenance_manifest.csv'
rows = []

with src_csv.open(newline='', encoding='utf-8') as f:
    reader = csv.DictReader(f)
    for row in reader:
        rel = row['local_file']
        path = repo / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run([
            'curl', '-L', '--fail', '--silent', '--show-error', '--retry', '2',
            '--output', str(path), row['source_url']
        ], check=True)
        data = path.read_bytes()
        if not data.startswith(b'%PDF'):
            raise RuntimeError(f'Not a PDF: {rel}')
        rows.append({
            'title': row['title'],
            'authors': row['authors'],
            'year': row['year'],
            'source_url': row['source_url'],
            'local_file': rel,
            'sha256': hashlib.sha256(data).hexdigest(),
        })

with out_csv.open('w', newline='', encoding='utf-8') as f:
    writer = csv.DictWriter(f, fieldnames=['title','authors','year','source_url','local_file','sha256'])
    writer.writeheader()
    writer.writerows(rows)

print(f"Refreshed {len(rows)} PDFs")
print(f"Updated {out_csv}")
PY
