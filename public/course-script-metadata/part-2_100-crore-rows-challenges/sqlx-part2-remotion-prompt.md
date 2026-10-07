# 🎞️ Remotion Prompt — 100 Crore Rows: 13 Real Challenges with SQLx (Part 2 of 2)

Paste this file into your AI coding agent or Remotion project generator. It defines the visual architecture, component contracts, and scene maps for all 18 sections and 13 challenges in Part 2.

---

## 0. What to Build

A Remotion project rendering **one continuous 16:9 master video** (1920×1080 @ 60 FPS) for Part 2. The video is a benchmark and performance engineering walkthrough of a PostgreSQL table populated with 100 crore rows (1 billion rows). 

**The Cardinal Rule:** Every measured latency, throughput metric, buffer page count, and plan tree on screen MUST be loaded directly from real JSON benchmark files in `public/data/`. Never hard-code fake benchmark numbers.

---

## 1. Render Specifications

- **Resolution:** 1920×1080, 60 fps, TypeScript, React, Remotion 4.x+.
- **Master Composition:** `Part2Master`, built with Remotion `<Series>`:
  - One `<Series.Sequence>` per script **Section / Challenge**.
  - One nested `<Sequence>` per script **Part**.
- **Timing Source:** `public/audio/part2/timings.json`. Use `calculateMetadata` to derive:
  `durationInFrames = Math.round(seconds * 60)`.
- **Transitions:** 0.4s (24 frames) silence between Parts; 0.8s (48 frames) with orange-to-cyan wipe between Sections.
- **Visual Beats:** Fractional beats within each Part (e.g. 0.0, 0.35, 0.70).

---

## 2. Design System & Theme

- **Background:** Deep obsidian `#08090C`, panels `#11141A`, borders `#1F2430`.
- **Convention:** Naive / Slow path in **Vivid Orange** (`#FF6633`), Optimized / Fast path in **Electric Cyan** (`#06B6D4`), Success in Green (`#10B981`), Failure/Timeout in Red (`#EF4444`).
- **Typography:**
  - Code, Latencies & Plans: JetBrains Mono (weight 500/700).
  - Headings & Lower Thirds: Inter (weight 600/800).
- **Big Metric Style:** 140px JetBrains Mono with 30-frame count-up easing.
- **Lower Thirds:** Max 8 words, Inter 40px, displaying the challenge rule.
- **Branding:** Persistent bottom-right corner badge "Aarambh Dev Hub" at 60% opacity.

---

## 3. Data Contracts (`public/data/`)

```text
public/data/
  lab-settings.json             // PostgreSQL SHOW settings & hardware specs
  headline.json                 // { rows: 1000000000, scan_ms: 164210, index_ms: 0.42 }
  load-benchmarks.json          // Array of { method, rows_per_sec, total_sec, bind_count }
  table-size.json               // { heap_bytes: 77309411328, index_bytes: 25769803776 }
  plans/
    c01_seq_scan.json           // Full EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)
    c01_index_scan.json         // B-tree index scan execution plan
    c01_index_only.json         // Covering index only scan execution plan
    c03_trgm_gin.json           // GIN trigram scan plan
    c05_offset_vs_keyset.json   // Plan comparing offset vs keyset
    c07_partition_prune.json    // Partition pruning execution plan
    c13_skip_locked.json        // SKIP LOCKED worker execution plan
  results/
    c01_summary.json            // Latencies: seq scan, btree, citext, covering, hash, brin
    c02_availability.json       // Latencies: count vs exists vs bloom
    c04_counts.json             // Latencies: count(*) vs reltuples vs trigger
    c05_deep_pages.json         // Latencies by page depth (page 1 to page 500,000)
    c08_top_n.json              // Window functions vs DISTINCT ON vs LATERAL
    c11_deletion.json           // DELETE vs Partition Drop timings
    c12_counters.json           // Row lock contention vs in-memory channel vs sharded
    master_summary.json         // Master comparison table across all 13 challenges
```

---

## 4. Core Shared Components

| Component | Props | Description |
|---|---|---|
| `PlanTree` | `planJson: object, highlightNodes: string[]` | Visual tree of PostgreSQL query plan nodes showing buffers hit/read and actual times. |
| `BigCounter` | `value: number, unit: string, tone: 'orange'\|'cyan'` | Count-up animated number with unit label. |
| `RowGridScan` | `totalRows: number, scanSpeed: number, matchIndex: number` | Dense grid of 10,000 micro-cells representing 1B rows with an active scanning head. |
| `LaneRace` | `lanes: {name: string, durationMs: number, color: string}[]` | Horizontal bar race animating relative query completion speeds. |
| `SplitMetric` | `naive: {val: string, label: string}, opt: {val: string, label: string}` | Before/After split comparison card (e.g. 2m 44s vs 0.42 ms). |
| `CodeBlock` | `code: string, lang: string, highlights: number[]` | VS Code editor view highlighting key SQL/Rust lines. |
| `LowerThird` | `text: string` | Bottom floating bar displaying the challenge rule. |

