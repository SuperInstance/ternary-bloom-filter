# Ternary Bloom Filter

**Ternary Bloom Filter** is a GPU-accelerable membership testing data structure using {-1, 0, +1} weighted bits — positive bits boost membership confidence, negative bits block it, and zero bits are neutral. This provides 16× memory density over FP32 Bloom filters.

## Why It Matters

Standard Bloom filters answer "is this item in the set?" with true/probably-false. They can't answer "is this item explicitly excluded?" Ternary Bloom filters can — negative insertions mark items as blocked, and queries return a signed confidence score rather than a boolean. This enables blocklist+allowlist filtering in a single data structure, critical for fleet access control where both inclusion and exclusion matter simultaneously.

## How It Works

### Standard Bloom Filter Refresher

A Bloom filter uses k hash functions to map items to positions in a bit array:

```
insert(item):
    for i in 0..k:
        bits[hash_i(item)] = 1

check(item):
    return AND(bits[hash_i(item)] for i in 0..k)
```

False positive rate: (1 - e^(-kn/m))^k for n items, m bits, k hashes.

### Ternary Extension

Instead of bits {0, 1}, the array uses trits {-1, 0, +1}:

```
insert_positive(item):
    for i in 0..k: bits[hash_i(item)] = +1

insert_negative(item):
    for i in 0..k: bits[hash_i(item)] = -1

check(item) → Σ bits[hash_i(item)] for i in 0..k
    score > 0: probably in positive set
    score < 0: probably in negative set
    score = 0: not in either set (or conflicting)
```

Hash function: `h(data, seed) = (seed · 31 + Σ byte) mod m`. Cost: **O(L + k)** where L = data length.

### False Positive Rate

With n+ items in the positive set and n- in the negative set:

```
P(false positive) ≈ (1 - e^(-k·n+/m))^k
```

The negative set doesn't increase false positives (it adds true negatives). But conflicts (same hash position for + and - items) increase uncertainty — the check returns 0 instead of ±k.

### Memory Density

```
FP32 Bloom:  32 bits per slot
Binary Bloom: 1 bit per slot
Ternary Bloom: 2 bits per slot (encodes -1, 0, +1)

Density vs FP32: 32/2 = 16×
Density vs binary: 1/2 = 0.5× (half as dense, but signed!)
```

### Merge Operation

```rust
merge(other):
    for (a, b) in bits.iter_mut().zip(other.bits):
        if b != 0 && a == 0: a = b   // adopt non-zero
        if a == b: continue           // agree
        // Conflicting: keep existing (conservative)
```

Merge: **O(m)** where m = filter size.

## Quick Start

```rust
use ternary_bloom_filter::TernaryBloom;

let mut filter = TernaryBloom::new(3); // 3 hash functions

filter.insert_positive(b"trusted_agent");
filter.insert_negative(b"banned_agent");

assert!(filter.contains(b"trusted_agent"));   // true (score > 0)
assert!(filter.blocked(b"banned_agent"));     // true (score < 0)
assert!(!filter.contains(b"unknown"));        // false (score = 0)
```

## API

| Type | Description |
|------|-------------|
| `TernaryBloom` | Vec<i8> of {-1, 0, +1} with k hash functions |
| `insert_positive(item)` | Boost membership for item |
| `insert_negative(item)` | Block membership for item |
| `check(item) → i32` | Signed confidence score |
| `contains(item) → bool` | True if check > 0 |
| `blocked(item) → bool` | True if check < 0 |
| `merge(other)` | Merge two filters (conservative) |

## Architecture Notes

Ternary Bloom Filter provides GPU-accelerable membership testing for fleet access control in SuperInstance. In γ + η = C, positive insertions represent γ (growth — adding trusted agents to the allowlist) while negative insertions represent η (avoidance — blocking hostile agents via the blocklist). The signed check score naturally encodes the γ - η difference. The 2-bit encoding is compatible with GPU ternary packing from `ternary-benchmark`.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for fleet security architecture.

## References

1. Bloom, B. H. (1970). "Space/Time Trade-offs in Hash Coding with Allowable Errors." *Communications of the ACM*, 13(7), 422–426.
2. Fan, B. et al. (2014). "Cuckoo Filter: Practically Better Than Bloom." *Proceedings of the 10th ACM International on Conference on Emerging Networking Experiments and Technologies*.
3. Mitzenmacher, M. & Upfal, E. (2017). *Probability and Computing*, 2nd ed. Cambridge University Press.

## License

Apache-2.0
