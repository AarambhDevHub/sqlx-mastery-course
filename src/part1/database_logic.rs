use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use sqlx::FromRow;
use sqlx::PgPool;

/// Summary record queried from view `user_order_details`
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserOrderViewRow {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub order_count: i64,
    pub total_spent: Decimal,
}

/// Summary record queried from materialized view `user_order_summaries`
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserOrderMaterializedSummary {
    pub user_id: i64,
    pub username: String,
    pub order_count: i64,
    pub total_spent: Decimal,
}

/// Payload sent via PostgreSQL `NOTIFY` when an order is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderEventPayload {
    pub order_id: i64,
    pub user_id: i64,
    pub amount: Decimal,
    pub status: String,
}

/// Queries the standard database view `user_order_details`
pub async fn query_user_order_view(
    pool: &PgPool,
) -> Result<Vec<UserOrderViewRow>, sqlx::Error> {
    sqlx::query_as::<_, UserOrderViewRow>(
        "SELECT * FROM user_order_details ORDER BY total_spent DESC"
    )
    .fetch_all(pool)
    .await
}

/// Queries the materialized view `user_order_summaries`
pub async fn query_materialized_view(
    pool: &PgPool,
) -> Result<Vec<UserOrderMaterializedSummary>, sqlx::Error> {
    sqlx::query_as::<_, UserOrderMaterializedSummary>(
        "SELECT * FROM user_order_summaries ORDER BY total_spent DESC"
    )
    .fetch_all(pool)
    .await
}

/// Demonstrates zero-downtime concurrent materialized view refresh:
/// `REFRESH MATERIALIZED VIEW CONCURRENTLY ...`
///
/// Because `user_order_summaries` has a UNIQUE index on `user_id`, concurrent refresh
/// updates the precomputed data without acquiring an exclusive read lock on the view.
pub async fn refresh_order_summary_materialized_view(
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query("REFRESH MATERIALIZED VIEW CONCURRENTLY user_order_summaries")
        .execute(pool)
        .await?;

    Ok(())
}

/// Subscribes to the PostgreSQL `order_events` notification channel via `PgListener`
/// and waits for the next asynchronous notification.
pub async fn listen_for_order_notification(
    pool: &PgPool,
) -> Result<OrderEventPayload, Box<dyn std::error::Error + Send + Sync>> {
    let mut listener = PgListener::connect_with(pool).await?;
    listener.listen("order_events").await?;

    let notification = listener.recv().await?;
    let payload: OrderEventPayload = serde_json::from_str(notification.payload())?;

    Ok(payload)
}
