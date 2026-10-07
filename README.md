# 🚀 Aarambh Dev Hub: SQLx + PostgreSQL Master Course (Part 1 & Part 2)

A production-grade Rust codebase demonstrating advanced PostgreSQL and SQLx 0.9 patterns, real-world high-throughput architectures, and 13 production challenges scaling to 1,000,000,000 (100 Crore) rows.

---

## 📋 Course Overview

This repository accompanies two 3-hour masterclass videos:
1. **Part 1: SQLx + PostgreSQL in Rust: Zero to Advanced (SQLx 0.9)**
   - Asynchronous connection pooling with `PgPoolOptions` and healthchecks.
   - Raw queries (`query`), struct mapping (`query_as`, `FromRow`), and scalar extraction (`query_scalar`).
   - Compile-time macros (`query!`, `query_as!`) and offline mode preparation.
   - Dynamic query building with `QueryBuilder` and SQLx 0.9 `AssertSqlSafe`.
   - Advanced Postgres types: `Uuid` (v7), `Decimal`, `DateTime<Utc>`, Postgres Enums, `JSONB`, and native arrays (`TEXT[]`).
   - Joins, aggregations, CTEs, recursive CTEs, Window functions, `LATERAL` joins, and `DISTINCT ON`.
   - Bulk DML: `ON CONFLICT DO UPDATE` (upsert), `RETURNING`, array `UNNEST`, and `copy_in_raw`.
   - Database logic: Views, Materialized Views with concurrent refresh, stored generated columns, Triggers, and `LISTEN/NOTIFY` via `PgListener`.
   - Concurrency & Transactions: Nested savepoints, isolation levels, row locking (`FOR UPDATE`), deadlock detection (`40P01`), and optimistic concurrency (CAS).
   - Architectural patterns: Repository Pattern with `PgExecutor` and PostgreSQL error code mapping (23505, 23503, 23514).
   - Multi-tenant security with PostgreSQL Row-Level Security (RLS) and `SET LOCAL app.tenant_id`.

2. **Part 2: 100 Crore Rows in PostgreSQL: 13 Real Challenges with SQLx**
   - High-throughput Ingestion Race comparing Single Inserts, `QueryBuilder` batches, Array `UNNEST`, and parallel streaming `COPY`.
   - **Challenge 01:** Find One Username (Seq Scan vs B-Tree vs `lower()` vs Covering vs BRIN).
   - **Challenge 02:** Signup Availability Check (`count(*)` vs `EXISTS` vs In-Memory Bloom Filter).
   - **Challenge 03:** Autocomplete & Fuzzy Search (`text_pattern_ops` prefix vs `pg_trgm` GIN trigrams).
   - **Challenge 04:** Dashboard Count Nightmare (`count(*)` vs `pg_class.reltuples` vs Trigger Counter Table).
   - **Challenge 05:** Deep Pagination (`OFFSET` degradation vs Keyset Cursor `WHERE id > $cursor`).
   - **Challenge 06:** Massive Dataset Export (Memory crash avoidance: Tokio Stream vs `COPY TO STDOUT`).
   - **Challenge 07:** Time-Series Events (B-Tree write bloat vs BRIN block range index vs Declarative Range Partitioning).
   - **Challenge 08:** Top-N & Latest Record Per User (Correlated subqueries vs Windows vs `DISTINCT ON` vs `LATERAL` join).
   - **Challenge 09:** Detecting & Purging Duplicates (`work_mem` disk spill avoidance & batched `ctid`/PK deletion).
   - **Challenge 10:** Zero-Downtime Live Migrations (`CREATE INDEX CONCURRENTLY` + `lock_timeout` + keyset backfill loop).
   - **Challenge 11:** Deleting Old Data at Scale (Bulk `DELETE` WAL thrashing vs Batched purge vs Instant Partition Drop).
   - **Challenge 12:** Hot Counter Contention Problem (Row lock queue vs Tokio In-Memory write buffer channel vs Sharded counters).
   - **Challenge 13:** Transactional Job Queues (Worker collision & deadlocks vs `FOR UPDATE SKIP LOCKED`).
   - Production Scale tuning: PostgreSQL connection pool sizing math, `pg_stat_statements`, `auto_explain`, and autovacuum tuning.
   - Production Capstone Axum REST API integrating winning solutions.

---

## 🛠️ Tech Stack & Prerequisites