---

## 5. Complete Section Scene Map

### Section 1: HOOK AND THE HEADLINE DEMO (`s01-hook`)
- **Part 1:** Dark obsidian screen. Massive BigCounter flashes `1,000,000,000 ROWS`. SplitMetric card appears: Left (Orange) `2m 44s`, Right (Cyan) `0.42 ms`.
- **Part 2:** 3D visualization of a massive 72 GB storage volume on an NVMe drive. Graphic showing 9 million disk pages being read one by one during a full table scan.
- **Part 3:** Terminal recording of the real benchmark harness: First run outputting `164,210 ms (Seq Scan)`. Second run outputting `0.42 ms (Index Scan)`. LowerThird: "0.42 ms in 100 Crore Rows".
- **Part 4:** 13 Challenge cards cascade onto screen in a grid, previewing the journey ahead.

### Section 2: TEST LAB SETUP, HARDWARE, AND POSTGRES SETTINGS (`s02-lab`)
- **Part 1:** Hardware HUD card: 16 Physical Cores, 64 GB DDR5 RAM, Enterprise NVMe SSD (5,000 MB/s).
- **Part 2:** Terminal showing PostgreSQL settings: `shared_buffers = 16GB`, `work_mem = 64MB`, `maintenance_work_mem = 4GB`, `effective_cache_size = 48GB`.
- **Part 3:** CodeBlock showing Rust benchmark harness in `src/benchmarks/harness.rs`: Warm-up iterations, p50/p95/p99 tracking.
- **Part 4:** PlanTree component parsing `EXPLAIN (ANALYZE, BUFFERS)`. Highlight nodes: `Buffers: shared hit=4 read=0`.

### Section 3: GENERATING AND LOADING 100 CRORE ROWS: THE INGESTION RACE (`s03-ingestion`)
- **Part 1:** LaneRace comparing 4 ingestion methods: Single Inserts, QueryBuilder, UNNEST, COPY.
- **Part 2:** Single inserts bar crawls at 600 rows/sec. Clock counts up to 19 days.
- **Part 3:** CodeBlock showing `QueryBuilder` batching. Red alert badge: "Postgres parameter limit: 65,535 reached at 21,845 rows".
- **Part 4:** CodeBlock showing `UNNEST($1, $2, $3)` using array parameters. Counter: 110,000 rows/sec.
- **Part 5:** CodeBlock showing `copy_in_raw` streaming CSV bytes directly. Counter jumps to 250,000 rows/sec.
- **Part 6:** Architecture diagram: 8 Tokio worker tasks feeding PostgreSQL connections in parallel. Metric hits `1,200,000 rows/sec`.
- **Part 7:** Table footprint card: `pg_total_relation_size = 72 GB`. Terminal runs `ANALYZE users`. LowerThird: "Drop indexes before load; build after load".

### Section 4: CHALLENGE 1: FIND ONE USERNAME (`c01-lookup`)
- **Part 1:** Challenge card: C1 — Find One Username. Query: `SELECT id, email FROM users WHERE username = $1`.
- **Part 2:** RowGridScan animates sequential scan: red-orange scanner sweeps millions of cells. PlanTree shows `Seq Scan on users`, 9,150,000 pages read, 164s.
- **Part 3:** Visual B-Tree traversal: Root node -> Level 1 -> Level 2 -> Leaf page. 4 page reads total. Latency: `0.45 ms`.
- **Part 4:** CodeBlock showing `WHERE lower(username) = 'alice'`. PlanTree flashes back to `Seq Scan`. Warning: "Function call invalidates standard B-Tree".
- **Part 5:** CodeBlock showing `CREATE INDEX ON users (lower(username))` and `citext` extension. Plan switches back to `0.44 ms`.
- **Part 6:** PlanTree showing `Index Only Scan` using `INCLUDE (email, created_at)`. Table heap access: 0 pages.
- **Part 7:** Table comparing Hash Index (no ranges, large size) vs Partial Index (`WHERE status = 'active'` cutting index size by 80%).
- **Part 8:** BRIN index diagram (Block Range Index): 28 MB vs 24 GB B-Tree. Random username scan fails on BRIN. Master index comparison chart. LowerThird: "Use B-Tree with INCLUDE for point lookups".

