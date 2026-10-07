use crate::part1::schema::User;
use sqlx::PgExecutor;
use std::fmt;

/// Domain error enum mapping PostgreSQL SQLSTATE codes into strongly-typed application errors
#[derive(Debug)]
pub enum UserRepoError {
    UsernameAlreadyTaken(String),
    EmailAlreadyTaken(String),
    NotFound,
    InvalidData(String),
    Database(sqlx::Error),
}

impl std::error::Error for UserRepoError {}

impl fmt::Display for UserRepoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRepoError::UsernameAlreadyTaken(u) => write!(f, "Username '{u}' is already taken"),
            UserRepoError::EmailAlreadyTaken(e) => write!(f, "Email '{e}' is already registered"),
            UserRepoError::NotFound => write!(f, "User not found"),
            UserRepoError::InvalidData(msg) => write!(f, "Invalid data constraint: {msg}"),
            UserRepoError::Database(err) => write!(f, "Database error: {err}"),
        }
    }
}

impl From<sqlx::Error> for UserRepoError {
    fn from(err: sqlx::Error) -> Self {
        if let sqlx::Error::Database(ref db_err) = err {
            if let Some(code) = db_err.code() {
                match code.as_ref() {
                    // 23505: unique_violation
                    "23505" => {
                        let constraint = db_err.constraint().unwrap_or_default();
                        if constraint.contains("username") {
                            return UserRepoError::UsernameAlreadyTaken("Username violates unique index".into());
                        } else if constraint.contains("email") {
                            return UserRepoError::EmailAlreadyTaken("Email violates unique index".into());
                        }
                    }
                    // 23503: foreign_key_violation
                    "23503" => {
                        return UserRepoError::NotFound;
                    }
                    // 23514: check_violation
                    "23514" => {
                        return UserRepoError::InvalidData(db_err.message().to_string());
                    }
                    _ => {}
                }
            }
        }
        UserRepoError::Database(err)
    }
}

/// Production Repository Pattern utilizing `PgExecutor<'e>`
///
/// Decouples query execution from connection pooling:
/// Callers can pass either `&PgPool` for standalone execution,
/// or `&mut *tx` for transactional multi-step operations.
pub struct UserRepository;

impl UserRepository {
    /// Finds a user by ID using any executor (`&PgPool`, `&mut PgConnection`, or `&mut *tx`)
    pub async fn find_by_id<'e, E>(executor: E, id: i64) -> Result<Option<User>, UserRepoError>
    where
        E: PgExecutor<'e>,
    {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(executor)
        .await?;

        Ok(user)
    }

    /// Creates a user within an atomic transaction or directly on a pool connection
    pub async fn create_user<'e, E>(
        executor: E,
        username: &str,
        email: &str,
        country: &str,
    ) -> Result<User, UserRepoError>
    where
        E: PgExecutor<'e>,
    {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, country)
            VALUES ($1, $2, $3)
            RETURNING *
            "#
        )
        .bind(username)
        .bind(email)
        .bind(country)
        .fetch_one(executor)
        .await?;

        Ok(user)
    }

    /// Updates user status using `PgExecutor`
    pub async fn update_status<'e, E>(
        executor: E,
        id: i64,
        new_status: &str,
    ) -> Result<bool, UserRepoError>
    where
        E: PgExecutor<'e>,
    {
        let res = sqlx::query(
            "UPDATE users SET status = $1, updated_at = now() WHERE id = $2"
        )
        .bind(new_status)
        .bind(id)
        .execute(executor)
        .await?;

        Ok(res.rows_affected() > 0)
    }
}
