use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Instant;

/// Benchmark result for Challenge 1: Find One Username
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupBenchmarkResult {
    pub seq_scan_ms: f64,
    pub btree_index_ms: f64,
    pub case_insensitive_ms: f64,
    pub covering_index_ms: f64,
    pub brin_index_ms: f64,
    pub rows_scanned_seq: i64,
}

/// Challenge 1: Point lookup on a billion rows
///
/// Demonstrates the difference between:
/// 1. Sequential table scan (O(N) reads all pages from disk)
/// 2. B-Tree index scan (O(log N) ~ 3-4 buffer page hits)
/// 3. Expression index `lower(username)` for case-insensitivity
/// 4. Covering index with `INCLUDE (email, created_at)` for Index Only Scan
/// 5. BRIN index (Block Range Index), showing why BRIN is ineffective on unsorted data.
pub async fn run_challenge_01(
    pool: &PgPool,
    target_username: &str,
) -> Result<LookupBenchmarkResult, Box<dyn std::error::Error + Send + Sync>> {
    // 1. Sequential scan (force seq scan via SET LOCAL enable_indexscan = off)
    let start = Instant::now();
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL enable_indexscan = off").execute(&mut *tx).await?;
    sqlx::query("SET LOCAL enable_bitmapscan = off").execute(&mut *tx).await?;
    let row = sqlx::query("SELECT id, username, email FROM lab_users WHERE username = $1")
        .bind(target_username)
        .fetch_optional(&mut *tx)
        .await?;
    tx.commit().await?;
    let seq_scan_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Ensure B-tree index exists
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_lab_users_username ON lab_users (username)")
        .execute(pool)
        .await?;

    // 2. B-tree Index Scan
    let start = Instant::now();
    let _ = sqlx::query("SELECT id, username, email FROM lab_users WHERE username = $1")
        .bind(target_username)
        .fetch_optional(pool)
        .await?;
    let btree_index_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Ensure lower(username) expression index exists
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_lab_users_lower ON lab_users (lower(username))")
        .execute(pool)
        .await?;

    // 3. Case-Insensitive Index Scan
    let start = Instant::now();
    let _ = sqlx::query("SELECT id, username, email FROM lab_users WHERE lower(username) = lower($1)")
        .bind(target_username)
        .fetch_optional(pool)
        .await?;
    let case_insensitive_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Ensure Covering Index with INCLUDE exists
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_lab_users_covering ON lab_users (username) INCLUDE (email, created_at)")
        .execute(pool)
        .await?;

    // 4. Covering Index Only Scan
    let start = Instant::now();
    let _ = sqlx::query("SELECT username, email, created_at FROM lab_users WHERE username = $1")
        .bind(target_username)
        .fetch_optional(pool)
        .await?;
    let covering_index_ms = start.elapsed().as_secs_f64() * 1000.0;

    // 5. BRIN index demonstration
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_lab_users_brin ON lab_users USING BRIN (username)")
        .execute(pool)
        .await?;

    let start = Instant::now();
    let _ = sqlx::query("SELECT id, username FROM lab_users WHERE username = $1")
        .bind(target_username)
        .fetch_optional(pool)
        .await?;
    let brin_index_ms = start.elapsed().as_secs_f64() * 1000.0;

    let rows_scanned_seq = if row.is_some() { 1 } else { 0 };

    Ok(LookupBenchmarkResult {
        seq_scan_ms,
        btree_index_ms,
        case_insensitive_ms,
        covering_index_ms,
        brin_index_ms,
        rows_scanned_seq,
    })
}
