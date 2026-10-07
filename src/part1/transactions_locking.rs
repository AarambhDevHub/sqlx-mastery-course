use rust_decimal::Decimal;
use sqlx::{Acquire, PgPool, Row};
use std::time::Duration;
use tokio::time::sleep;

/// Demonstrates standard transaction commit lifecycle
pub async fn transfer_funds_committed(
    pool: &PgPool,
    from_account_id: i64,
    to_account_id: i64,
    amount: Decimal,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Deduct from sender
    sqlx::query("UPDATE accounts SET balance = balance - $1 WHERE id = $2")
        .bind(amount)
        .bind(from_account_id)
        .execute(&mut *tx)
        .await?;

    // Credit to recipient
    sqlx::query("UPDATE accounts SET balance = balance + $1 WHERE id = $2")
        .bind(amount)
        .bind(to_account_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

/// Demonstrates automatic transaction rollback when an error occurs or `tx` is dropped early
pub async fn transfer_funds_auto_rollback(
    pool: &PgPool,
    from_account_id: i64,
    amount: Decimal,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE accounts SET balance = balance - $1 WHERE id = $2")
        .bind(amount)
        .bind(from_account_id)
        .execute(&mut *tx)
        .await?;

    // Simulating an unexpected failure: returning an error causes `tx` to be dropped.
    // SQLx's `Drop` implementation will automatically issue a `ROLLBACK`.
    Err(sqlx::Error::RowNotFound)
}

/// Demonstrates Savepoints (nested transactions in SQLx)
///
/// Calling `tx.begin().await` generates a `SAVEPOINT`. Rolling it back or dropping it
/// allows the parent transaction to proceed and commit unaffected.
pub async fn savepoint_demonstration(
    pool: &PgPool,
    account_id: i64,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Parent transaction modification
    sqlx::query("UPDATE accounts SET balance = balance + 100 WHERE id = $1")
        .bind(account_id)
        .execute(&mut *tx)
        .await?;

    // Create a savepoint via Acquire::begin
    {
        let mut sp = tx.begin().await?;
        // Speculative update that violates balance check or fails
        let res = sqlx::query("UPDATE accounts SET balance = -99999 WHERE id = $1")
            .bind(account_id)
            .execute(&mut *sp)
            .await;

        if res.is_err() {
            // Rollback savepoint without aborting outer transaction
            sp.rollback().await?;
        } else {
            sp.commit().await?;
        }
    }

    // Outer transaction can still commit successfully
    tx.commit().await?;
    Ok(())
}

/// Demonstrates setting Serializable transaction isolation level
pub async fn execute_serializable_read(
    pool: &PgPool,
    account_id: i64,
) -> Result<Decimal, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
        .execute(&mut *tx)
        .await?;

    let row = sqlx::query("SELECT balance FROM accounts WHERE id = $1")
        .bind(account_id)
        .fetch_one(&mut *tx)
        .await?;

    let balance: Decimal = row.try_get("balance")?;
    tx.commit().await?;
    Ok(balance)
}

/// Demonstrates pessimistic row-level locking (`SELECT ... FOR UPDATE`)
pub async fn get_balance_for_update(
    pool: &PgPool,
    account_id: i64,
) -> Result<Decimal, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let row = sqlx::query("SELECT balance FROM accounts WHERE id = $1 FOR UPDATE")
        .bind(account_id)
        .fetch_one(&mut *tx)
        .await?;

    let balance: Decimal = row.try_get("balance")?;
    tx.commit().await?;
    Ok(balance)
}

/// Demonstrates optimistic concurrency control with Compare-And-Swap (CAS)
/// and version checking. Returns `true` if update succeeded, `false` if version conflict.
pub async fn update_balance_optimistic(
    pool: &PgPool,
    account_id: i64,
    new_balance: Decimal,
    expected_version: i32,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE accounts
        SET balance = $1, version = version + 1, updated_at = now()
        WHERE id = $2 AND version = $3
        "#
    )
    .bind(new_balance)
    .bind(account_id)
    .bind(expected_version)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() == 1)
}

/// Simulates a deadlock scenario by spawning two tasks acquiring locks in reverse order,
/// detecting PostgreSQL's SQLSTATE error code `40P01` (deadlock_detected).
pub async fn simulate_deadlock(
    pool: &PgPool,
    acc_a: i64,
    acc_b: i64,
) -> Result<bool, sqlx::Error> {
    let pool1 = pool.clone();
    let pool2 = pool.clone();

    // Task 1: Lock A, sleep 100ms, then lock B
    let handle1 = tokio::spawn(async move {
        let mut tx = pool1.begin().await?;
        sqlx::query("SELECT * FROM accounts WHERE id = $1 FOR UPDATE")
            .bind(acc_a)
            .execute(&mut *tx)
            .await?;
        sleep(Duration::from_millis(150)).await;
        sqlx::query("SELECT * FROM accounts WHERE id = $1 FOR UPDATE")
            .bind(acc_b)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok::<(), sqlx::Error>(())
    });

    // Task 2: Lock B, sleep 100ms, then lock A (reverse order -> deadlock)
    let handle2 = tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        let mut tx = pool2.begin().await?;
        sqlx::query("SELECT * FROM accounts WHERE id = $1 FOR UPDATE")
            .bind(acc_b)
            .execute(&mut *tx)
            .await?;
        sleep(Duration::from_millis(150)).await;
        sqlx::query("SELECT * FROM accounts WHERE id = $1 FOR UPDATE")
            .bind(acc_a)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok::<(), sqlx::Error>(())
    });

    let res1 = handle1.await.unwrap();
    let res2 = handle2.await.unwrap();

    let is_deadlock_detected = match (&res1, &res2) {
        (Err(sqlx::Error::Database(db_err)), _) if db_err.code().as_deref() == Some("40P01") => true,
        (_, Err(sqlx::Error::Database(db_err))) if db_err.code().as_deref() == Some("40P01") => true,
        _ => false,
    };

    Ok(is_deadlock_detected)
}
