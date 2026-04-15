# `kenken-io`

Versioned structured I/O for the Keen engine.

Current focus:
- `io-rkyv`: snapshot v1 encoding/decoding using `rkyv` for fast, zero-copy-friendly persistence.
- `format-sgt-desc`: text encode/decode via upstream-compatible SGT `desc` format
  (`kenken_io::sgt_desc_snapshot`) for portable/auditable snapshots.

The snapshot format is intentionally *not* the upstream “desc” string; it is a versioned, engine-owned representation.
