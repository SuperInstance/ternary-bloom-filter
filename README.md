# ternary-bloom-filter

**Ternary Bloom filter with {-1, 0, +1} weighted bits for GPU membership testing. Positive bits boost membership, negative bits block it.**

## Background

Bloom filters (Burton H. Bloom, 1970) are a probabilistic data structure for set membership testing. They use an array of bits and multiple hash functions to answer "is this item in the set?" with zero false negatives and bounded false positives.

`ternary-bloom-filter` extends the classic Bloom filter by replacing binary bits (0/1) with **ternary bits** {-1, 0, +1}:

| Value | Meaning | Effect |
|-------|---------|--------|
| +1 | Positive | Item is in the set (promoted) |
| 0 | Neutral | No information |
| -1 | Negative | Item is explicitly excluded (blocked) |

This creates a **weighted membership test**: not just "is it in the set?" but "is it in the set, or is it explicitly banned?" The negative bits act as a *blocklist* overlaid on the Bloom filter, enabling "membership with exceptions."

## How It Works

### Data Structure

A fixed-size array (default 1024) of `i8` values, each in {-1, 0, +1}. Items are hashed with `hash_count` independent hash functions.

### Operations

- **`insert_positive(item)`**: Set all hash positions to +1. Marks the item as a member.
- **`insert_negative(item)`**: Set all hash positions to -1. Explicitly blocks the item.
- **`check(item) → i32`**: Sum the values at all hash positions. Positive = likely present. Negative = likely blocked. Zero = unknown.
- **`contains(item) → bool`**: `check(item) > 0`
- **`blocked(item) → bool`**: `check(item) < 0`

### GPU Packing

The filter can be packed into `Vec<u32>` where each `u32` holds 16 ternary values (2 bits each):

```
-1 → 0b11, +1 → 0b01, 0 → 0b00
```

This compact encoding enables the filter to be uploaded to GPU constant memory for kernel-time membership testing.

### Merge

Filters can be merged conservatively: positive wins ties, negative beats neutral, and positive+negative conflicts resolve to neutral (0).

## Experimental Results

The test suite validates:

- **Positive insertion and detection**: Items inserted as positive are detected by `contains()`.
- **Negative blocking**: Items inserted as negative are detected by `blocked()`.
- **Absent items**: Items not in the filter return `check() == 0`.
- **Score magnitude**: Items inserted with more hash functions get higher `check()` scores.
- **Merge correctness**: Merged filters preserve both sources' positive entries.
- **GPU packing**: Packed representation has correct length (1024 / 16 = 64 `u32`s).
- **Fill rate tracking**: Fill rate increases as items are inserted.

## Impact for GPU Cluster Computing

Ternary Bloom filters are uniquely valuable in GPU environments:

- **GPU-native**: The 2-bit encoding fits perfectly into GPU shared memory. Membership tests compile to XNOR + popcount — the same instructions used in Binary Neural Networks.
- **Blocklist overlay**: A single filter serves dual purpose: "these kernels are approved (+1)" and "these kernels are banned (-1)." No separate blocklist needed.
- **Mergeable**: Filters from different GPU nodes can be merged, enabling distributed membership tracking without central coordination.

## Use Cases

1. **Kernel Authorization**: A GPU node maintains a ternary Bloom filter of authorized (+1) and banned (-1) kernel hashes. Before executing a kernel, it checks the filter — blocked kernels are rejected instantly.
2. **Cache Admission Control**: A GPU cache uses ternary Bloom filters: +1 for frequently-accessed data (admit to cache), -1 for cold data (skip cache), 0 for unknown (admit with low priority).
3. **Distributed Deduplication**: GPU nodes merge their Bloom filters to track which data blocks exist cluster-wide. Positive = "I have it," negative = "confirmed deleted."
4. **Request Filtering**: An inference gateway uses negative Bloom filter bits to block known-malicious or duplicate requests before they reach GPU workers.

## Open Questions

1. **Optimal hash count**: What is the optimal number of hash functions for ternary Bloom filters, given that the three-valued bits change the false positive/negative calculus?
2. **Counting variant**: Can a counting ternary Bloom filter (using >1-bit counters) support deletions without the negative-bit ambiguity?
3. **GPU kernel integration**: What is the throughput of ternary Bloom filter checks on modern GPUs (e.g., H100)? Can it reach 100M checks/second?

## Connection to Oxide Stack

`ternary-bloom-filter` is the **observation layer** of the GPU runtime:

| Layer | Crate | Role |
|-------|-------|------|
| 1 — Tracking | `ternary-bloom-filter` | Compact membership tracking |
| 2 — Communication | `ternary-epidemic` | Bloom filters prevent re-gossip |
| 3 — Routing | `ternary-routing` | Route health tracked via Bloom filters |
| 4 — Search | `ternary-search-index` | Similar packed ternary encoding |
| 5 — Analytics | `ternary-sketch` | Complementary approximate analytics |

Bloom filters are the "eyes" of the cluster — they tell nodes what they've seen and what they haven't.
