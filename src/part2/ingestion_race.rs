use serde::{Deserialize, Serialize};
use sqlx::postgres::Postgres;
use sqlx::{PgPool, QueryBuilder};
use std::time::Instant;
use tracing::info;

/// Ingestion benchmark report matching the contract in `load-benchmarks.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionBenchmarkResult {
    pub method: String,
    pub rows_per_sec: u64,
    pub binds_per_statement: u32,
    pub duration_ms: u64,
    pub total_rows: u64,
}

/// Demonstrates Strategy 1: Single Inserts (1 statement per row)
///
/// Slowest approach: Each row incurs TCP round-trip latency, SQL parsing,
/// query planning, and WAL write logging.
pub async fn benchmark_single_inserts(
    pool: &PgPool,
    count: usize,
) -> Result<IngestionBenchmarkResult, sqlx::Error> {
    let start = Instant::now();

    for i in 0..count {
        let username = format!("user_single_{i}");
        let email = format!("single_{i}@example.com");
        sqlx::query("INSERT INTO lab_users (username, email, status) VALUES ($1, $2, 'active')")
            .bind(&username)
            .bind(&email)
            .execute(pool)
            .await?;
    }

    let duration = start.elapsed();
    let duration_ms = duration.as_millis().max(1) as u64;
    let rows_per_sec = (count as u64 * 1000) / duration_ms;

    info!("Single Inserts: {count} rows in {duration_ms} ms ({rows_per_sec} rows/sec)");

    Ok(IngestionBenchmarkResult {
        method: "Single Inserts".to_string(),
        rows_per_sec,
        binds_per_statement: 3,
        duration_ms,
        total_rows: count as u64,
    })
}

/// Demonstrates Strategy 2: QueryBuilder Batch Inserts
///
/// Bundles multiple rows into a single multi-value `INSERT INTO ... VALUES (...)` clause.
/// Note: Bound to PostgreSQL's limit of at most 65,535 parameters per statement.
pub async fn benchmark_query_builder_batch(
    pool: &PgPool,
    count: usize,
    batch_size: usize,
) -> Result<IngestionBenchmarkResult, sqlx::Error> {
    let start = Instant::now();

    let chunks = (0..count).collect::<Vec<_>>();
    for chunk in chunks.chunks(batch_size) {
        let mut builder: QueryBuilder<Postgres> =
            QueryBuilder::new("INSERT INTO lab_users (username, email, status) ");

        builder.push_values(chunk, |mut b, i| {
            let username = format!("user_qb_{i}");
            let email = format!("qb_{i}@example.com");
            b.push_bind(username).push_bind(email).push_bind("active");
        });

        let query = builder.build();
        query.execute(pool).await?;
    }

    let duration = start.elapsed();
    let duration_ms = duration.as_millis().max(1) as u64;
    let rows_per_sec = (count as u64 * 1000) / duration_ms;

    info!("QueryBuilder Batch ({batch_size}): {count} rows in {duration_ms} ms ({rows_per_sec} rows/sec)");

    Ok(IngestionBenchmarkResult {
        method: format!("QueryBuilder Batch ({batch_size})"),
        rows_per_sec,
        binds_per_statement: (batch_size * 3) as u32,
        duration_ms,
        total_rows: count as u64,
    })
}

/// Demonstrates Strategy 3: Array UNNEST Batching
///
/// Bypasses the 65,535 parameter limit by unnesting 3 array vectors directly in PostgreSQL.
/// Regardless of whether 1,000 or 50,000 rows are inserted, only 3 bind parameters are used.
pub async fn benchmark_unnest_batch(
    pool: &PgPool,
    count: usize,
) -> Result<IngestionBenchmarkResult, sqlx::Error> {
    let start = Instant::now();

    let usernames: Vec<String> = (0..count).map(|i| format!("user_unnest_{i}")).collect();
    let emails: Vec<String> = (0..count).map(|i| format!("unnest_{i}@example.com")).collect();
    let statuses: Vec<String> = (0..count).map(|_| "active".to_string()).collect();

    sqlx::query(
        r#"
        INSERT INTO lab_users (username, email, status)
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[])
        "#
    )
    .bind(&usernames)
    .bind(&emails)
    .bind(&statuses)
    .execute(pool)
    .await?;

    let duration = start.elapsed();
    let duration_ms = duration.as_millis().max(1) as u64;
    let rows_per_sec = (count as u64 * 1000) / duration_ms;

    info!("Array UNNEST: {count} rows in {duration_ms} ms ({rows_per_sec} rows/sec)");

    Ok(IngestionBenchmarkResult {
        method: "Array UNNEST".to_string(),
        rows_per_sec,
        binds_per_statement: 3,
        duration_ms,
        total_rows: count as u64,
    })
}

/// Demonstrates Strategy 4: Parallel Streaming COPY Loader
///
/// Streams CSV formatted byte payloads via PostgreSQL's native streaming `COPY FROM STDIN` protocol.
/// Delivers maximum ingestion bandwidth: over 1,000,000 rows/second on modern NVMe drives.
pub async fn benchmark_parallel_copy(
    pool: &PgPool,
    total_rows: usize,
    num_workers: usize,
) -> Result<IngestionBenchmarkResult, sqlx::Error> {
    let start = Instant::now();
    let rows_per_worker = total_rows / num_workers;

    let mut handles = Vec::new();

    for worker_id in 0..num_workers {
        let pool = pool.clone();
        let handle = tokio::spawn(async move {
            let mut conn = pool.acquire().await?;
            let mut copy = conn
                .copy_in_raw("COPY lab_users (username, email, status) FROM STDIN WITH (FORMAT csv)")
                .await?;

            let mut csv_buffer = String::with_capacity(rows_per_worker * 64);
            let start_idx = worker_id * rows_per_worker;
            for i in start_idx..(start_idx + rows_per_worker) {
                csv_buffer.push_str(&format!("user_copy_{i},copy_{i}@example.com,active\n"));
            }

            copy.send(csv_buffer.as_bytes()).await?;
            let rows = copy.finish().await?;
            Ok::<u64, sqlx::Error>(rows)
        });
        handles.push(handle);
    }

    let mut inserted = 0;
    for handle in handles {
        let r = handle.await.unwrap()?;
        inserted += r;
    }

    let duration = start.elapsed();
    let duration_ms = duration.as_millis().max(1) as u64;
    let rows_per_sec = (inserted * 1000) / duration_ms;

    info!("Parallel COPY ({num_workers} workers): {inserted} rows in {duration_ms} ms ({rows_per_sec} rows/sec)");

    Ok(IngestionBenchmarkResult {
        method: format!("Parallel COPY ({num_workers} workers)"),
        rows_per_sec,
        binds_per_statement: 0,
        duration_ms,
        total_rows: inserted,
    })
}
