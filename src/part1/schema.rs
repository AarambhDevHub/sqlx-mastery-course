use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Postgres custom ENUM mapping for `user_role`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    Moderator,
    #[default]
    Member,
}

/// JSONB profile model embedded within the user record
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UserProfile {
    #[serde(default)]
    pub bio: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub preferences: serde_json::Value,
}

/// Core User entity mapped from the `users` table
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub country: String,
    pub status: String,
    pub role: UserRole,
    pub profile: sqlx::types::Json<UserProfile>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Order entity mapped from the `orders` table
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: i64,
    pub user_id: i64,
    pub amount: Decimal,
    pub status: String,
    pub total_cents: i64,
    pub created_at: DateTime<Utc>,
}

/// Embedded metadata struct demonstrating `#[sqlx(flatten)]`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, Default)]
pub struct UserMetadata {
    pub country: String,
    pub status: String,
}

/// Demonstrates `FromRow` attributes:
/// - `rename`: column alias mapping
/// - `flatten`: nested struct decomposition
/// - `skip`: local computed in-memory fields
/// - `default`: fallback value if column is null/missing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct UserDecorated {
    pub id: i64,
    #[sqlx(rename = "user_name")]
    pub username: String,
    pub email: String,
    #[sqlx(flatten)]
    pub metadata: UserMetadata,
    #[sqlx(default)]
    pub role: UserRole,
    #[sqlx(skip)]
    pub display_badge: Option<String>,
}

/// Employee model for hierarchical queries & recursive CTEs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Employee {
    pub id: i64,
    pub name: String,
    pub manager_id: Option<i64>,
    pub department: String,
    pub salary: Decimal,
    pub created_at: DateTime<Utc>,
}

/// Account model for transaction and concurrency demonstrations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: i64,
    pub user_id: Option<i64>,
    pub balance: Decimal,
    pub version: i32,
    pub updated_at: DateTime<Utc>,
}

/// Multi-tenant document model for Row-Level Security
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: i64,
    pub tenant_id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}
