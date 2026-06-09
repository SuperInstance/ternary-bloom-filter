# ternary-bloom-filter

Ternary Bloom filter for GPU-accelerable membership testing. {-1,0,+1} weighted bits allow boost/block/neutral membership. 16x denser than FP32.

## Why This Matters

# ternary-bloom-filter
Ternary Bloom filter: {-1,0,+1} weighted bits for GPU membership testing.
Positive bits boost membership, negative bits block it.

## The Five-Layer Stack

This crate is part of the **Oxide Stack** — a distributed GPU runtime built on five layers:

```
┌─────────────────┐
│  cudaclaw        │  Persistent GPU kernels, warp consensus, SmartCRDT
├─────────────────┤
│  cuda-oxide      │  Flux → MIR → Pliron → NVVM → PTX compiler
├─────────────────┤
│  flux-core       │  Bytecode VM + A2A agent protocol
├─────────────────┤
│  pincher         │  "Vector DB as runtime, LLM as compiler"
├─────────────────┤
│  open-parallel   │  Async runtime (tokio fork)
└─────────────────┘
```

The key insight: **ternary values {-1, 0, +1} map directly to GPU compute**. They pack 16× denser than FP32, enable XNOR+popcount matmul, and conservation laws become compile-time checks.

## Design

Every value in this crate follows **ternary algebra** (Z₃):

| Value | Meaning | GPU Analog |
|-------|---------|------------|
| +1 | Positive / Active / Healthy | Warp vote yes |
| 0 | Neutral / Pending / Balanced | Warp vote abstain |
| -1 | Negative / Failed / Overloaded | Warp vote no |

This isn't arbitrary — ternary is the natural encoding for:
1. **BitNet b1.58** (Microsoft) — ternary LLMs at 60% less power
2. **GPU warp voting** — hardware ballot returns ternary consensus
3. **Conservation laws** — {-1, 0, +1} preserves quantity

## Key Types

```rust
pub struct TernaryBloom
pub fn new
pub fn insert_positive
pub fn insert_negative
pub fn check
pub fn contains
pub fn blocked
pub fn merge
pub fn pack_for_gpu
pub fn positive_count
pub fn negative_count
pub fn fill_rate
```

## Usage

```toml
[dependencies]
ternary-bloom-filter = "0.1.0"
```

```rust
use ternary_bloom_filter::*;
// See src/lib.rs tests for complete working examples
```

## Testing

```bash
git clone https://github.com/SuperInstance/ternary-bloom-filter.git
cd ternary-bloom-filter
cargo test    # 8 tests
```

## Stats

| Metric | Value |
|--------|-------|
| Tests | 8 |
| Lines of Rust | 151 |
| Public API | 12 items |

## License

Apache-2.0
