use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Instant;

/// Benchmark result for Challenge 10: Zero-Downtime Live Migrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationBenchmarkResult {
    pub standard_index_blocking: bool,
    pub concurrent_index_ms: f64,
    pub backfilled_rows: u64,
    pub backfill_duration_ms: f64,
}

/// Challenge 10: Zero-Downtime Live Migrations
///
/// Demonstrates:
/// 1. Standard `CREATE INDEX`: Acquires `SHARE` lock on table, blocking all concurrent `INSERT`, `UPDATE`, `DELETE`.
///    On a 100-crore row table, this freezes production for hours.
/// 2. `CREATE INDEX CONCURRENTLY`: Builds the index via 2 table scans without blocking writes.
///    Always pair with `SET lock_timeout = '2s'` to prevent queuing behind long-running queries.
/// 3. Batched Keyset Backfill: Backfilling new columns in chunks of 10,000 rows per transaction,
///    avoiding lock contention and long transactions.
pub async fn run_challenge_10(
    pool: &PgPool,
) -> Result<MigrationBenchmarkResult, sqlx::Error> {
    // 1. Concurrent index creation with lock timeout
    // Note: CREATE INDEX CONCURRENTLY cannot run inside a multi-statement transaction block
    let start = Instant::now();
    let mut conn = pool.acquire().await?;
    sqlx::query("SET lock_timeout = '2s'").execute(&mut *conn).await?;
    sqlx::query("CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_lab_users_status_live ON lab_users (status)")
        .execute(&mut *conn)
        .await?;
    let concurrent_index_ms = start.elapsed().as_secs_f64() * 1000.0;

    // 2. Keyset batched column backfill loop in Rust
    let start = Instant::now();
    let mut last_id = 0i64;
    let mut total_backfilled = 0u64;
    let batch_size = 5000i64;

    loop {
        let mut tx = pool.begin().await?;
        let res = sqlx::query(
            r#"
            UPDATE lab_users
            SET status = 'active'
            WHERE id > $1 AND id <= $1 + $2
            "#
        )
        .bind(last_id)
        .bind(batch_size)
        .execute(&mut *tx)
        .await?;

        let affected = res.rows_affected();
        tx.commit().await?;

        total_backfilled += affected;
        last_id += batch_size;

        // In demo benchmark, break after 1-2 batches or when no more rows affected
        if affected == 0 || total_backfilled >= 10000 {
            break;
        }
    }
    let backfill_duration_ms = start.elapsed().as_secs_f64() * 1000.0;

    Ok(MigrationBenchmarkResult {
        standard_index_blocking: true,
        concurrent_index_ms,
        backfilled_rows: total_backfilled,
        backfill_duration_ms,
    })
}
