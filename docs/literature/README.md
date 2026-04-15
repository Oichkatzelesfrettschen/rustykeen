# Literature Corpus (PDF + Provenance)

This directory is the in-repo source-of-truth corpus for architecture/design/technical-debt analysis in `rustykeen`.

## Contents

- `pdfs/` — downloaded source PDFs
- `sources.csv` — deterministic source list (title/authors/year/url/path)
- `provenance_manifest.csv` — downloaded files with SHA256 checksums
- `first_principles_derivation.md` — implementation-grounded mathematical derivation for solver/generator architecture
- `refresh_corpus.sh` — re-download all PDFs and regenerate manifest
- `verify_corpus.sh` — verify file presence + SHA256 integrity

## Selection criteria

The corpus prioritizes:

1. **High-authority foundational papers** (DLX/Algorithm X, SAT/CDCL, arc consistency, rigorous benchmarking).
2. **Direct relevance to this codebase** (KenKen/Latin-square structure, exact-cover techniques, CSP propagation, SAT solving, puzzle generation, and benchmarking methodology).
3. **Temporal coverage** from legacy foundations through modern work **up to Apr 2026**.
4. **Open, reproducible retrieval** via stable URLs and deterministic local paths.

## Coverage index (21 PDFs)

- **Exact cover / DLX / Latin-square foundations**
  - Knuth 2000 (`Dancing links`), Kapanowski 2010, Bartlett 2012, Fontana 2013, Kawase et al. 2025, Kemp 2026.
- **Constraint propagation / CSP**
  - Cooper & Schiex 2001, Chen et al. 2011, Berkholz & Verbitsky 2013.
- **SAT/CDCL architecture and solver engineering**
  - Moskewicz et al. 2001 (Chaff), Een & Sorensson 2003 (MiniSAT), Li et al. 2020, Zhang et al. 2024, Su et al. 2025, Cai et al. 2026, Singh et al. 2026.
- **Puzzle generation methodology**
  - Nishikawa & Toda 2020 (exact clue-generation method).
- **Performance methodology / benchmarking rigor**
  - Georges et al. 2007, Kalibera & Jones 2013, Li et al. 2017 (DoKnowMe), van der Kouwe et al. 2018.

## Reproducibility

From repo root:

```bash
./docs/literature/refresh_corpus.sh
./docs/literature/verify_corpus.sh
```

This re-fetches all PDFs defined in `sources.csv`, regenerates `provenance_manifest.csv`, and verifies local integrity against SHA256.
