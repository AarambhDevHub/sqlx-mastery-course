use dotenvy::dotenv;
use rust_decimal::Decimal;
use sqlx_mastery_course::part1::{
    advanced_sql, database_logic, joins_aggregates, powerful_dml, queries_basics,
    queries_dynamic, queries_macros,
    repository_pattern::{UserRepoError, UserRepository},
    schema::{UserProfile, UserRole},
    security_rls, transactions_locking, types_custom,
};
use sqlx_mastery_course::pool::init_pool;
use std::env;
use uuid::Uuid;

async fn get_test_pool() -> sqlx::PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:Darshan7069@localhost:5432/mastery_db".to_string());
    init_pool(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL test instance")
}

#[tokio::test]
async fn test_part1_crud_and_basics() {
    let pool = get_test_pool().await;

    let test_username = format!("user_{}", Uuid::now_v7());
    let test_email = format!("{}@test.com", test_username);

    // 1. Upsert
    let user_id = powerful_dml::upsert_user(&pool, &test_username, &test_email, "IN")
        .await
        .expect("Upsert failed");
    assert!(user_id > 0);

    // 2. Fetch one by ID
    let user = queries_basics::get_user_by_id(&pool, user_id)
        .await
        .expect("get_user_by_id failed");
    assert_eq!(user.username, test_username);
    assert_eq!(user.country, "IN");

    // 3. Raw column extraction
    let username = queries_basics::get_username_raw(&pool, user_id)
        .await
        .expect("get_username_raw failed");
    assert_eq!(username, test_username);

    // 4. Fetch optional (non-existent email)
    let non_existent = queries_basics::find_user_by_email(&pool, "does_not_exist@example.com")
        .await
        .expect("find_user_by_email query failed");
    assert!(non_existent.is_none());

    // 5. Query scalar count
    let count = queries_basics::count_users(&pool)
        .await
        .expect("count_users failed");
    assert!(count > 0);

    // 6. FromRow Decorated attributes
    let decorated = queries_basics::get_user_decorated(&pool, user_id)
        .await
        .expect("get_user_decorated failed");
    assert_eq!(decorated.username, test_username);
    assert_eq!(decorated.metadata.country, "IN");
    assert!(decorated.display_badge.is_some());
}

#[tokio::test]
async fn test_part1_macros_and_dynamic_sql() {
    let pool = get_test_pool().await;

    let test_username = format!("macro_{}", Uuid::now_v7());
    let test_email = format!("{}@macro.com", test_username);

    let id = powerful_dml::upsert_user(&pool, &test_username, &test_email, "US")
        .await
        .expect("Seed user failed");

    // Test query! macro
    let summary = queries_macros::get_user_summary_macro(&pool, id)
        .await
        .expect("get_user_summary_macro failed");
    assert!(summary.is_some());
    let s = summary.unwrap();
    assert_eq!(s.username, test_username);
    assert_eq!(s.role, UserRole::Member);

    // Test update with macro
    let affected = queries_macros::update_user_status_macro(&pool, id, "suspended")
        .await
        .expect("update_user_status_macro failed");
    assert_eq!(affected, 1);

    // Test dynamic query builder
    let filter = queries_dynamic::UserFilter {
        country: Some("US".to_string()),
        status: Some("suspended".to_string()),
        sort_by: Some("username".to_string()),
        sort_desc: false,
        limit: Some(10),
        ..Default::default()
    };
    let results = queries_dynamic::search_users_dynamic(&pool, &filter)
        .await
        .expect("search_users_dynamic failed");
    assert!(results.iter().any(|u| u.id == id));
}

#[tokio::test]
async fn test_part1_custom_types_and_dml() {
    let pool = get_test_pool().await;

    let v7 = types_custom::generate_uuid_v7();
    assert_ne!(v7, Uuid::nil());

    let uname = format!("types_{}", v7);
    let email = format!("{}@types.com", uname);

    let user_id = powerful_dml::upsert_user(&pool, &uname, &email, "IN")
        .await
        .expect("Upsert failed");

    // JSONB Profile update
    let profile = UserProfile {
        bio: Some("Rust ace".into()),
        avatar_url: Some("https://example.com/avatar.png".into()),
        preferences: serde_json::json!({ "theme": "dark", "notifications": true }),
    };
    types_custom::update_user_profile(&pool, user_id, profile.clone())
        .await
        .expect("update_user_profile failed");

    // Orders and Decimal math
    let order_id: i64 = sqlx::query_scalar(
        "INSERT INTO orders (user_id, amount, status) VALUES ($1, $2, 'completed') RETURNING id"
    )
    .bind(user_id)
    .bind(Decimal::new(14999, 2)) // 149.99
    .fetch_one(&pool)
    .await
    .expect("Order insert failed");
    assert!(order_id > 0);

    let total = types_custom::calculate_exact_order_total(&pool, user_id)
        .await
        .expect("calculate_exact_order_total failed");
    assert_eq!(total, Decimal::new(14999, 2));

    // Bulk UNNEST
    let batch_u: Vec<String> = (0..5).map(|i| format!("batch_{}_{}", i, Uuid::now_v7())).collect();
    let batch_e: Vec<String> = batch_u.iter().map(|u| format!("{u}@batch.com")).collect();
    let batch_c: Vec<String> = vec!["IN".into(), "US".into(), "UK".into(), "DE".into(), "JP".into()];

    let unnest_rows = powerful_dml::bulk_insert_users_unnest(&pool, &batch_u, &batch_e, &batch_c)
        .await
        .expect("bulk_insert_users_unnest failed");
    assert_eq!(unnest_rows, 5);
}

