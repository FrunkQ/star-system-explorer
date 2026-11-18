# ADR 0002: pinned maths and float policy

Status: accepted (K1).

- All transcendental functions on deterministic paths go through `sse_core::math`, which calls `libm` at an exact pinned version (`=0.2.16`). `+ - * / sqrt` are IEEE correctly rounded on every target and used directly.
- No fused multiply-add (Rust never contracts on its own; `mul_add` is banned by clippy). No fast-math.
- NaN: payload and sign are not portable, so NaN is never a value. The encoder rejects it, the harness folds every NaN to one bit pattern, and (from K4) an atom producing NaN is an error.
- Evidence: `tests/golden/harness.json` reproduced natively (Linux x86-64, macOS arm64, Windows x86-64 in CI), in Node, and in Chromium, Firefox and WebKit.