### Section 5: CHALLENGE 2: SIGNUP USERNAME AVAILABILITY CHECK (`c02-availability`)
- **Part 1:** Registration form UI UI typing "darshan". Query check: Does username exist?
- **Part 2:** Naive `SELECT count(*) FROM users WHERE username = $1` plan vs `SELECT EXISTS(...)`.
- **Part 3:** Index scan with `EXISTS` halting at first index match. Latency: `0.28 ms`.
- **Part 4:** Bloom filter bit-array visualization in Rust memory. 0 false negatives. Zero DB queries for available names. LowerThird: "Use EXISTS and in-memory Bloom filters".

### Section 6: CHALLENGE 3: AUTOCOMPLETE AND FUZZY SEARCH (`c03-autocomplete`)
- **Part 1:** Search box input typing "ali...". Query: `WHERE username LIKE '%ali%'`.
- **Part 2:** PlanTree shows sequential scan for leading wildcard `%`. Latency: 160s.
- **Part 3:** CodeBlock showing `CREATE INDEX ON users (username text_pattern_ops)`. Prefix search `LIKE 'ali%'` drops to `0.6 ms`.
- **Part 4:** GIN Trigram index visual: "alice" split into ["  a", " al", "ali", "lic", "ice", "ce "]. Inverted index lookups in `12 ms`. LowerThird: "text_pattern_ops for prefixes; GIN trigrams for fuzzy".

### Section 7: CHALLENGE 4: THE DASHBOARD COUNT NIGHTMARE (`c04-counts`)
- **Part 1:** Admin dashboard displaying user count card.
- **Part 2:** PostgreSQL MVCC explanation: Why `count(*)` scans all pages to check transaction visibility. Latency: 135s.
- **Part 3:** CodeBlock querying `pg_class.reltuples`. Latency: `0.12 ms` (99.9% accurate).
- **Part 4:** Trigger-maintained single-row counter table. Latency: `0.30 ms` (100% exact). LowerThird: "Use reltuples for dashboards; counters for exact totals".

### Section 8: CHALLENGE 5: DEEP PAGINATION (`c05-pagination`)
- **Part 1:** Page selector jumping to Page 500,000.
- **Part 2:** Diagram showing `OFFSET 10000000 LIMIT 20`: Database reads 10 million rows and discards them. Latency: 6.2s.
- **Part 3:** Keyset cursor diagram: B-Tree index seek directly to `WHERE id > 10000000 LIMIT 20`. Latency: `0.40 ms`.
- **Part 4:** Line chart: `OFFSET` latency line climbing steeply into seconds vs Keyset pagination staying flat at 0.4 ms forever. LowerThird: "Replace OFFSET with keyset pagination (WHERE id > cursor)".

### Section 9: CHALLENGE 6: EXPORTING MASSIVE DATASETS (`c06-export`)
- **Part 1:** CSV Export button clicked for 50 million rows.
- **Part 2:** Rust process memory graph: `fetch_all` triggers exponential RAM growth -> OOM Killer SIGKILL.
- **Part 3:** Tokio stream visualization: `fetch` yielding bounded 10,000-row chunks. RAM stays flat at 30 MB.
- **Part 4:** Native wire protocol streaming: `COPY users TO STDOUT (FORMAT CSV)`. Throughput hits 1,000,000 rows/sec. LowerThird: "Stream with fetch or COPY TO STDOUT; never fetch_all".

### Section 10: CHALLENGE 7: QUERYING THE LAST 24 HOURS OF EVENTS (`c07-timeseries`)
- **Part 1:** Events table timeline: 100 crore historical rows, 1 million new rows per day.
- **Part 2:** 20 GB B-Tree index on `created_at` updating on every write (massive write amplification).
- **Part 3:** BRIN index visual: 12 MB index mapping physical page ranges to time windows. Latency: `1.8 ms`.
- **Part 4:** Declarative Range Partitioning visual: Table sliced into daily partitions. PlanTree shows Partition Pruning: historical partitions bypassed. Latency: `0.3 ms`. LowerThird: "Use range partitions for time-series data retention".

### Section 11: CHALLENGE 8: TOP-N AND LATEST RECORD PER USER (`c08-top-n`)
- **Part 1:** Query requirement: Fetch latest 3 orders for 10,000 users.
- **Part 2:** Correlated subquery quadratic nested loops crawling on screen.
- **Part 3:** `DISTINCT ON (user_id)` index scan extracting single latest order in 1 pass.
- **Part 4:** `LATERAL` join index seek benchmarked against window functions `ROW_NUMBER()`. Lateral join wins 4x faster. LowerThird: "DISTINCT ON for latest record; LATERAL joins for top-N".

