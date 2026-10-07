use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Instant;

/// Benchmark result for Challenge 5: Deep Pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationBenchmarkResult {
    pub offset_latency_ms: f64,
    pub keyset_latency_ms: f64,
    pub depth_offset: i64,
    pub page_size: usize,
}

/// Challenge 5: Deep Pagination
///
/// Demonstrates:
/// 1. Naive `LIMIT 20 OFFSET 100000`: PostgreSQL must read 100,020 index tuples from disk
///    and throw away the first 100,000 rows. Memory and execution time degrade linearly with depth.
/// 2. Keyset (Cursor-based) pagination: `WHERE id > $last_seen_id ORDER BY id LIMIT 20`.
///    B-Tree seeks directly to the first target key, giving O(1) constant sub-millisecond
///    latency whether on Page 1 or Page 500,000.
pub async fn run_challenge_05(
    pool: &PgPool,
    depth_offset: i64,
    limit: i64,
) -> Result<PaginationBenchmarkResult, sqlx::Error> {
    // 1. Naive OFFSET pagination
    let start = Instant::now();
    let _offset_rows = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM lab_users ORDER BY id ASC LIMIT $1 OFFSET $2"
    )
    .bind(limit)
    .bind(depth_offset)
    .fetch_all(pool)
    .await?;
    let offset_latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    // 2. Keyset Cursor pagination (seek by last seen id)
    let start = Instant::now();
    let keyset_rows = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM lab_users WHERE id > $1 ORDER BY id ASC LIMIT $2"
    )
    .bind(depth_offset)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    let keyset_latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    Ok(PaginationBenchmarkResult {
        offset_latency_ms,
        keyset_latency_ms,
        depth_offset,
        page_size: keyset_rows.len(),
    })
}