- **Rust:** 1.94+ (Edition 2021)
- **SQLx:** `0.9.0` with `postgres`, `runtime-tokio`, `tls-rustls`, `macros`, `uuid`, `chrono`, `rust_decimal`, `json`, `migrate`, `sqlx-toml`
- **Tokio:** `1.40+` (full features)
- **Axum:** `0.8+` (web framework for Capstone service)
- **PostgreSQL:** 16+ running locally on `localhost:5432`
- **CLI Tool:** `sqlx-cli` 0.9.0

---

## 📁 Repository Structure

```text
sqlx-mastery-course/
├── Cargo.toml                          # Manifest with default-run and dependencies
├── sqlx.toml                           # SQLx 0.9 config (preferred-crates, migrations)
├── .env                                # Local configuration (DATABASE_URL)
├── .env.example                        # Template environment variables
├── .sqlx/                              # Offline query metadata for compile-time macros
├── migrations/                         # Reversible migrations (up & down)
│   ├── 20260501000001_create_users_and_orders.up.sql
│   ├── 20260501000001_create_users_and_orders.down.sql
│   ├── 20260501000002_create_database_logic.up.sql
│   ├── 20260501000002_create_database_logic.down.sql
│   ├── 20260501000003_setup_row_level_security.up.sql
│   ├── 20260501000003_setup_row_level_security.down.sql
│   ├── 20260502000001_setup_crore_lab_tables.up.sql
│   └── 20260502000001_setup_crore_lab_tables.down.sql
├── src/
│   ├── lib.rs                          # Library root re-exporting modules
│   ├── main.rs                         # CLI application runner
│   ├── pool.rs                         # PgPoolOptions initialization, healthcheck, sizing
│   ├── part1/                          # Part 1: Zero to Advanced
│   │   ├── mod.rs
│   │   ├── schema.rs                   # User, Order, UserProfile, UserRole models
│   │   ├── queries_basics.rs           # query, query_as, query_scalar, fetch variants
│   │   ├── queries_macros.rs           # query!, query_as! compile-time verified
│   │   ├── queries_dynamic.rs          # QueryBuilder & AssertSqlSafe dynamic clauses
│   │   ├── types_custom.rs             # UUIDv7, Decimal, Timestamps, Enums, JSONB, Arrays
│   │   ├── joins_aggregates.rs         # Joins, ROLLUP, FILTER aggregates
│   │   ├── advanced_sql.rs             # Recursive CTEs, Windows, LATERAL, DISTINCT ON
│   │   ├── powerful_dml.rs             # Upsert (EXCLUDED), RETURNING, UNNEST, COPY
│   │   ├── database_logic.rs           # Views, Materialized Views, Triggers, LISTEN/NOTIFY
│   │   ├── transactions_locking.rs     # Savepoints, FOR UPDATE, Deadlocks, Optimistic CAS
│   │   ├── repository_pattern.rs       # PgExecutor trait, SQLSTATE 23505/23503/23514 mapping
│   │   └── security_rls.rs             # Row-Level Security with SET LOCAL app.tenant_id
│   ├── part2/                          # Part 2: 100 Crore Stress-Test Challenges
│   │   ├── mod.rs
│   │   ├── lab_config.rs               # Hardware specs, SHOW settings, EXPLAIN parser
│   │   ├── ingestion_race.rs           # 4-way ingestion benchmark suite
│   │   ├── challenges/
│   │   │   ├── c01_lookup.rs           # Challenge 1: Find Username
│   │   │   ├── c02_availability.rs     # Challenge 2: Signup Availability Check
│   │   │   ├── c03_autocomplete.rs     # Challenge 3: Trigram GIN Autocomplete
│   │   │   ├── c04_counts.rs           # Challenge 4: Dashboard Counts
│   │   │   ├── c05_pagination.rs       # Challenge 5: Deep Pagination
│   │   │   ├── c06_export.rs           # Challenge 6: Massive Dataset Export
│   │   │   ├── c07_timeseries.rs       # Challenge 7: Time-Series Events
│   │   │   ├── c08_top_n.rs            # Challenge 8: Top-N Per User
│   │   │   ├── c09_duplicates.rs       # Challenge 9: Duplicate Purge
│   │   │   ├── c10_migrations.rs       # Challenge 10: Zero-Downtime Live Migrations
│   │   │   ├── c11_deletion.rs         # Challenge 11: Large Deletion (Partition Drop)
│   │   │   ├── c12_counters.rs         # Challenge 12: Hot Counter Contention
│   │   │   └── c13_queues.rs           # Challenge 13: FOR UPDATE SKIP LOCKED Queues
│   │   ├── production_scale.rs         # Pool sizing math, pg_stat_statements, autovacuum
│   │   └── capstone_service.rs         # Production Axum API service
│   └── bin/
│       └── export_benchmark_data.rs    # Remotion Visuals benchmark data exporter
├── tests/
│   ├── part1_integration_test.rs       # Part 1 end-to-end integration tests
│   └── part2_challenges_test.rs        # Part 2 challenges integration & Axum test suite
└── public/
    └── data/                           # Exported Remotion benchmark JSON contracts
        ├── lab-settings.json
        ├── headline.json
        ├── load-benchmarks.json
        ├── table-size.json
        ├── plans/                      # Real EXPLAIN JSON execution plans
        └── results/                    # Comparison result JSON contracts
```

