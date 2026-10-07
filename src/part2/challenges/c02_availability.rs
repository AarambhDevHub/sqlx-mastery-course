use fastbloom::BloomFilter;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Instant;

/// Benchmark metrics for Challenge 2: Username Availability Check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityBenchmarkResult {
    pub count_star_ms: f64,
    pub exists_query_ms: f64,
    pub bloom_filter_nanos: u64,
    pub db_queries_avoided: bool,
}

/// Challenge 2: Username Availability Check
///
/// Demonstrates:
/// 1. Naive `SELECT count(*) FROM users WHERE username = $1`: Scans matching entries, slower in MVCC.
/// 2. Optimized `SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)`: Halts immediately upon finding the first match.
/// 3. In-memory Bloom filter: Inverted availability check.
///    If `bloom.contains(username)` is false, username is guaranteed 100% available without touching PostgreSQL!
pub async fn run_challenge_02(
    pool: &PgPool,
    test_username: &str,
    existing_usernames: &[String],
) -> Result<AvailabilityBenchmarkResult, sqlx::Error> {
    // 1. Naive count(*) query
    let start = Instant::now();
    let _count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM lab_users WHERE username = $1"
    )
    .bind(test_username)
    .fetch_one(pool)
    .await?;
    let count_star_ms = start.elapsed().as_secs_f64() * 1000.0;

    // 2. EXISTS(SELECT 1) query
    let start = Instant::now();
    let _exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM lab_users WHERE username = $1)"
    )
    .bind(test_username)
    .fetch_one(pool)
    .await?;
    let exists_query_ms = start.elapsed().as_secs_f64() * 1000.0;

    // 3. In-memory Bloom Filter
    let mut bloom = BloomFilter::with_num_bits(65536).expected_items(existing_usernames.len().max(10));
    for u in existing_usernames {
        bloom.insert(u.as_str());
    }

    let start = Instant::now();
    let is_present_in_bloom = bloom.contains(test_username);
    let bloom_filter_nanos = start.elapsed().as_nanos() as u64;

    // If bloom filter returns false, the username is guaranteed not taken
    // -> 0 DB round trips!
    let db_queries_avoided = !is_present_in_bloom;

    Ok(AvailabilityBenchmarkResult {
        count_star_ms,
        exists_query_ms,
        bloom_filter_nanos,
        db_queries_avoided,
    })
}
