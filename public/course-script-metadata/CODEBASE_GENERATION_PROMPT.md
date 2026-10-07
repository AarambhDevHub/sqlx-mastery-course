# 🚀 MASTER AGENT PROMPT: Build, Test, and Verify the Complete SQLx + PostgreSQL Master Codebase (Part 1 & Part 2)

> **Context for the AI Agent:**  
> You are tasked with generating, testing, and verifying the complete, production-ready Rust codebase for **Aarambh Dev Hub's SQLx + PostgreSQL Master Course**.  
> This course is split into two 3-hour masterclass videos:
> 1. **Part 1:** *SQLx + PostgreSQL in Rust: Complete Course from Zero to Advanced (SQLx 0.9)*
> 2. **Part 2:** *100 Crore Rows in PostgreSQL: 13 Real Challenges with SQLx (Find One User in Milliseconds)*
>
> You must build this as a **single, unified, production-grade Cargo workspace / project** containing 100% working, idiomatic, fully tested Rust code. Every feature, query, migration, challenge, and benchmark described below must compile, pass tests against a real PostgreSQL instance, and output the required benchmark data.

---

## 📋 Table of Contents
1. [Environment & Tech Stack Specifications](#1-environment--tech-stack-specifications)
2. [Project Architecture & Directory Layout](#2-project-architecture--directory-layout)
3. [Part 1 Implementation Requirements (Zero to Advanced)](#3-part-1-implementation-requirements-zero-to-advanced)
4. [Part 2 Implementation Requirements (100 Crore Rows Stress-Test)](#4-part-2-implementation-requirements-100-crore-rows-stress-test)
5. [Benchmark JSON Data Contracts (For Remotion Visuals)](#5-benchmark-json-data-contracts-for-remotion-visuals)
6. [Step-by-Step Build, Test, and Verification Checklist](#6-step-by-step-build-test-and-verification-checklist)

---

## 1. Environment & Tech Stack Specifications

- **Language & MSRV:** Rust 1.94+ (2024 edition or 2021 edition compatible with SQLx 0.9).
- **Core Library:** `sqlx` version `0.9` (pure Rust asynchronous SQL toolkit).
  - Features required: `postgres`, `runtime-tokio-rustls`, `macros`, `uuid`, `chrono`, `rust_decimal`, `json`.
- **Runtime:** `tokio` (full features).
- **Web Framework (for Part 2 Capstone):** `axum` (0.7 / 0.8), `tower`, `tower-http`.
- **Utility Crates:**
  - `uuid` (v4 and v7 support, serde)
  - `chrono` (serde)
  - `rust_decimal` (serde-postgres)
  - `serde`, `serde_json`
  - `dotenvy`
  - `tracing`, `tracing-subscriber`
  - `futures` (for Streams)
  - `fastbloom` or equivalent lightweight in-memory Bloom filter
- **Database Engine:** PostgreSQL 16+ running via Docker.
- **CLI Tooling:** `sqlx-cli` (installed without `--locked`).

---

## 2. Project Architecture & Directory Layout

Build the codebase with the following layout:

```text
sqlx-mastery-course/
├── Cargo.toml                          # Unified Cargo workspace or multi-module package
├── docker-compose.yml                  # PostgreSQL 16 container with tuned parameters
├── sqlx.toml                           # SQLx 0.9 configuration file
├── .env.example                        # DATABASE_URL=postgres://postgres:postgres@localhost:5432/mastery_db
├── migrations/                         # Reversible migrations for Part 1 & Part 2
│   ├── 20260501000001_create_users_and_orders.up.sql
│   ├── 20260501000001_create_users_and_orders.down.sql
│   ├── 20260501000002_create_database_logic.up.sql
│   ├── 20260501000002_create_database_logic.down.sql
│   ├── 20260501000003_setup_row_level_security.up.sql
│   ├── 20260501000003_setup_row_level_security.down.sql
│   ├── 20260502000001_setup_crore_lab_tables.up.sql
│   └── 20260502000001_setup_crore_lab_tables.down.sql
├── src/
│   ├── lib.rs                          # Library root exporting part1 and part2 modules
│   ├── main.rs                         # CLI runner for demos, migrations, and benchmarks
│   ├── pool.rs                         # PgPoolOptions initialization, healthchecks, settings
│   ├── part1/                          # PART 1: ZERO TO ADVANCED MODULES
│   │   ├── mod.rs
│   │   ├── schema.rs                   # Struct models (User, Order, Profile, UserRole)
│   │   ├── queries_basics.rs           # query, query_as, query_scalar, FromRow, fetch variants
│   │   ├── queries_macros.rs           # query!, query_as!, offline verification
│   │   ├── queries_dynamic.rs          # QueryBuilder, AssertSqlSafe usage in SQLx 0.9
│   │   ├── types_custom.rs             # UUIDv7, Decimal, Timestamps, Enums, JSONB, Arrays
│   │   ├── joins_aggregates.rs         # Joins, subqueries, EXISTS, FILTER, ROLLUP
│   │   ├── advanced_sql.rs             # CTEs, recursive CTEs, window functions, LATERAL, DISTINCT ON
│   │   ├── powerful_dml.rs             # ON CONFLICT DO UPDATE (EXCLUDED), RETURNING, UNNEST, COPY
│   │   ├── database_logic.rs           # Views, Materialized Views concurrent refresh, Triggers, PgListener
│   │   ├── transactions_locking.rs     # Transactions, drop-rollback, savepoints, isolation levels, FOR UPDATE, deadlocks, optimistic CAS
│   │   ├── repository_pattern.rs       # Executor trait (PgExecutor), error mapping (23505, 23503, 23514)
│   │   └── security_rls.rs             # Row-Level Security, tenant switching with SET LOCAL
│   ├── part2/                          # PART 2: 100 CRORE ROWS CHALLENGES
│   │   ├── mod.rs
│   │   ├── lab_config.rs               # Hardware specs, SHOW settings, EXPLAIN ANALYZE parser
│   │   ├── ingestion_race.rs           # 4-way ingestion: Single, QueryBuilder, UNNEST, Parallel COPY
│   │   ├── challenges/
│   │   │   ├── c01_lookup.rs           # Seq Scan vs B-Tree vs lower() vs Covering vs BRIN
│   │   │   ├── c02_availability.rs     # Index check vs Cache vs Bloom Filter
│   │   │   ├── c03_autocomplete.rs     # text_pattern_ops prefix vs pg_trgm GIN fuzzy search
│   │   │   ├── c04_counts.rs           # count(*) vs pg_class.reltuples vs Trigger counter table
│   │   │   ├── c05_pagination.rs       # OFFSET vs Keyset Cursor (WHERE id > cursor)
│   │   │   ├── c06_export.rs           # fetch_all OOM vs Tokio Stream vs COPY TO STDOUT
│   │   │   ├── c07_timeseries.rs       # B-Tree vs BRIN vs Declarative Range Partitioning
│   │   │   ├── c08_top_n.rs            # Correlated subqueries vs Windows vs DISTINCT ON vs LATERAL
│   │   │   ├── c09_duplicates.rs       # work_mem tuning (preventing disk spill) & batched ctid purge
│   │   │   ├── c10_migrations.rs       # CREATE INDEX CONCURRENTLY, lock_timeout, batched keyset backfill
│   │   │   ├── c11_deletion.rs         # Bulk DELETE vs Batched vs Instant Partition Drop
│   │   │   ├── c12_counters.rs         # Row lock contention vs in-memory channel vs Sharded counters
│   │   │   └── c13_queues.rs           # Polling collisions vs FOR UPDATE SKIP LOCKED workers
│   │   ├── production_scale.rs         # Pool sizing math, pg_stat_statements, auto_explain, autovacuum
│   │   └── capstone_service.rs         # Production Axum API service integrating winning solutions
│   └── bin/
│       └── export_benchmark_data.rs    # Tool to run benchmarks and output public/data JSON files
├── tests/
│   ├── part1_integration_test.rs       # #[sqlx::test] tests for Part 1 features
│   └── part2_challenges_test.rs        # Automated tests verifying all 13 challenge behaviors
└── public/
    └── data/                           # JSON benchmark files generated for Remotion
        ├── lab-settings.json
        ├── headline.json
        ├── load-benchmarks.json
        ├── table-size.json
        ├── plans/                      # Real EXPLAIN JSON output files
        └── results/                    # Result JSON files for every challenge
```

---

## 3. Part 1 Implementation Requirements (Zero to Advanced)

### 3.1 Connection & Pool Initialization (`pool.rs`)
- Implement `init_pool(database_url: &str) -> Result<PgPool, sqlx::Error>`.
- Configure `PgPoolOptions`:
  - `max_connections(20)`
  - `min_connections(5)`
  - `acquire_timeout(Duration::from_secs(3))`
  - `idle_timeout(Duration::from_secs(600))`
  - `max_lifetime(Duration::from_secs(1800))`
- Implement a healthcheck ping function that runs `SELECT 1` on startup.

### 3.2 Schema Design & Constraints (`migrations/`)
- Migration 1 (`create_users_and_orders.up.sql`):
  - `users` table:
    - `id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY`
    - `username TEXT NOT NULL UNIQUE`
    - `email TEXT NOT NULL UNIQUE`
    - `country TEXT NOT NULL DEFAULT 'IN'`
    - `status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'pending'))`
    - `profile JSONB NOT NULL DEFAULT '{}'::jsonb`
    - `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
    - `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`
  - `orders` table:
    - `id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY`
    - `user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE`
    - `amount NUMERIC(12, 2) NOT NULL CHECK (amount >= 0)`
    - `status TEXT NOT NULL DEFAULT 'completed' CHECK (status IN ('pending', 'completed', 'refunded'))`
    - `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
  - Foreign key index: `CREATE INDEX idx_orders_user_id ON orders (user_id);`
- Matching down migration dropping `orders` then `users`.

### 3.3 Core Query API (`queries_basics.rs`)
- Models: `User` struct deriving `Debug`, `FromRow`, `Serialize`, `Deserialize`.
- Implement functions demonstrating:
  - `sqlx::query` extracting raw `PgRow` values manually via `row.get("username")`.
  - `sqlx::query_as::<_, User>` using `$1` positional parameter binding.
  - `sqlx::query_scalar::<_, i64>` to query single scalar values (e.g. `SELECT count(*) FROM users`).
  - `fetch_one`: fetches exactly 1 row; returns error if 0 or >1 rows.
  - `fetch_optional`: returns `Option<User>`.
  - `fetch_all`: returns `Vec<User>`.
  - `fetch`: returns a Tokio stream of `Result<User, sqlx::Error>`.
  - Demonstrate `FromRow` attributes:
    - `#[sqlx(rename = "user_name")]`
    - `#[sqlx(flatten)]` for embedded child structs (e.g. `UserMetadata`)
    - `#[sqlx(skip)]` for computed fields
    - `#[sqlx(default)]` for optional fallback values

### 3.4 Macros, Offline Mode, & Dynamic SQL (`queries_macros.rs`, `queries_dynamic.rs`)
- Implement `query!` and `query_as!` demonstrations:
  - Compile-time type check on `$1` and returned columns.
  - Document `cargo sqlx prepare` instructions to generate the `.sqlx` cache.
- Implement dynamic SQL using `QueryBuilder<Postgres>`:
  - Dynamic filter builder that conditionally appends clauses:
    - `.push(" AND country = ").push_bind(country)`
    - `.push(" AND created_at >= ").push_bind(from_date)`
  - Use `AssertSqlSafe` (SQLx 0.9) to safely insert dynamic sort column identifiers (`ORDER BY created_at DESC`).

### 3.5 Advanced Postgres Types (`types_custom.rs`)
- Implement conversions for:
  - `uuid::Uuid` (support UUID v7 generation using `Uuid::now_v7()`).
  - `rust_decimal::Decimal` mapped to `NUMERIC(12, 2)` (demonstrating exact math without `f64` rounding bugs).
  - `chrono::DateTime<Utc>` mapped to `TIMESTAMPTZ`.
  - Custom Postgres Enum:
    - Database enum: `CREATE TYPE user_role AS ENUM ('admin', 'moderator', 'member');`
    - Rust enum: `#[derive(Debug, PartialEq, sqlx::Type)] #[sqlx(type_name = "user_role", rename_all = "snake_case")] pub enum UserRole { Admin, Moderator, Member }`
  - `sqlx::types::Json<UserProfile>` mapped to `JSONB` column.
  - Native Postgres Arrays: `Vec<String>` mapped to `TEXT[]` with containment queries (`@>`).

### 3.6 Joins, Aggregates & Advanced SQL (`joins_aggregates.rs`, `advanced_sql.rs`)
- Joins:
  - `INNER JOIN`, `LEFT JOIN` (mapping right-side columns to `Option<Order>` in Rust structs).
  - Self-join: Employee-manager hierarchy query.
- Aggregates:
  - `COUNT(*) FILTER (WHERE status = 'completed')` conditional counting.
  - `ROLLUP (country, status)` and `GROUPING SETS`.
- Advanced SQL:
  - CTE with `WITH active_users AS (...)`.
  - Recursive CTE: traversing an organizational hierarchy tree returning `(id, name, path, depth)`.
  - Window functions:
    - `ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY created_at DESC)`
    - `SUM(amount) OVER (PARTITION BY user_id ORDER BY created_at)` running cumulative total.
    - `LEAD` / `LAG` for delta calculation.
  - `LATERAL` join: fetching the top 3 latest orders for every user.
  - `DISTINCT ON (user_id)`: extracting the single latest order per user.

### 3.7 Powerful DML & Database Logic (`powerful_dml.rs`, `database_logic.rs`)
- Upsert:
  - `INSERT INTO users (...) VALUES (...) ON CONFLICT (username) DO UPDATE SET updated_at = now() RETURNING id;` using `EXCLUDED`.
- `RETURNING`: Inserting and updating while returning modified records in 1 round trip.
- Bulk `UNNEST`:
  - `INSERT INTO users (username, email, country) SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[])` (using only 3 bind parameters for thousands of rows).
- `copy_in_raw`: Streaming CSV bytes straight into Postgres.
- Database Logic:
  - View creation and query.
  - Materialized View: `CREATE MATERIALIZED VIEW user_order_summaries AS ...` with a unique index, and a Rust function executing `REFRESH MATERIALIZED VIEW CONCURRENTLY user_order_summaries`.
  - Generated column: `total_cents BIGINT GENERATED ALWAYS AS (amount * 100) STORED`.
  - Trigger: `BEFORE UPDATE` trigger function executing `NEW.updated_at = now()`.
  - `LISTEN / NOTIFY`: Trigger firing `pg_notify('order_events', payload)` and a Rust async task listening via `PgListener`.

### 3.8 Transactions, Concurrency, & Locking (`transactions_locking.rs`)
- Basic transaction lifecycle: `pool.begin()`, `tx.commit()`, and automatic rollback on `drop`.
- Savepoints: Calling `tx.begin()` to create a `SAVEPOINT`, rolling back the savepoint without aborting the parent transaction.
- Isolation levels: Setting `BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE`.
- Row-level lock: `SELECT amount FROM accounts WHERE id = $1 FOR UPDATE`.
- Deadlock simulation: Two concurrent tasks locking rows in opposite order, catching the `40P01` deadlock detected error.
- Optimistic locking: `UPDATE accounts SET balance = $1, version = version + 1 WHERE id = $2 AND version = $3` returning rows affected.

### 3.9 Repository Pattern & Error Mapping (`repository_pattern.rs`)
- Define `UserRepository`:
  - Functions take `E: PgExecutor<'e>` so they can be called with `&PgPool` OR `&mut *tx`.
  - Signature example: `pub async fn find_by_id<'e, E>(executor: E, id: i64) -> Result<Option<User>, UserRepoError> where E: PgExecutor<'e>`.
- Custom Error Enum `UserRepoError`:
  - Inspect `sqlx::Error::Database`.
  - Match on PostgreSQL SQLSTATE error code:
    - `23505` (unique_violation) -> `UserRepoError::UsernameAlreadyTaken(String)`
    - `23503` (foreign_key_violation) -> `UserRepoError::NotFound`
    - `23514` (check_violation) -> `UserRepoError::InvalidData(String)`

### 3.10 Security & Testing (`security_rls.rs`, `tests/`)
- Row-Level Security:
  - `ALTER TABLE documents ENABLE ROW LEVEL SECURITY;`
  - Policy: `CREATE POLICY tenant_isolation ON documents USING (tenant_id = current_setting('app.tenant_id')::uuid);`
  - In Rust: acquire connection, run `SET LOCAL app.tenant_id = $1`, query `documents` verifying cross-tenant isolation.
- Integration tests in `tests/part1_integration_test.rs`:
  - Annotate test functions with `#[sqlx::test(migrations = "./migrations")]`.
  - Verify that tests run in parallel against isolated ephemeral databases.

---

## 4. Part 2 Implementation Requirements (100 Crore Rows Stress-Test)

### 4.1 Ingestion Race (`ingestion_race.rs`)
Implement a benchmark suite comparing the 4 ingestion strategies:
1. `SingleInsert`: 1 row per query.
2. `QueryBuilderBatch`: 20,000 rows per insert (verifying the 65,535 parameter limit).
3. `UnnestBatch`: Vector arrays unnested (3 parameters).
4. `CopyLoader`: Multi-threaded parallel workers using `sqlx::copy_in_raw`.
- Record and log: throughput (rows/sec), total duration, memory usage.
- Rule: Drop indexes before load, build indexes after load.

### 4.2 The 13 Challenges (`src/part2/challenges/`)

#### Challenge 1: Find One Username (`c01_lookup.rs`)
- Compare across table of 100 crore rows (or scaled test dataset of 1M - 10M rows):
  - Naive: `SELECT * FROM users WHERE username = $1` (Sequential scan).
  - Solution 1: `CREATE UNIQUE INDEX idx_users_username ON users (username)` (B-Tree index scan).
  - Solution 2: Case-insensitive query `lower(username) = 'alice'` with `CREATE INDEX ON users (lower(username))` vs `citext`.
  - Solution 3: Covering index `CREATE UNIQUE INDEX ... INCLUDE (email, created_at)` (`Index Only Scan`).
  - Solution 4: Hash index vs B-Tree.
  - Solution 5: Partial index `CREATE INDEX ... WHERE status = 'active'`.
  - Solution 6: BRIN index on username (demonstrating why BRIN fails on random data).

#### Challenge 2: Signup Availability Check (`c02_availability.rs`)
- Compare:
  - Naive `SELECT count(*) FROM users WHERE username = $1`.
  - Solution 1: `SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)`.
  - Solution 2: In-memory Bloom filter in Rust testing availability before querying PostgreSQL (0 DB queries on clean names).

#### Challenge 3: Autocomplete & Fuzzy Search (`c03_autocomplete.rs`)
- Compare:
  - Naive `WHERE username LIKE '%alice%'` (forces sequential scan).
  - Solution 1: Prefix search `LIKE 'ali%'` with `CREATE INDEX ON users (username text_pattern_ops)`.
  - Solution 2: Fuzzy search with `CREATE EXTENSION pg_trgm; CREATE INDEX ... USING GIN (username gin_trgm_ops)`.

#### Challenge 4: The Dashboard Count Nightmare (`c04_counts.rs`)
- Compare:
  - Naive `SELECT count(*) FROM users` (scans all pages in MVCC).
  - Solution 1: Fast catalog estimate `SELECT reltuples::bigint FROM pg_class WHERE relname = 'users'`.
  - Solution 2: Trigger-maintained single-row counter table `counter_aggregates`.

#### Challenge 5: Deep Pagination (`c05_pagination.rs`)
- Compare:
  - Naive `ORDER BY id LIMIT 20 OFFSET 10000000` (reads and discards 10M rows).
  - Solution: Keyset pagination `WHERE id > $last_seen_id ORDER BY id LIMIT 20` (constant 0.4 ms at any depth).

#### Challenge 6: Exporting Massive Datasets (`c06_export.rs`)
- Compare:
  - Naive `fetch_all` (allocates entire dataset in memory -> OOM crash).
  - Solution 1: Tokio streaming `fetch` writing to file with bounded 10,000-row chunks.
  - Solution 2: Native `COPY users TO STDOUT (FORMAT CSV)` streaming straight to file/socket.

#### Challenge 7: Last 24 Hours of Events (`c07_timeseries.rs`)
- Compare:
  - Naive B-Tree on `created_at` (bloated index, heavy write amplification).
  - Solution 1: BRIN index on monotonically increasing `created_at` (tiny 12MB footprint).
  - Solution 2: Declarative Range Partitioning by day/month with partition pruning in `EXPLAIN`.

#### Challenge 8: Top-N and Latest Record Per User (`c08_top_n.rs`)
- Compare:
  - Correlated subqueries in `WHERE` clause.
  - Window functions: `ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY created_at DESC) WHERE rn <= 3`.
  - `DISTINCT ON (user_id)` for single latest record.
  - `LATERAL` join with composite index `(user_id, created_at DESC)`.

#### Challenge 9: Detecting & Cleaning Duplicates (`c09_duplicates.rs`)
- Compare:
  - `GROUP BY email HAVING count(*) > 1` with default `work_mem = 4MB` (spilling temp files to disk) vs `SET work_mem = '2GB'`.
  - Safe batched deletion using `ctid` or primary keys in chunks of 10,000 rows.

#### Challenge 10: Zero-Downtime Live Migrations (`c10_migrations.rs`)
- Compare:
  - Standard `CREATE INDEX` (acquires `SHARE` lock, blocking all writes).
  - Solution: `CREATE INDEX CONCURRENTLY` combined with `SET lock_timeout = '2s'`.
  - Keyset batched column backfill loop in Rust updating 20,000 rows per transaction.

#### Challenge 11: Deleting Old Data at Scale (`c11_deletion.rs`)
- Compare:
  - Bulk `DELETE FROM events WHERE created_at < cutoff` (table bloat, WAL thrashing).
  - Batched chunk delete (`DELETE ... WHERE id IN (SELECT id ... LIMIT 50000)`).
  - Instantaneous partition drop: `ALTER TABLE events DETACH PARTITION ...; DROP TABLE ...` (0.08 ms, zero I/O).

#### Challenge 12: The Hot Counter Contention Problem (`c12_counters.rs`)
- Compare:
  - Naive `UPDATE posts SET views = views + 1 WHERE id = $1` under 10k RPS (row lock contention).
  - Solution 1: In-memory write buffer channel in Tokio flushing batched increments every 100 ms.
  - Solution 2: Sharded counters (16 rows in `counter_shards`, summing on read).

#### Challenge 13: High-Throughput Job Queues (`c13_queues.rs`)
- Compare:
  - Naive worker polling `SELECT id FROM jobs WHERE status = 'pending' LIMIT 1` (collisions and deadlocks).
  - Solution: `SELECT id, payload FROM jobs WHERE status = 'pending' FOR UPDATE SKIP LOCKED LIMIT 1`.

### 4.3 Capstone Axum Service (`capstone_service.rs`)
Implement an Axum REST service bringing all winners together:
- Route `GET /users/search?q=prefix` using `text_pattern_ops`.
- Route `GET /users/check?username=xyz` using Bloom filter + `EXISTS`.
- Route `GET /users?cursor=123&limit=20` using keyset pagination.
- Route `POST /posts/:id/view` buffering view counts via Tokio mpsc channel.
- Background worker task consuming jobs via `SELECT ... FOR UPDATE SKIP LOCKED`.

---

## 5. Benchmark JSON Data Contracts (For Remotion Visuals)

Implement `src/bin/export_benchmark_data.rs`. When executed (`cargo run --bin export_benchmark_data`), it must run the benchmarks (or a verified local dataset) and export the following JSON files to `public/data/`:

1. `public/data/lab-settings.json`:
   ```json
   {
     "hardware": "16 vCPUs (AMD EPYC), 64 GB RAM, Enterprise NVMe SSD",
     "shared_buffers": "16GB",
     "work_mem": "64MB",
     "maintenance_work_mem": "4GB",
     "effective_cache_size": "48GB",
     "max_parallel_workers_per_gather": 4
   }
   ```
2. `public/data/headline.json`:
   ```json
   {
     "rows": 1000000000,
     "scan_ms": 164210,
     "index_ms": 0.42
   }
   ```
3. `public/data/load-benchmarks.json`:
   ```json
   [
     { "method": "Single Inserts", "rows_per_sec": 620, "binds_per_statement": 3 },
     { "method": "QueryBuilder Batch (20k)", "rows_per_sec": 85000, "binds_per_statement": 60000 },
     { "method": "Array UNNEST", "rows_per_sec": 110000, "binds_per_statement": 3 },
     { "method": "Parallel COPY (8 workers)", "rows_per_sec": 1240000, "binds_per_statement": 0 }
   ]
   ```
4. `public/data/plans/c01_seq_scan.json` and `c01_index_scan.json`: Valid PostgreSQL `EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)` plan objects.
5. `public/data/results/master_summary.json`: Complete comparison matrix of all 13 challenges with naive vs optimized latencies and buffer counts.

---

## 6. Step-by-Step Build, Test, and Verification Checklist

When you run this project, complete the following verification steps:

1. **Docker Environment:**
   ```bash
   docker compose up -d
   ```
   Verify PostgreSQL is healthy on `localhost:5432`.
2. **Migrations:**
   ```bash
   cargo install sqlx-cli --no-default-features --features rustls,postgres
   sqlx database create
   sqlx migrate run
   ```
3. **Compile-Time Checks & Offline Cache:**
   ```bash
   cargo check
   cargo sqlx prepare
   ```
4. **Automated Test Suite:**
   ```bash
   cargo test -- --nocapture
   ```
   Ensure 100% of integration tests pass (Part 1 repository & types, Part 2 challenge behaviors).
5. **Export Remotion Benchmark Data:**
   ```bash
   cargo run --bin export_benchmark_data
   ```
   Verify that all JSON files in `public/data/` are created and valid.
6. **Code Quality:**
   - Zero compiler warnings (`cargo clippy --all-targets -- -D warnings`).
   - Clean, idiomatic code with thorough module documentation.

---
*Once all code is written, tested against PostgreSQL, and verified, package the repository and deliver it for direct use in the Aarambh Dev Hub production pipeline!*
