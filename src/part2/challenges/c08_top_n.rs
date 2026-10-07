use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Instant;

/// Benchmark result for Challenge 8: Top-N and Latest Record Per User
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopNBenchmarkResult {
    pub correlated_subquery_ms: f64,
    pub window_function_ms: f64,
    pub distinct_on_ms: f64,
    pub lateral_join_ms: f64,
}

/// Challenge 8: Top-N and Latest Record Per User
///
/// Demonstrates:
/// 1. Correlated Subqueries: Re-executes the inner query for each outer candidate row (O(N*M) catastrophic scale).
/// 2. Window Functions: `ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY created_at DESC)`.
/// 3. `DISTINCT ON (user_id)`: Highly optimized in PostgreSQL for extracting exactly 1 latest item.
/// 4. `LATERAL` Join: Combined with index `(user_id, created_at DESC)`, executes an index seek per user.
pub async fn run_challenge_08(
    pool: &PgPool,
) -> Result<TopNBenchmarkResult, sqlx::Error> {
    // Ensure index on orders for fast lookups
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_user_created ON orders (user_id, created_at DESC)")
        .execute(pool)
        .await?;

    // 1. Correlated Subquery
    let start = Instant::now();
    let _ = sqlx::query(
        r#"
        SELECT o1.id, o1.user_id, o1.amount
        FROM orders o1
        WHERE o1.id = (
            SELECT o2.id 
            FROM orders o2 
            WHERE o2.user_id = o1.user_id 
            ORDER BY o2.created_at DESC 
            LIMIT 1
        )
        LIMIT 20
        "#
    )
    .fetch_all(pool)
    .await?;
    let correlated_subquery_ms = start.elapsed().as_secs_f64() * 1000.0;

    // 2. Window Functions
    let start = Instant::now();
    let _ = sqlx::query(
        r#"
        WITH ranked AS (
            SELECT id, user_id, amount,
                   ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY created_at DESC) AS rn
            FROM orders
        )
        SELECT id, user_id, amount
        FROM ranked
        WHERE rn <= 3
        LIMIT 20
        "#
    )
    .fetch_all(pool)
    .await?;
    let window_function_ms = start.elapsed().as_secs_f64() * 1000.0;

    // 3. DISTINCT ON
    let start = Instant::now();
    let _ = sqlx::query(
        r#"
        SELECT DISTINCT ON (user_id) user_id, id, amount, created_at
        FROM orders
        ORDER BY user_id, created_at DESC
        LIMIT 20
        "#
    )
    .fetch_all(pool)
    .await?;
    let distinct_on_ms = start.elapsed().as_secs_f64() * 1000.0;

    // 4. LATERAL Join
    let start = Instant::now();
    let _ = sqlx::query(
        r#"
        SELECT u.id AS user_id, l.id AS order_id, l.amount
        FROM users u
        CROSS JOIN LATERAL (
            SELECT id, amount
            FROM orders o
            WHERE o.user_id = u.id
            ORDER BY o.created_at DESC
            LIMIT 3
        ) l
        LIMIT 20
        "#
    )
    .fetch_all(pool)
    .await?;
    let lateral_join_ms = start.elapsed().as_secs_f64() * 1000.0;

    Ok(TopNBenchmarkResult {
        correlated_subquery_ms,
        window_function_ms,
        distinct_on_ms,
        lateral_join_ms,
    })
}
