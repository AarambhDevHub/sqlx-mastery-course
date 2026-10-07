use std::time::Duration;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use tracing::info;

/// Initializes a production-grade PostgreSQL connection pool using `PgPoolOptions`.
///
/// Configures pool bounds, acquisition timeouts, and connection recycling lifetimes.
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    info!("Initializing PostgreSQL connection pool...");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await?;

    info!("Database connection pool established successfully.");
    Ok(pool)
}

/// Runs a lightweight health check query (`SELECT 1`) to verify database responsiveness.
pub async fn check_health(pool: &PgPool) -> Result<(), sqlx::Error> {
    let row = sqlx::query("SELECT 1 as ping")
        .fetch_one(pool)
        .await?;

    let ping: i32 = row.try_get("ping")?;
    if ping == 1 {
        info!("Database health check PASSED (SELECT 1).");
        Ok(())
    } else {
        Err(sqlx::Error::Protocol("Unexpected health check value".into()))
    }
}

/// Calculates the theoretically optimal connection pool size according to the PostgreSQL formula:
/// `pool_size = (cpu_cores * 2) + effective_spindle_count`
pub fn calculate_ideal_pool_size(cpu_cores: u32, effective_spindles: u32) -> u32 {
    (cpu_cores * 2) + effective_spindles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_size_calculation() {
        // 16 cores on NVMe SSD (effective spindle count ~ 1-4)
        assert_eq!(calculate_ideal_pool_size(16, 1), 33);
        assert_eq!(calculate_ideal_pool_size(8, 1), 17);
    }
}
