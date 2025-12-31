# Security & Reliability

## Memory Safety
Core avoids unsafe; FFI audited; Android HWASan/MTE; Scudo allocator default; UniFFI reduces binding bugs vs manual JNI.

## Determinism
ChaCha20 RNG; record config/seed; CI reproducibility checks; fixed heuristic ordering.

## Visibility
Hidden symbols where applicable; UniFFI exports stable interfaces; cbindgen only if C++ shim used.

## Threat Model
Untrusted puzzle specs; strict parsing; bounds checks; avoid panics; DOS via pathological puzzles mitigated by time/step budgets; Kotlin ↔ Rust boundary validated types.

## Telemetry
Debug-only profiling; no PII; opt-in difficulty metrics; tracing filtered on Android.
