use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Instant;

/// Benchmark result for Challenge 9: Detecting & Cleaning Duplicates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicatesBenchmarkResult {
    pub low_work_mem_ms: f64,
    pub tuned_work_mem_ms: f64,
    pub purged_duplicates_count: u64,
    pub purge_duration_ms: f64,
}

/// Challenge 9: Detecting & Cleaning Duplicates at Scale
///
/// Demonstrates:
/// 1. `GROUP BY email HAVING count(*) > 1`: Default `work_mem = 4MB` spills intermediate hash tables
///    to temporary disk files (catastrophic disk I/O thrashing).
/// 2. Tuning `SET LOCAL work_mem = '64MB'` keeps the entire hash aggregation in high-speed RAM.
/// 3. Safe batched deletion: Using `ctid` or keyset batching to purge duplicates in chunks without locking
///    the entire table.
pub async fn run_challenge_09(
    pool: &PgPool,
) -> Result<DuplicatesBenchmarkResult, sqlx::Error> {
    // 1. Group By with low work_mem (4MB)
    let start = Instant::now();
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL work_mem = '4MB'").execute(&mut *tx).await?;
    let _ = sqlx::query("SELECT email, count(*) FROM lab_users GROUP BY email HAVING count(*) > 1 LIMIT 5")
        .fetch_all(&mut *tx)
        .await?;
    tx.commit().await?;
    let low_work_mem_ms = start.elapsed().as_secs_f64() * 1000.0;

    // 2. Group By with tuned work_mem (64MB)
    let start = Instant::now();
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL work_mem = '64MB'").execute(&mut *tx).await?;
    let _ = sqlx::query("SELECT email, count(*) FROM lab_users GROUP BY email HAVING count(*) > 1 LIMIT 5")
        .fetch_all(&mut *tx)
        .await?;
    tx.commit().await?;
    let tuned_work_mem_ms = start.elapsed().as_secs_f64() * 1000.0;

    // 3. Batched duplicate purge using ctid
    let start = Instant::now();
    let res = sqlx::query(
        r#"
        DELETE FROM lab_users
        WHERE ctid IN (
            SELECT ctid
            FROM (
                SELECT ctid, ROW_NUMBER() OVER (PARTITION BY email ORDER BY id) as rn
                FROM lab_users
            ) t
            WHERE t.rn > 1
            LIMIT 1000
        )
        "#
    )
    .execute(pool)
    .await?;
    let purge_duration_ms = start.elapsed().as_secs_f64() * 1000.0;

    Ok(DuplicatesBenchmarkResult {
        low_work_mem_ms,
        tuned_work_mem_ms,
        purged_duplicates_count: res.rows_affected(),
        purge_duration_ms,
    })
}
