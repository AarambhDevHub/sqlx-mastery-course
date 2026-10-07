use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;

/// Result struct for Left Join query
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserWithOrderInfo {
    pub user_id: i64,
    pub username: String,
    pub order_id: Option<i64>,
    pub order_amount: Option<Decimal>,
    pub order_status: Option<String>,
}

/// Result struct for Employee self-join
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmployeeManager {
    pub employee_id: i64,
    pub employee_name: String,
    pub manager_id: Option<i64>,
    pub manager_name: Option<String>,
}

/// Result struct for conditional filter aggregate query
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderStatusBreakdown {
    pub total_orders: i64,
    pub completed_orders: i64,
    pub pending_orders: i64,
    pub refunded_orders: i64,
}

/// Result struct for ROLLUP / GROUPING SETS query
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CountryStatusRollup {
    pub country: Option<String>,
    pub status: Option<String>,
    pub count: i64,
}

/// Demonstrates LEFT JOIN between users and orders
pub async fn get_users_with_optional_orders(
    pool: &PgPool,
) -> Result<Vec<UserWithOrderInfo>, sqlx::Error> {
    sqlx::query_as::<_, UserWithOrderInfo>(
        r#"
        SELECT 
            u.id AS user_id,
            u.username,
            o.id AS order_id,
            o.amount AS order_amount,
            o.status AS order_status
        FROM users u
        LEFT JOIN orders o ON u.id = o.user_id
        ORDER BY u.id ASC, o.id ASC
        "#
    )
    .fetch_all(pool)
    .await
}

/// Demonstrates self-join for employee-to-manager hierarchy mapping
pub async fn get_employees_with_managers(
    pool: &PgPool,
) -> Result<Vec<EmployeeManager>, sqlx::Error> {
    sqlx::query_as::<_, EmployeeManager>(
        r#"
        SELECT 
            e.id AS employee_id,
            e.name AS employee_name,
            m.id AS manager_id,
            m.name AS manager_name
        FROM employees e
        LEFT JOIN employees m ON e.manager_id = m.id
        ORDER BY e.id ASC
        "#
    )
    .fetch_all(pool)
    .await
}

/// Demonstrates aggregate query with conditional `FILTER (WHERE ...)` clauses
pub async fn get_order_status_aggregates(
    pool: &PgPool,
) -> Result<OrderStatusBreakdown, sqlx::Error> {
    sqlx::query_as::<_, OrderStatusBreakdown>(
        r#"
        SELECT 
            COUNT(*) AS total_orders,
            COUNT(*) FILTER (WHERE status = 'completed') AS completed_orders,
            COUNT(*) FILTER (WHERE status = 'pending') AS pending_orders,
            COUNT(*) FILTER (WHERE status = 'refunded') AS refunded_orders
        FROM orders
        "#
    )
    .fetch_one(pool)
    .await
}

/// Demonstrates multidimensional aggregation using `ROLLUP`
pub async fn get_country_status_rollup(
    pool: &PgPool,
) -> Result<Vec<CountryStatusRollup>, sqlx::Error> {
    sqlx::query_as::<_, CountryStatusRollup>(
        r#"
        SELECT 
            country,
            status,
            COUNT(*) AS count
        FROM users
        GROUP BY ROLLUP (country, status)
        ORDER BY country NULLS LAST, status NULLS LAST
        "#
    )
    .fetch_all(pool)
    .await
}
