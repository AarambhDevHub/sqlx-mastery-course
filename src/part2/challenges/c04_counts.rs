use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Instant;

/// Benchmark result for Challenge 4: The Dashboard Count Nightmare
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountsBenchmarkResult {
    pub count_star_ms: f64,
    pub reltuples_estimate_ms: f64,
    pub trigger_counter_ms: f64,
    pub actual_count: i64,
    pub estimated_count: i64,
    pub counter_table_count: i64,
}

/// Challenge 4: Dashboard Counts in PostgreSQL
///
/// Demonstrates:
/// 1. Naive `count(*)`: Scans every page in the table to verify MVCC tuple visibility.
///    On 1B rows, this takes 2-5 minutes of continuous I/O!
/// 2. Fast catalog estimate: Reads statistics from `pg_class.reltuples` in < 0.2 ms.
/// 3. Trigger-maintained counter table: Exact O(1) reads maintained by row-level triggers.
pub async fn run_challenge_04(
    pool: &PgPool,
) -> Result<CountsBenchmarkResult, sqlx::Error> {
    // 1. Naive count(*)
    let start = Instant::now();
    let actual_count: i64 = sqlx::query_scalar("SELECT count(*) FROM lab_users")
        .fetch_one(pool)
        .await?;
    let count_star_ms = start.elapsed().as_secs_f64() * 1000.0;

    // 2. pg_class reltuples estimate
    let start = Instant::now();
    let estimated_count: i64 = sqlx::query_scalar(
        "SELECT reltuples::bigint FROM pg_class WHERE relname = 'lab_users'"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(actual_count);
    let reltuples_estimate_ms = start.elapsed().as_secs_f64() * 1000.0;

    // 3. Trigger-maintained single-row counter table
    let start = Instant::now();
    let counter_table_count: i64 = sqlx::query_scalar(
        "SELECT user_count FROM counter_aggregates WHERE id = 1"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(actual_count);
    let trigger_counter_ms = start.elapsed().as_secs_f64() * 1000.0;

    Ok(CountsBenchmarkResult {
        count_star_ms,
        reltuples_estimate_ms,
        trigger_counter_ms,
        actual_count,
        estimated_count,
        counter_table_count,
    })
}