### Section 12: CHALLENGE 9: DETECTING AND CLEANING BILLION-ROW DUPLICATES (`c09-duplicates`)
- **Part 1:** Data corruption scenario: Duplicate user records found.
- **Part 2:** `GROUP BY email HAVING count(*) > 1` plan spilling to disk with default `work_mem = 4MB`. Hundreds of temp files created.
- **Part 3:** `SET work_mem = '2GB'`: Hash aggregate executes purely in RAM. Execution time drops from 3 hours to 4 minutes.
- **Part 4:** CodeBlock showing batched deletion script in Rust using `ctid` in 10,000-row chunks. LowerThird: "Boost work_mem to prevent disk spills; delete in chunks".

### Section 13: CHALLENGE 10: ZERO-DOWNTIME LIVE MIGRATIONS (`c10-migrations`)
- **Part 1:** Production web traffic graph handling 5,000 requests/sec.
- **Part 2:** Standard `CREATE INDEX` executed: Red table lock (`SHARE` lock) freezes all incoming writes. Connection pool collapses.
- **Part 3:** `CREATE INDEX CONCURRENTLY` executed with `SET lock_timeout = '2s'`. Traffic graph stays green without interruption.
- **Part 4:** Keyset column backfill loop in Rust updating 20,000 rows per transaction. LowerThird: "Always use CONCURRENTLY with lock_timeout on live tables".

### Section 14: CHALLENGE 11: DELETING OLD DATA AT SCALE (`c11-deletion`)
- **Part 1:** Policy requirement: Purge 90-day-old event data (100 million rows).
- **Part 2:** Bulk `DELETE` execution: Massive dead tuple bloat, gigabytes of WAL generated, replication lag spikes.
- **Part 3:** Batched chunk delete in Rust (`DELETE ... WHERE id IN (...) LIMIT 50000`).
- **Part 4:** Range partition drop: `DROP TABLE events_2025_01`. Latency: `0.08 ms`. Zero WAL, zero dead tuples. LowerThird: "Drop partitions instead of running bulk DELETE".

### Section 15: CHALLENGE 12: THE HOT COUNTER CONTENTION PROBLEM (`c12-counters`)
- **Part 1:** Viral post receiving 10,000 concurrent view updates per second.
- **Part 2:** `UPDATE posts SET views = views + 1`: Row-level lock queue explodes. CPU context switching spikes.
- **Part 3:** In-memory Tokio mpsc channel buffering increments in Rust; flushing 1 batch update every 100 ms.
- **Part 4:** Sharded counter visual: Counter distributed across 16 rows. Contention slashed 16x. LowerThird: "Buffer increments in memory or shard hot counters".

### Section 16: CHALLENGE 13: HIGH-THROUGHPUT JOB QUEUES (`c13-queues`)
- **Part 1:** 32 parallel worker tasks polling PostgreSQL for pending jobs.
- **Part 2:** Naive polling: Workers collide on the same job ID. Deadlocks and transaction rollbacks.
- **Part 3:** `SELECT ... FOR UPDATE SKIP LOCKED LIMIT 1`: Lock manager skips locked rows instantly.
- **Part 4:** Throughput chart: Linear scaling across all 32 workers with 0 deadlocks. LowerThird: "Build transactional job queues with SKIP LOCKED".

### Section 17: PRODUCTION AT SCALE: POOL SIZING & DIAGNOSTICS (`s14-production`)
- **Part 1:** Pool sizing curve: 20 connections vs 200 connections. Formula: `Pool = (CoreCount * 2) + SpindleCount`.
- **Part 2:** Terminal showing `pg_stat_statements` identifying top 3 slowest queries.
- **Part 3:** `auto_explain` logs capturing slow queries automatically.
- **Part 4:** Autovacuum tuning (`scale_factor = 0.05`) and read-replica routing in SQLx.

### Section 18: CAPSTONE ARCHITECTURE AND MASTER SUMMARY (`s15-capstone`)
- **Part 1:** Capstone Axum service architecture diagram in Rust integrating all 13 winning strategies.
- **Part 2:** Code walkthrough in VS Code of `src/capstone/service.rs`.
- **Part 3:** Master Comparison Table displaying all 13 challenges side-by-side with naive vs optimized latencies.
- **Part 4:** Closing branding card and call to action: "Subscribe to Aarambh Dev Hub".
