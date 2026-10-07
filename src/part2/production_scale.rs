use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

/// Diagnostics model for pg_stat_statements top query tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStat {
    pub queryid: Option<i64>,
    pub query: String,
    pub calls: i64,
    pub total_exec_time_ms: f64,
    pub mean_exec_time_ms: f64,
}

/// Sizing recommendation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolSizingAdvice {
    pub cpu_cores: u32,
    pub effective_spindles: u32,
    pub recommended_pool_size: u32,
    pub explanation: String,
}

/// Computes the recommended connection pool size based on hardware resources:
///
/// Formula: `connections = (cores * 2) + effective_spindles`
pub fn compute_pool_sizing(cpu_cores: u32, effective_spindles: u32) -> PoolSizingAdvice {
    let size = (cpu_cores * 2) + effective_spindles;
    PoolSizingAdvice {
        cpu_cores,
        effective_spindles,
        recommended_pool_size: size,
        explanation: format!(
            "On {cpu_cores} physical cores with NVMe storage, {size} connections maximizes throughput without CPU context-switching thrashing."
        ),
    }
}

/// Queries PostgreSQL's `pg_stat_statements` (if enabled) for the top slow queries
pub async fn fetch_top_slow_queries(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<QueryStat>, sqlx::Error> {
    // Attempt querying pg_stat_statements if extension is present
    let rows = sqlx::query(
        r#"
        SELECT 
            queryid, 
            query, 
            calls, 
            total_exec_time AS total_exec_time_ms, 
            mean_exec_time AS mean_exec_time_ms
        FROM pg_stat_statements
        ORDER BY total_exec_time DESC
        LIMIT $1
        "#
    )
    .bind(limit)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(r) => {
            let mut list = Vec::new();
            for row in r {
                list.push(QueryStat {
                    queryid: row.try_get("queryid").ok(),
                    query: row.try_get("query").unwrap_or_default(),
                    calls: row.try_get("calls").unwrap_or(0),
                    total_exec_time_ms: row.try_get("total_exec_time_ms").unwrap_or(0.0),
                    mean_exec_time_ms: row.try_get("mean_exec_time_ms").unwrap_or(0.0),
                });
            }
            Ok(list)
        }
        Err(_) => {
            // Extension might not be loaded in shared_preload_libraries on dev instance
            Ok(vec![])
        }
    }
}
