# ternary-bloom-filter

Ternary Bloom filter with {-1, 0, +1} weighted bits for membership and exclusion testing.

## Why This Exists

A standard Bloom filter answers "probably in set" or "definitely not in set." But in some workloads you also need "definitely excluded" — a stronger negative signal. A ternary Bloom filter extends each bit to a ternary counter: positive bits boost membership confidence, negative bits actively block it. Items inserted as "negative" act as a permanent exclusion list. This gives you both a membership filter and a blocklist in one data structure, GPU-packable as `Vec<u32>`.

## Architecture

### Core Types

- **`TernaryBloom`** — Array of `i8` counters, each hash function maps to a position.
  - `insert_positive`: Increment counters → boosts membership signal.
  - `insert_negative`: Decrement counters → creates exclusion signal.
  - `check`: Sum of all hash positions → positive means likely member, negative means likely blocked, zero means unknown.

### GPU Packing

`pack_for_gpu()` converts the i8 array into a packed u32 representation for GPU upload.

## Usage

```rust
use ternary_bloom_filter::TernaryBloom;

let mut filter = TernaryBloom::new(3); // 3 hash functions

// Insert allowed kernels
filter.insert_positive(b"matmul");
filter.insert_positive(b"conv2d");
filter.insert_positive(b"layernorm");

// Block dangerous kernels
filter.insert_negative(b"eval");  // never execute
filter.insert_negative(b"shell");

// Query
assert!(filter.contains(b"matmul"));        // positive signal
assert!(filter.blocked(b"eval"));            // negative signal
assert_eq!(filter.check(b"unknown"), 0);     // neutral

// Pack for GPU upload
let gpu_data: Vec<u32> = filter.pack_for_gpu();
```

## API Reference

| Method | Returns | Description |
|--------|---------|-------------|
| `new(hash_count)` | `TernaryBloom` | Create filter with N hash functions |
| `insert_positive(item)` | `()` | Boost item's membership signal |
| `insert_negative(item)` | `()` | Block item's membership signal |
| `check(item)` | `i32` | Sum of hash positions (>0 member, <0 blocked) |
| `contains(item)` | `bool` | Check > 0 |
| `blocked(item)` | `bool` | Check < 0 |
| `merge(other)` | `()` | Merge another filter into this one |
| `pack_for_gpu()` | `Vec<u32>` | Pack i8 array to u32 for GPU upload |
| `positive_count()` / `negative_count()` | `u64` | Insertion counters |
| `fill_rate()` | `f64` | Fraction of non-zero positions |

## The Deeper Idea

The ternary Bloom filter is a **soft firewall**. Traditional firewalls are binary (allow/deny). Traditional Bloom filters are positive-only (in-set/not-in-set). A ternary Bloom filter combines both: it can simultaneously track "these are good" and "these are bad" with a single query. The neutral state (score = 0) means "no evidence either way," which is the correct default for security-sensitive systems — you don't want to accidentally allow something you haven't explicitly reviewed.

## Related Crates

- **ternary-sketch** — Count-Min sketch with ternary counters
- **ternary-search-index** — ternary-weighted document search
- **ternary-pack** — bit-packing ternary values into u32
