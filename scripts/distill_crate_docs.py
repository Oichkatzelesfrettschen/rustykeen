#!/usr/bin/env python3
"""
Distill third_party/crate_docs/* into repo documentation under docs/deps/.

We intentionally do not vendor full third-party READMEs into the repo;
we keep raw fetches under git-ignored third_party/ and commit our own
summaries + decisions.
"""

from __future__ import annotations

import json
import pathlib
import re


def load_text(path: pathlib.Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def extract_features(readme: str) -> list[str]:
    # Heuristic: find a "Features" section header and collect bullet lines until next header.
    m = re.search(r"^#{1,4}\s+Features\s*$", readme, flags=re.IGNORECASE | re.MULTILINE)
    if not m:
        return []
    tail = readme[m.end() :]
    # stop at next header
    stop = re.search(r"^#{1,4}\s+\S", tail, flags=re.MULTILINE)
    if stop:
        tail = tail[: stop.start()]
    feats = []
    for line in tail.splitlines():
        line = line.strip()
        if line.startswith(("-", "*")):
            feats.append(line.lstrip("-* ").strip())
        if len(feats) >= 12:
            break
    return feats


def extract_cargo_features(versions_json: str, max_version: str) -> list[str]:
    try:
        obj = json.loads(versions_json)
        for v in obj.get("versions", []):
            if v.get("num") == max_version:
                feats = v.get("features") or {}
                return sorted(feats.keys())
    except Exception:  # noqa: BLE001
        return []
    return []


def main() -> int:
    root = pathlib.Path(__file__).resolve().parents[1]
    src_root = root / "third_party" / "crate_docs"
    out_root = root / "docs" / "deps"
    out_root.mkdir(parents=True, exist_ok=True)

    index_path = src_root / "index.json"
    index = json.loads(load_text(index_path))

    mapping = {
        # Blue smoke core
        "dlx-rs": ("Latin exact-cover solver (DLX / Algorithm X)", "solver-dlx", "planned"),
        "bitvec": ("Bit-level candidate domains / masks", "core-bitvec", "planned"),
        "mimalloc": ("High-performance global allocator (non-iOS)", "alloc-mimalloc", "planned"),
        "bumpalo": ("Arena allocator for search nodes", "alloc-bumpalo", "planned"),
        "smallvec": ("Small, stack-backed vectors (cage cell lists)", "core-smallvec", "now"),
        "wide": ("SIMD vector types for hotpath checks", "simd-wide", "planned"),
        "soa_derive": ("Struct-of-Arrays layout for batch throughput", "layout-soa", "planned"),
        "likely_stable": ("Branch prediction hints", "perf-likely", "planned"),
        "static_assertions": ("Compile-time size/alignment contracts", "perf-assertions", "planned"),
        # Hyper-scale
        "rayon": ("Parallel generation / batch solving", "parallel-rayon", "planned"),
        "parking_lot": ("Fast locks for caches", "sync-parking_lot", "planned"),
        "ringbuf": ("Lock-free SPSC telemetry queue", "telemetry-ringbuf", "planned"),
        "dashmap": ("Concurrent caches/maps", "cache-dashmap", "planned"),
        "nohash-hasher": ("Fast hash for integer keys", "hash-fast", "planned"),
        "fxhash": ("Fast hash alternative", "hash-fast", "planned"),
        "rand_pcg": ("Deterministic RNG streams (candidate)", "rng-pcg", "planned"),
        "fixed": ("Deterministic fixed-point math", "math-fixed", "planned"),
        "num-integer": ("GCD/LCM and integer utilities", "math-num-integer", "planned"),
        # Zero-overhead architecture
        "rkyv": ("Zero-copy snapshots / persistence", "io-rkyv", "planned"),
        "crux_core": ("Headless UI architecture core", "ui-crux", "planned"),
        "uniffi": ("Kotlin/Swift bindings generator", "ffi-uniffi", "planned"),
        "rust-embed": ("Embed assets into binaries", "assets-embed", "planned"),
        "bytemuck": ("Zero-cost byte casting where safe", "bytes-bytemuck", "planned"),
        "anyhow": ("Ergonomic edge error handling", "errors-anyhow", "planned"),
        "thiserror": ("Typed library errors", "errors-thiserror", "now"),
        # Tooling / verification
        "tracing": ("Structured spans/events", "telemetry-tracing", "now"),
        "tracing-subscriber": ("Tracing output routing", "telemetry-subscriber", "planned"),
        "tracing-tracy": ("Tracy profiler integration", "telemetry-tracy", "planned"),
        "criterion": ("Statistical benchmarking", "bench-criterion", "planned"),
        "ratatui": ("Developer TUI dashboard", "dev-tui", "planned"),
        "varisat": ("SAT solver for uniqueness proofs", "sat-varisat", "planned"),
        "z3": ("SMT solver for formal checks", "smt-z3", "planned"),
        "kani": ("Model checking harnesses", "verify-kani", "planned"),
        "proptest": ("Property-based tests", "fuzz", "planned"),
        "bolero": ("Fuzz/property tests", "fuzz", "planned"),
        "nom": ("Legacy corpus parsing", "io-nom", "planned"),
    }

    crate_rows = []
    for crate, status in sorted(index.items()):
        meta_path = src_root / crate / "meta.json"
        readme_path = src_root / crate / "readme.md"
        if not meta_path.exists():
            continue
        meta = json.loads(load_text(meta_path))
        c = meta.get("crate", {})
        crate_id = c.get("id", crate)
        version = c.get("max_version", "")
        repo = c.get("repository") or ""
        docs = c.get("documentation") or ""
        desc = (c.get("description") or "").strip()

        readme = load_text(readme_path) if readme_path.exists() else ""
        feats = extract_features(readme)
        versions_path = src_root / crate / "versions.json"
        cargo_feats = (
            extract_cargo_features(load_text(versions_path), version) if versions_path.exists() else []
        )

        role, gate, st = mapping.get(crate_id, ("TBD", "TBD", "planned"))

        out = []
        out.append(f"# `{crate_id}` (audit)\n")
        if desc:
            out.append(desc + "\n")
        out.append("## Upstream\n")
        out.append(f"- crates.io: `https://crates.io/crates/{crate_id}`\n")
        if version:
            out.append(f"- latest observed: `{version}`\n")
        if repo:
            out.append(f"- repository: `{repo}`\n")
        if docs:
            out.append(f"- documentation: `{docs}`\n")
        out.append("\n## Why we care (engine mapping)\n")
        out.append(f"- Intended role: {role}\n")
        out.append(f"- Planned gate: `{gate}`\n")
        out.append(f"- Adoption status: `{st}`\n")
        out.append("\n## Notable features (from upstream docs, heuristic)\n")
        if feats:
            for f in feats:
                out.append(f"- {f}\n")
        else:
            out.append("- (no Features section detected in README)\n")

        out.append("\n## Cargo features (from crates.io metadata)\n")
        if cargo_feats:
            for f in cargo_feats[:40]:
                out.append(f"- `{f}`\n")
        else:
            out.append("- (no feature metadata available)\n")

        (out_root / f"{crate_id}.md").write_text("".join(out), encoding="utf-8")
        crate_rows.append((crate_id, desc))

    # Write an index for navigation.
    idx = []
    idx.append("# Dependency docs index\n\n")
    idx.append("Generated from `third_party/crate_docs/`.\n\n")
    for crate_id, desc in crate_rows:
        line = f"- `docs/deps/{crate_id}.md`"
        if desc:
            line += f" — {desc}"
        idx.append(line + "\n")
    (out_root / "README.md").write_text("".join(idx), encoding="utf-8")

    print(f"Wrote {len(crate_rows)} summaries under {out_root}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