#[tokio::test]
async fn test_part1_joins_aggregates_and_advanced_sql() {
    let pool = get_test_pool().await;

    // Seed employees for recursive CTE
    let ceo_id: i64 = sqlx::query_scalar(
        "INSERT INTO employees (name, manager_id, department, salary) VALUES ('Alice CEO', NULL, 'Executive', 200000.00) RETURNING id"
    )
    .fetch_one(&pool)
    .await
    .expect("Insert CEO failed");

    let _vp_id: i64 = sqlx::query_scalar(
        "INSERT INTO employees (name, manager_id, department, salary) VALUES ('Bob VP', $1, 'Engineering', 150000.00) RETURNING id"
    )
    .bind(ceo_id)
    .fetch_one(&pool)
    .await
    .expect("Insert VP failed");

    // Test Recursive CTE
    let tree = advanced_sql::get_organization_tree(&pool)
        .await
        .expect("get_organization_tree failed");
    assert!(tree.len() >= 2);
    assert!(tree.iter().any(|node| node.depth == 1 && node.manager_id.is_none()));
    assert!(tree.iter().any(|node| node.depth == 2));

    // Joins & Aggregates
    let joins = joins_aggregates::get_users_with_optional_orders(&pool)
        .await
        .expect("get_users_with_optional_orders failed");
    assert!(!joins.is_empty());

    let rollup = joins_aggregates::get_country_status_rollup(&pool)
        .await
        .expect("get_country_status_rollup failed");
    assert!(!rollup.is_empty());

    // Materialized view refresh
    database_logic::refresh_order_summary_materialized_view(&pool)
        .await
        .expect("refresh_order_summary_materialized_view failed");

    let mv_rows = database_logic::query_materialized_view(&pool)
        .await
        .expect("query_materialized_view failed");
    assert!(!mv_rows.is_empty());
}

#[tokio::test]
async fn test_part1_transactions_locking_and_repository() {
    let pool = get_test_pool().await;

    // Create accounts
    let acc1: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (balance, version) VALUES (1000.00, 1) RETURNING id"
    )
    .fetch_one(&pool)
    .await
    .expect("Insert account 1 failed");

    let acc2: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (balance, version) VALUES (500.00, 1) RETURNING id"
    )
    .fetch_one(&pool)
    .await
    .expect("Insert account 2 failed");

    // Successful transfer
    transactions_locking::transfer_funds_committed(&pool, acc1, acc2, Decimal::new(20000, 2))
        .await
        .expect("Transfer failed");

    // Auto-rollback test
    let _ = transactions_locking::transfer_funds_auto_rollback(&pool, acc1, Decimal::new(10000, 2)).await;

    // Savepoint test
    transactions_locking::savepoint_demonstration(&pool, acc1)
        .await
        .expect("Savepoint demonstration failed");

    // Optimistic locking CAS
    let cas_success = transactions_locking::update_balance_optimistic(
        &pool,
        acc1,
        Decimal::new(95000, 2),
        1,
    )
    .await
    .expect("CAS update failed");
    assert!(cas_success);

    // Stale version CAS should fail
    let stale_cas = transactions_locking::update_balance_optimistic(
        &pool,
        acc1,
        Decimal::new(99000, 2),
        1, // version is already 2
    )
    .await
    .expect("Stale CAS update query failed");
    assert!(!stale_cas);

    // Repository Pattern with PgExecutor
    let unique_u = format!("repo_{}", Uuid::now_v7());
    let unique_e = format!("{}@repo.com", unique_u);

    // Call with &PgPool
    let user_created = UserRepository::create_user(&pool, &unique_u, &unique_e, "IN")
        .await
        .expect("UserRepository create_user with pool failed");
    assert_eq!(user_created.username, unique_u);

    // Call with active &mut *tx
    let mut tx = pool.begin().await.expect("Begin tx failed");
    let found_in_tx = UserRepository::find_by_id(&mut *tx, user_created.id)
        .await
        .expect("UserRepository find_by_id in tx failed");
    assert!(found_in_tx.is_some());
    tx.commit().await.expect("Commit tx failed");

    // Verify SQLSTATE 23505 unique error mapping
    let dup_err = UserRepository::create_user(&pool, &unique_u, "diff@email.com", "IN")
        .await
        .unwrap_err();
    match dup_err {
        UserRepoError::UsernameAlreadyTaken(_) => {} // verified!
        other => panic!("Expected UsernameAlreadyTaken, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_part1_row_level_security() {
    let pool = get_test_pool().await;

    let tenant_1 = Uuid::new_v4();
    let tenant_2 = Uuid::new_v4();

    // Insert doc for tenant 1
    let doc1 = security_rls::insert_document_for_tenant(&pool, tenant_1, "Tenant 1 Doc", "Private Secret 1")
        .await
        .expect("insert doc for tenant 1 failed");

    // Insert doc for tenant 2
    let doc2 = security_rls::insert_document_for_tenant(&pool, tenant_2, "Tenant 2 Doc", "Private Secret 2")
        .await
        .expect("insert doc for tenant 2 failed");

    // Query as tenant 1: MUST ONLY see doc1
    let t1_docs = security_rls::fetch_documents_for_tenant(&pool, tenant_1)
        .await
        .expect("fetch docs for tenant 1 failed");
    assert!(t1_docs.iter().any(|d| d.id == doc1.id));
    assert!(!t1_docs.iter().any(|d| d.id == doc2.id));

    // Query as tenant 2: MUST ONLY see doc2
    let t2_docs = security_rls::fetch_documents_for_tenant(&pool, tenant_2)
        .await
        .expect("fetch docs for tenant 2 failed");
    assert!(t2_docs.iter().any(|d| d.id == doc2.id));
    assert!(!t2_docs.iter().any(|d| d.id == doc1.id));
}
