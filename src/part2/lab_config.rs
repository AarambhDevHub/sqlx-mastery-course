use serde::{Deserialize, Serialize};
use sqlx::{AssertSqlSafe, PgPool, Row};

/// Lab hardware specifications and PostgreSQL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabSettings {
    pub hardware: String,
    pub shared_buffers: String,
    pub work_mem: String,
    pub maintenance_work_mem: String,
    pub effective_cache_size: String,
    pub max_parallel_workers_per_gather: i32,
}

impl Default for LabSettings {
    fn default() -> Self {
        Self {
            hardware: "16 vCPUs (AMD EPYC), 64 GB RAM, Enterprise NVMe SSD".to_string(),
            shared_buffers: "16GB".to_string(),
            work_mem: "64MB".to_string(),
            maintenance_work_mem: "4GB".to_string(),
            effective_cache_size: "48GB".to_string(),
            max_parallel_workers_per_gather: 4,
        }
    }
}

/// Fetches current active PostgreSQL configuration settings via `SHOW <setting>`
pub async fn fetch_postgres_runtime_settings(pool: &PgPool) -> Result<LabSettings, sqlx::Error> {
    let shared_buffers: String = sqlx::query_scalar("SHOW shared_buffers")
        .fetch_one(pool)
        .await
        .unwrap_or_else(|_| "128MB".into());

    let work_mem: String = sqlx::query_scalar("SHOW work_mem")
        .fetch_one(pool)
        .await
        .unwrap_or_else(|_| "4MB".into());

    let maintenance_work_mem: String = sqlx::query_scalar("SHOW maintenance_work_mem")
        .fetch_one(pool)
        .await
        .unwrap_or_else(|_| "64MB".into());

    let effective_cache_size: String = sqlx::query_scalar("SHOW effective_cache_size")
        .fetch_one(pool)
        .await
        .unwrap_or_else(|_| "4GB".into());

    let max_workers: String = sqlx::query_scalar("SHOW max_parallel_workers_per_gather")
        .fetch_one(pool)
        .await
        .unwrap_or_else(|_| "2".into());

    let max_parallel_workers_per_gather: i32 = max_workers.parse().unwrap_or(2);

    Ok(LabSettings {
        hardware: "16 vCPUs (AMD EPYC), 64 GB RAM, Enterprise NVMe SSD".to_string(),
        shared_buffers,
        work_mem,
        maintenance_work_mem,
        effective_cache_size,
        max_parallel_workers_per_gather,
    })
}

/// Result of parsing an `EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)` execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainPlanMetrics {
    pub execution_time_ms: f64,
    pub planning_time_ms: f64,
    pub node_type: String,
    pub shared_hit_blocks: i64,
    pub shared_read_blocks: i64,
    pub raw_json: serde_json::Value,
}

/// Executes and parses `EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)` on any raw query string
pub async fn explain_analyze_query(
    pool: &PgPool,
    sql: &str,
) -> Result<ExplainPlanMetrics, Box<dyn std::error::Error + Send + Sync>> {
    let explain_sql = format!("EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) {sql}");
    let row = sqlx::query(AssertSqlSafe(explain_sql)).fetch_one(pool).await?;

    let plan_value: serde_json::Value = row.try_get(0)?;
    let root = plan_value
        .as_array()
        .and_then(|a| a.first())
        .ok_or("Empty plan array")?;

    let plan_node = root.get("Plan").ok_or("Missing Plan node")?;
    let execution_time_ms = root
        .get("Execution Time")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let planning_time_ms = root
        .get("Planning Time")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let node_type = plan_node
        .get("Node Type")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();
    let shared_hit_blocks = plan_node
        .get("Shared Hit Blocks")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let shared_read_blocks = plan_node
        .get("Shared Read Blocks")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    Ok(ExplainPlanMetrics {
        execution_time_ms,
        planning_time_ms,
        node_type,
        shared_hit_blocks,
        shared_read_blocks,
        raw_json: plan_value,
    })
}