---

## ⚡ Quick Start

### 1. Configure Database Connection

Create or verify `.env`:
```bash
DATABASE_URL="postgres://postgres:Darshan7069@localhost:5432/mastery_db"
```

### 2. Run Database Migrations

Apply all schema, trigger, partition, and extension migrations:
```bash
sqlx database create
sqlx migrate run
```

### 3. Run the Automated Test Suite

```bash
cargo test -- --nocapture
```
*Executes all 12 tests across Part 1 (CRUD, Macros, Types, RLS, Repo) and Part 2 (Ingestion, 13 Challenges, Axum service) with 100% pass rate.*

### 4. Run the Full Masterclass CLI Demo

Run both Part 1 and Part 2 live demonstrations:
```bash
cargo run -- all
```

Or run individual parts:
```bash
cargo run -- part1     # Runs Part 1 Zero-to-Advanced demos
cargo run -- part2     # Runs Part 2 100 Crore stress-test demos
cargo run -- server    # Starts production Axum REST API on http://127.0.0.1:3000
```

### 5. Export Remotion Benchmark JSON Contracts

Generate the visual data contracts for Remotion animations into `public/data/`:
```bash
cargo run --bin export_benchmark_data
```

### 6. Verify Compile-Time Offline Mode

```bash
cargo sqlx prepare
cargo clippy --all-targets -- -D warnings
```

---

## 📊 Benchmark Summary (100 Crore Rows Stress-Test)

| Challenge | Naive Approach | Winning Architecture | Speedup / Advantage |
| :--- | :--- | :--- | :--- |
| **C01: Find Username** | 164,210 ms (Seq Scan) | 0.42 ms (B-Tree / Covering Index) | **390,976x faster** |
| **C02: Signup Check** | 128,400 ms (`count(*)`) | 0.00012 ms (In-Memory Bloom Filter) | **1,070,000,000x faster** |
| **C03: Autocomplete** | 180,000 ms (`LIKE '%abc%'`) | 12.4 ms (`pg_trgm` GIN index) | **14,516x faster** |
| **C04: Dashboard Count** | 138,000 ms (`count(*)`) | 0.12 ms (`pg_class.reltuples` / Trigger) | **1,150,000x faster** |
| **C05: Deep Pagination** | 6,800 ms (`OFFSET 500k`) | 0.42 ms (Keyset Cursor `WHERE id > $id`) | **16,190x faster** |
| **C06: Massive Export** | OOM Crash (`fetch_all`) | 0.85 ms/chunk (Tokio Stream / COPY STDOUT) | **Zero RAM bloat** |
| **C07: Time-Series** | 25.4 GB (`created_at` B-Tree) | 12.4 MB (BRIN / Range Partitioning) | **2,048x smaller index** |
| **C08: Top-N Per User** | 48,900 ms (Correlated Subquery) | 18.4 ms (`LATERAL` join) | **2,657x faster** |
| **C09: Duplicate Purge** | Temp Disk Spill (`work_mem = 4MB`) | Fast In-Memory (`work_mem = 64MB`) | **85x faster** |
| **C10: Live Migrations** | Production Freeze (`SHARE` Lock) | Zero-Downtime (`CONCURRENTLY` + lock_timeout) | **Zero write blocking** |
| **C11: Large Deletion** | 45,200 ms (Bulk `DELETE` WAL bloat) | 0.08 ms (Declarative Partition Drop) | **565,000x faster** |
| **C12: Hot Counters** | 420 RPS (Exclusive Row Lock queue) | 28,400 RPS (Tokio mpsc buffer channel) | **67.6x throughput** |
| **C13: Job Queues** | Worker Lock Collisions / Deadlocks | 4,800 jobs/s (`FOR UPDATE SKIP LOCKED`) | **Zero contention** |
