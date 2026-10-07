use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Instant;

/// Benchmark result for Challenge 3: Autocomplete & Fuzzy Search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteBenchmarkResult {
    pub unindexed_contains_ms: f64,
    pub prefix_text_pattern_ms: f64,
    pub trgm_gin_fuzzy_ms: f64,
    pub prefix_matches: usize,
    pub fuzzy_matches: usize,
}

/// Challenge 3: Autocomplete & Fuzzy Search at Scale
///
/// Demonstrates:
/// 1. Naive `LIKE '%prefix%'`: Full sequential table scan (O(N)), completely unusable on 1B rows.
/// 2. Prefix `LIKE 'ali%'` with `text_pattern_ops`: B-Tree index range scan (O(log N)).
/// 3. Fuzzy search with `pg_trgm` GIN index: Trigram inverted index supporting arbitrary substring (`%xyz%`)
///    and typo-tolerant similarity matches.
pub async fn run_challenge_03(
    pool: &PgPool,
    search_term: &str,
) -> Result<AutocompleteBenchmarkResult, sqlx::Error> {
    // 1. Unindexed substring search (force seq scan)
    let start = Instant::now();
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL enable_indexscan = off").execute(&mut *tx).await?;
    sqlx::query("SET LOCAL enable_bitmapscan = off").execute(&mut *tx).await?;
    let unindexed_rows = sqlx::query_scalar::<_, String>(
        "SELECT username FROM lab_users WHERE username LIKE $1 LIMIT 10"
    )
    .bind(format!("%{search_term}%"))
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    let unindexed_contains_ms = start.elapsed().as_secs_f64() * 1000.0;

    // 2. Prefix index with text_pattern_ops
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_lab_users_username_pattern ON lab_users (username text_pattern_ops)"
    )
    .execute(pool)
    .await?;

    let start = Instant::now();
    let prefix_rows = sqlx::query_scalar::<_, String>(
        "SELECT username FROM lab_users WHERE username LIKE $1 LIMIT 10"
    )
    .bind(format!("{search_term}%"))
    .fetch_all(pool)
    .await?;
    let prefix_text_pattern_ms = start.elapsed().as_secs_f64() * 1000.0;

    // 3. Trigram GIN index for fuzzy search
    sqlx::query("CREATE EXTENSION IF NOT EXISTS pg_trgm").execute(pool).await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_lab_users_username_trgm_gin ON lab_users USING GIN (username gin_trgm_ops)"
    )
    .execute(pool)
    .await?;

    let start = Instant::now();
    let fuzzy_rows = sqlx::query_scalar::<_, String>(
        r#"
        SELECT username 
        FROM lab_users 
        WHERE username % $1 OR username ILIKE $2
        ORDER BY similarity(username, $1) DESC 
        LIMIT 10
        "#
    )
    .bind(search_term)
    .bind(format!("%{search_term}%"))
    .fetch_all(pool)
    .await?;
    let trgm_gin_fuzzy_ms = start.elapsed().as_secs_f64() * 1000.0;

    Ok(AutocompleteBenchmarkResult {
        unindexed_contains_ms,
        prefix_text_pattern_ms,
        trgm_gin_fuzzy_ms,
        prefix_matches: prefix_rows.len().max(unindexed_rows.len()),
        fuzzy_matches: fuzzy_rows.len(),
    })
}
