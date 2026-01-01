#!/usr/bin/env python3
"""
Fetch crates.io metadata + README for our canonical dependency list.

Writes raw artifacts to: third_party/crate_docs/<crate>/{meta.json,readme.md}
(third_party is git-ignored; distilled summaries should live under docs/).
"""

from __future__ import annotations

import json
import os
import pathlib
import sys
import urllib.error
import urllib.request


CRATES = [
    "dlx_rs",
    "bitvec",
    "mimalloc",
    "bumpalo",
    "smallvec",
    "wide",
    "soa_derive",
    "likely_stable",
    "static_assertions",
    "rayon",
    "parking_lot",
    "ringbuf",
    "dashmap",
    "nohash-hasher",
    "fxhash",
    "rand_pcg",
    "fixed",
    "num-integer",
    "rkyv",
    "crux_core",
    "uniffi",
    "rust-embed",
    "bytemuck",
    "anyhow",
    "thiserror",
    "tracing",
    "tracing-subscriber",
    "tracing-tracy",
    "criterion",
    "ratatui",
    "varisat",
    "z3",
    "kani",
    "proptest",
    "bolero",
    "nom",
]


def http_get(url: str) -> bytes:
    req = urllib.request.Request(
        url,
        headers={
            "User-Agent": "rustykeen-crate-doc-audit/1.0",
            "Accept": "*/*",
        },
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        return resp.read()


def main() -> int:
    root = pathlib.Path(__file__).resolve().parents[1]
    out_root = root / "third_party" / "crate_docs"
    out_root.mkdir(parents=True, exist_ok=True)

    results: dict[str, dict[str, str]] = {}

    for crate in CRATES:
        crate_dir = out_root / crate
        crate_dir.mkdir(parents=True, exist_ok=True)

        meta_url = f"https://crates.io/api/v1/crates/{crate}"
        readme_url = ""
        versions_url = f"https://crates.io/api/v1/crates/{crate}/versions"

        entry: dict[str, str] = {"meta_url": meta_url, "readme_url": readme_url}
        try:
            meta = http_get(meta_url)
            (crate_dir / "meta.json").write_bytes(meta)
            entry["meta"] = "ok"

            meta_obj = json.loads(meta.decode("utf-8"))
            crate_id = meta_obj.get("crate", {}).get("id") or crate
            max_version = meta_obj.get("crate", {}).get("max_version")
            if max_version:
                readme_url = (
                    f"https://crates.io/api/v1/crates/{crate_id}/{max_version}/readme"
                )
                versions_url = f"https://crates.io/api/v1/crates/{crate_id}/versions"
        except urllib.error.HTTPError as e:
            entry["meta"] = f"http_error:{e.code}"
        except Exception as e:  # noqa: BLE001
            entry["meta"] = f"error:{type(e).__name__}"

        try:
            vers = http_get(versions_url)
            (crate_dir / "versions.json").write_bytes(vers)
            entry["versions"] = "ok"
        except urllib.error.HTTPError as e:
            entry["versions"] = f"http_error:{e.code}"
        except Exception as e:  # noqa: BLE001
            entry["versions"] = f"error:{type(e).__name__}"

        try:
            if not readme_url:
                entry["readme"] = "skipped:no_meta"
            else:
                entry["readme_url"] = readme_url
                readme = http_get(readme_url)
                (crate_dir / "readme.md").write_bytes(readme)
                entry["readme"] = "ok"
        except urllib.error.HTTPError as e:
            entry["readme"] = f"http_error:{e.code}"
        except Exception as e:  # noqa: BLE001
            entry["readme"] = f"error:{type(e).__name__}"

        results[crate] = entry

    (out_root / "index.json").write_text(json.dumps(results, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {len(CRATES)} crate doc bundles under {out_root}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
