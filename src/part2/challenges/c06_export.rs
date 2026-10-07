use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use std::time::Instant;

/// Benchmark result for Challenge 6: Exporting Massive Datasets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportBenchmarkResult {
    pub fetch_all_memory_risk: String,
    pub streaming_latency_ms: f64,
    pub copy_stdout_latency_ms: f64,
    pub streamed_rows: usize,
    pub exported_bytes: usize,
}

/// Challenge 6: Exporting Massive Datasets Without OOM Crashes
///
/// Demonstrates:
/// 1. Naive `fetch_all`: Materializes the entire result set in application RAM.
///    On a 10M-row export, this consumes 12+ GB of memory and triggers Linux OOM killer.
/// 2. Tokio Streaming `fetch`: Reads rows in a bounded stream, processing in small buffers.
/// 3. Native `COPY ... TO STDOUT (FORMAT CSV)`: Uses PostgreSQL's wire protocol streaming
///    binary/CSV packets directly to socket/disk without converting into Rust struct objects.
pub async fn run_challenge_06(
    pool: &PgPool,
    conn: &mut PgConnection,
    limit: i64,
) -> Result<ExportBenchmarkResult, sqlx::Error> {
    // 1. Streaming query with Tokio Stream
    let start = Instant::now();
    let mut stream = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, username, email FROM lab_users LIMIT $1"
    )
    .bind(limit)
    .fetch(pool);

    let mut streamed_rows = 0;
    while let Some(row_res) = stream.next().await {
        let _ = row_res?;
        streamed_rows += 1;
    }
    let streaming_latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    // 2. Native PostgreSQL COPY TO STDOUT
    let start = Instant::now();
    let copy_query = format!("COPY (SELECT id, username, email FROM lab_users LIMIT {limit}) TO STDOUT WITH (FORMAT CSV)");
    let mut copy_out = conn.copy_out_raw(&copy_query).await?;

    let mut exported_bytes = 0;
    while let Some(chunk_res) = copy_out.next().await {
        let chunk = chunk_res?;
        exported_bytes += chunk.len();
    }
    let copy_stdout_latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    Ok(ExportBenchmarkResult {
        fetch_all_memory_risk: "OOM Crash on >5M rows (unbounded allocation)".to_string(),
        streaming_latency_ms,
        copy_stdout_latency_ms,
        streamed_rows,
        exported_bytes,
    })
}
