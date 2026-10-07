# 🎬 YouTube Script — SQLx + PostgreSQL Full Course in Rust (Part 1 of 2)

**Channel:** Aarambh Dev Hub  
**Estimated Duration:** about 3 hours (target 2h55m – 3h15m)  
**Voice:** Google Gemini TTS (gemini-3.8-flash-tts) — natural conversational tone  
**Note:** No bullet reads — flows like a real person. No code syntax reading (never pronounce characters like semicolons, brackets, or arrow signs). All code is already written and displayed in VS Code on camera, so this is a senior engineer walking through a clean codebase. Project directory for this video: `sqlx_mastery`. Built for SQLx 0.9 and PostgreSQL 16+.

---

## HOOK AND THE 100 CRORE ROWS TEASER

### Part 1
Your SQLx code compiles. Your integration tests pass. And yet, it can still bring your production database to its knees the moment real traffic lands.

### Part 2
That is the uncomfortable reality of database engineering. The Rust compiler can verify your types. SQLx can even query a live database during compilation to guarantee that your columns and types match. But the compiler cannot warn you that a query is about to execute a full sequential scan across a table with hundreds of millions of rows, or that two concurrent transactions are about to deadlock each other into a total freeze.

### Part 3
Before we dive into the foundations, take a look at what is running on my second monitor right now. This is a preview of Part 2 of this masterclass series. Here, we have loaded one hundred crore rows into PostgreSQL—that is one billion rows. And right here on screen, we execute a query searching for a single username among that entire billion-row dataset. Look at the latency: zero point four two milliseconds. In Part 2, we tackle thirteen real-world high-scale challenges just like that.

### Part 4
To survive that kind of scale, you need an unbreakable foundation first. That is what we are building today in Part 1. We start from an empty Docker container, walk through robust schema modeling and constraints, master the complete SQLx query API, compile-time macros, offline verification, advanced Postgres types, joins, window functions, upserts, database logic, transactions, locking, clean repository architectures, row-level security, and test harnesses. Everything is written in Rust, running on SQLx 0.9. Let us get started.

---

## THE ROADMAP AND WHAT'S NEW IN SQLX 0.9

### Part 1
Here is our roadmap across the next three hours. We begin with setup, creating our PostgreSQL container in Docker, configuring the SQLx command-line interface, and establishing our first connection pool from Rust. Then we move straight into schema modeling and constraints, because catching bad data at the database level is orders of magnitude cheaper than catching it in application logic.

### Part 2
From there, we explore the core SQLx query API: mapping rows to structs, handling nullability, and configuring derive attributes. Next, we cover compile-time checked macros, offline metadata preparation for continuous integration pipelines, and constructing safe dynamic SQL with query builders. Then we tackle advanced Postgres types in Rust—UUIDs, high-precision decimals, timestamps, native arrays, and JSONB.

### Part 3
In the second half, we dive deep into advanced SQL: multi-table joins, subqueries, aggregates, common table expressions, recursive CTEs, window functions, lateral joins, and deduplication with DISTINCT ON. Finally, we explore powerful data modification techniques, transactions, isolation levels, row locking, deadlocks, repository design patterns with the Executor trait, row-level security, and automated testing with isolated test databases.

### Part 4
Before we touch the project configuration, let us highlight what makes SQLx 0.9 distinct. Released recently, SQLx 0.9 updates its minimum supported Rust version to 1.94, modernizes its internal async architecture, removes deprecated legacy connection interfaces, and introduces new safety invariants such as the AssertSqlSafe marker trait for dynamic query construction. SQLx remains a pure Rust async driver: there is no C dependency, no libpq requirement, and no heavy object-relational mapping layer hiding what your database is actually doing.

---

## TEST LAB SETUP AND DOCKER ENVIRONMENT

### Part 1
Let us look at our project layout in VS Code. We start inside our `sqlx_mastery` folder. In the root, we have our `docker-compose.yml` file. It configures a PostgreSQL 16 container, exposes port 5432, sets up our default database name, username, and password, and attaches a persistent local volume. Having a containerized database guarantees that your local development environment mirrors what runs in production.

### Part 2
Next is the SQLx command-line interface. When installing `sqlx-cli`, there is a critical detail many developers miss: never install it with the locked flag. On modern Rust toolchains, dependency resolution requires flexibility. We install it with default features disabled, enabling only Rustls and the Postgres driver. This keeps the binary lightweight and avoids pulling in unnecessary SQLite or MySQL dependencies.

### Part 3
Now let us open `Cargo.toml`. Our dependencies include SQLx version 0.9 with features for Postgres, the Tokio runtime with Rustls, macros, UUID support, Chrono timestamps, Rust Decimal, and Serde JSON. We also include Tokio for our asynchronous runtime, Serde for data serialization, and Dotenvy to load environment variables from our local `.env` configuration file.

### Part 4
In `.env`, we specify our database connection string. It includes the Postgres protocol, user credentials, host address, port, and database name. We also create a `sqlx.toml` configuration file in the project root. This tells the SQLx tooling where our migration directory lives and how offline query metadata should be managed.

### Part 5
Let us open `src/main.rs` to inspect our connection initialization. We use `PgPoolOptions` to configure our connection pool. A database pool maintains a warm set of persistent TCP sockets connected to PostgreSQL. We set a maximum pool size, a minimum idle connection threshold, an acquire timeout, and an idle timeout. When the application starts, we connect to the database and immediately ping the server to verify network connectivity before serving any requests.

---

## SCHEMA DESIGN, CONSTRAINTS, AND NORMALIZATION

### Part 1
Let us look at schema modeling. Many developers treat database tables as simple dumb storage bins, pushing all validation rules into Rust application code. That is a dangerous mistake. Application code can have bugs, background jobs can bypass API validation, and multiple microservices might touch the same database. Your database schema must be your final, unbreachable line of defense.

### Part 2
Let us inspect our primary migration file inside the `migrations` folder. Notice the primary key definition on our `users` table: we use `BIGINT GENERATED ALWAYS AS IDENTITY`. Older tutorials often recommend the `SERIAL` pseudo-type. In modern PostgreSQL, `SERIAL` is considered legacy; identity columns adhere strictly to the SQL standard, integrate cleanly with permissions, and prevent accidental manual overrides.

### Part 3
Next, look at our column data types. For text fields like username and email, we use PostgreSQL's native `TEXT` type rather than an arbitrary `VARCHAR(255)`. In Postgres, `TEXT` and `VARCHAR` have identical internal storage and performance characteristics. Limiting a string to 255 characters does not save space; it simply introduces artificial limits that will cause headaches down the road. If you need length validation, use a check constraint instead.

### Part 4
Now examine the constraints. We mark username and email as `NOT NULL` and add explicit `UNIQUE` constraints. For user status, we apply a `CHECK` constraint ensuring the value belongs to a specific set: active, suspended, or pending. For creation timestamps, we choose `TIMESTAMPTZ`—timestamp with time zone—defaulting to `now()`. Storing timestamps without time zones in a distributed system is an invitation to data corruption whenever clients operate across daylight saving boundaries.

### Part 5
Now look at our `orders` table. It contains a foreign key referencing the `users` table with `ON DELETE CASCADE`. When a user is removed, their associated orders are purged cleanly. But notice what comes right after the table definition: an explicit index on the `user_id` foreign key column. PostgreSQL does NOT automatically create indexes on foreign key columns. If you omit this index, deleting a user or joining users to orders will force a full sequential scan of the entire orders table, locking the table and crippling performance.

### Part 6
Let us briefly touch on normalization in plain English. First normal form means each column contains atomic values, not packed lists or delimited strings. Second normal form requires every non-key column to depend on the entire primary key. Third normal form requires non-key columns to depend solely on the primary key, with no transitive dependencies. Keep your schema normalized to avoid update anomalies. We only introduce denormalization when concrete benchmark metrics demand it.

---

## QUERYING WITH SQLX: THE CORE API

### Part 1
Now that our schema is applied, let us examine how SQLx executes queries from Rust. Open `src/queries/basics.rs`. SQLx provides three foundational query functions: `sqlx::query`, `sqlx::query_as`, and `sqlx::query_scalar`. Understanding when to reach for each is essential.

### Part 2
The lowest-level function is `sqlx::query`. It takes a raw SQL string and returns a query object. When executed, it returns a `PgRow`. You can extract values from this row by column name or numeric index using the `get` method. This gives you complete manual control, but writing manual column extractions across hundreds of models is tedious and error-prone.

### Part 3
That brings us to `sqlx::query_as`. This is what you will use for the vast majority of application queries. It takes the target struct type as a generic parameter and maps the returned database columns directly into your struct fields. To enable this, your struct derives the `FromRow` trait. During execution, SQLx matches column names returned by PostgreSQL to the field names of your Rust struct.

### Part 4
Let us talk about parameter binding. Look at the SQL query on screen: `SELECT id, username, email FROM users WHERE id = $1`. We bind our parameter using the `bind` method. Never, under any circumstances, interpolate user variables into a query string using Rust's `format!` macro. When you use positional parameter binding, SQLx sends the query template and the parameter values separately over the PostgreSQL wire protocol. PostgreSQL prepares the execution plan before substituting the values, making SQL injection mathematically impossible.

### Part 5
Next, look at the execution methods. SQLx provides `fetch_one`, `fetch_optional`, `fetch_all`, and `fetch`. `fetch_one` expects exactly one row; if zero rows or multiple rows return, it yields an error. `fetch_optional` returns an `Option` wrapping your struct, making it ideal for lookups by unique identifier where a missing record represents a normal `None` condition rather than an unexpected failure. `fetch_all` returns a `Vec` of structs. And `fetch` returns an asynchronous stream of rows, allowing you to process millions of records without buffering them into memory.

### Part 6
Now let us look at `FromRow` derive attributes in our model definitions. If your database uses snake_case column names and your Rust struct fields need custom mappings, you can annotate fields with `#[sqlx(rename = "...")]`. If you have shared sub-structures—such as an address struct embedded inside a user profile—you can annotate the field with `#[sqlx(flatten)]`. SQLx will flatten the nested fields into the top-level row mapping. You can also use `#[sqlx(skip)]` to ignore fields that are computed in memory, and `#[sqlx(default)]` to provide fallback values when a column is missing from the result set.

---

## COMPILE-TIME MACROS, OFFLINE MODE, AND DYNAMIC SQL

### Part 1
Now we come to one of SQLx's most famous superpowers: the `query!` and `query_as!` macros. Open `src/queries/macros.rs`. When you write a query using these macros, SQLx does something remarkable during `cargo build`. It opens a connection to your local development database, sends the SQL query to PostgreSQL using the PREPARE command, and asks PostgreSQL to inspect the query against the live database catalog.

### Part 2
PostgreSQL verifies that the table names exist, that every referenced column exists, and that the parameter types match. Even better, PostgreSQL tells SQLx the exact data types and nullability of the returned columns. If you misspell a column name or try to bind an integer to a text parameter, your code will fail to compile right inside your terminal. You get type safety for raw SQL without any code generation or ORM overhead.

### Part 3
However, this compile-time checking introduces a problem: what happens when your continuous integration server builds your application without access to a running PostgreSQL database? Or what happens when building a minimal Docker image? The build will fail because SQLx cannot reach a database. This is where offline mode comes in.

### Part 4
To enable offline mode, we run `cargo sqlx prepare`. SQLx executes every macro in your codebase against your development database, captures the query definitions, parameter types, and result schemas, and saves them into JSON files inside a `.sqlx` directory. You commit this `.sqlx` directory to your Git repository. In your CI pipeline or Docker build, you set the environment variable `SQLX_OFFLINE=true`. SQLx reads the saved JSON files instead of contacting a live database, giving you full compile-time verification in completely air-gapped environments.

### Part 5
Now let us examine dynamic SQL queries. Sometimes your application needs to filter results based on optional search criteria—a user might filter by country, or status, or date range, or any combination of them. Compile-time macros cannot help here because the query structure changes at runtime. Older tutorials would resort to string concatenation, risking SQL injection. In SQLx, we use `QueryBuilder`.

### Part 6
Look at `src/queries/dynamic.rs`. We instantiate a `QueryBuilder` for Postgres. We start with our base query. Then, using conditional logic, we push additional SQL clauses and bind parameters. SQLx automatically numbers the bind parameters correctly, turning our inputs into `$1`, `$2`, and `$3`. Furthermore, SQLx 0.9 introduces the `AssertSqlSafe` trait. If you need to push a dynamic column identifier for sorting, you must explicitly declare that the identifier is safe, ensuring that arbitrary user strings cannot be injected into structural SQL positions.

---

## ADVANCED POSTGRES TYPES IN RUST

### Part 1
PostgreSQL has one of the richest type systems of any database engine. Mapping those types into idiomatic Rust requires understanding how SQLx handles serialization and deserialization. Open `src/types/custom.rs`. We will walk through UUIDs, high-precision decimals, timestamps, custom enums, JSONB, and native arrays.

### Part 2
First, UUIDs. We use the `uuid` crate. In modern distributed systems, using sequential integers for external identifiers exposes business metrics and invites enumeration attacks. We use UUID version 7. Unlike traditional UUID version 4, which is purely random, UUID version 7 embeds a millisecond-precision Unix timestamp in its most significant bits. This makes UUID v7 monotonically increasing, preserving B-tree index locality and dramatically reducing page fragmentation during heavy inserts.

### Part 3
Next, financial values and precision numbers. Never use `f32` or `f64` for money. Floating-point arithmetic suffers from binary rounding errors that will inevitably corrupt account balances. In PostgreSQL, we store money as `NUMERIC`. In Rust, we map this to the `Decimal` type from the `rust_decimal` crate. SQLx handles the wire conversion losslessly, guaranteeing exact decimal arithmetic for every transaction.

### Part 4
Now look at timestamps. We use `DateTime<Utc>` from the `chrono` crate, mapped to PostgreSQL's `TIMESTAMPTZ`. Always store timestamps in UTC at the database level. Converting to local time zones should only happen at the user interface presentation layer. Storing naive timestamps without time zone information makes it impossible to order events accurately across distributed systems.

### Part 5
Next, custom database enums. In PostgreSQL, we can define an enum type with `CREATE TYPE user_role AS ENUM ('admin', 'moderator', 'member')`. In Rust, we define a matching enum and derive `sqlx::Type`. We annotate the struct with `#[sqlx(type_name = "user_role", rename_all = "snake_case")]`. SQLx maps the database enum values to your Rust enum variants at compile time, rejecting invalid strings before they ever reach your application logic.

### Part 6
Finally, JSONB and native arrays. Look at our user profile column. It is typed as `JSONB`. In Rust, we define a strongly typed `UserProfile` struct deriving `Serialize` and `Deserialize`, and wrap it in `sqlx::types::Json<UserProfile>`. SQLx serializes and deserializes the JSON payload automatically. For lists of tags or identifiers, PostgreSQL supports native arrays like `TEXT[]`. In Rust, this maps cleanly to `Vec<String>`. You can use array operators like containment and overlaps directly in your SQL queries.

---

## JOINS, SUBQUERIES, AND AGGREGATES

### Part 1
Now let us master SQL querying in depth. Many developers write dozens of tiny individual queries in Rust and join the data in memory. This is an anti-pattern: it floods your network with round trips, ignores database indexes, and wastes CPU cycles. Let the database do what it was engineered to do. Open `src/queries/joins.rs`.

### Part 2
We begin with multi-table joins. An `INNER JOIN` returns rows only when there is a match in both tables. A `LEFT JOIN` returns all rows from the left table, padding missing right-table columns with NULL. Look at our Rust struct mapping for a `LEFT JOIN`: because right-table columns can be NULL when no matching record exists, those fields in our Rust struct MUST be wrapped in `Option<T>`. If you declare them as non-optional, SQLx will return a runtime error whenever an unmatched row is encountered.

### Part 3
Next, self-joins. A self-join joins a table to itself. This is the classic pattern for modeling hierarchical data, such as employees and their managers, or categories and subcategories within the same table. We assign distinct table aliases, like `e` for employee and `m` for manager, and link them on the manager identifier.

### Part 4
Now let us inspect subqueries. We look at scalar subqueries, which return a single value, and correlated subqueries, where the inner query references values from the outer query. Look at `WHERE EXISTS`. When checking whether a user has placed any orders, `WHERE EXISTS` is significantly faster than `WHERE id IN (SELECT user_id FROM orders)`. `EXISTS` halts evaluation the moment the very first matching row is found in the index, whereas `IN` can force PostgreSQL to evaluate and buffer the entire subquery result set.

### Part 5
Next are set operations: `UNION`, `UNION ALL`, `INTERSECT`, and `EXCEPT`. The critical distinction is between `UNION` and `UNION ALL`. `UNION` removes duplicate rows by sorting and deduplicating the combined result set in memory, consuming CPU and memory. `UNION ALL` simply concatenates the two streams without deduplication. Unless you explicitly require duplicate elimination, always default to `UNION ALL`.

### Part 6
Now let us examine aggregates and modern PostgreSQL filtering. Beyond basic `COUNT`, `SUM`, and `AVG`, look at the `FILTER` clause: `COUNT(*) FILTER (WHERE status = 'completed')`. Older SQL patterns required clunky `CASE WHEN` statements inside sums. The `FILTER` clause allows you to compute multiple conditional aggregates in a single, elegant scan of the table. We also demonstrate `ROLLUP` and `GROUPING SETS` for generating multi-level analytical subtotals in a single query.

---

## ADVANCED SQL: CTES, WINDOW FUNCTIONS, LATERAL, AND DISTINCT ON

### Part 1
Now we step into advanced SQL territory. These are the tools that separate junior developers from senior database engineers. Open `src/queries/advanced.rs`. We begin with Common Table Expressions, or CTEs, declared using the `WITH` clause.

### Part 2
CTEs allow you to break complex, convoluted queries into readable, named logical steps. In PostgreSQL 12 and later, standard CTEs are automatically inlined by the query planner unless you explicitly mark them with the `MATERIALIZED` keyword. Look at recursive CTEs: a recursive CTE consists of an anchor member and a recursive member joined by `UNION ALL`. This is how you traverse tree structures—such as nested category taxonomies or organizational charts—in a single database query without recursion in application code.

### Part 3
Next, window functions. A standard aggregate collapses multiple rows into a single summary row. A window function, by contrast, computes calculations across a set of rows while preserving the identity of every individual row. Look at `ROW_NUMBER()`, `RANK()`, and `DENSE_RANK()`. We use them with the `OVER (PARTITION BY ... ORDER BY ...)` clause to rank items within their category.

### Part 4
Notice how we calculate running totals: `SUM(amount) OVER (PARTITION BY user_id ORDER BY created_at)`. PostgreSQL tracks the cumulative sum for each user, ordering by transaction date, without needing self-joins or secondary loops. We also demonstrate `LEAD` and `LAG`, which access values from the next or previous row in the partition. This makes calculating month-over-month revenue growth trivial.

### Part 5
Now examine `LATERAL` joins. A `LATERAL` join behaves like a `for-each` loop in SQL. It allows the subquery on the right side of the join to reference columns from rows yielded by the left side. Look at our query fetching the latest three orders for every active user: by joining users to a lateral subquery that selects orders ordered by date descending with a limit of three, PostgreSQL uses our composite index to pluck the exact three rows per user instantly.

### Part 6
Finally, look at `DISTINCT ON`. This is a unique PostgreSQL feature that does not exist in standard SQL or MySQL. `SELECT DISTINCT ON (user_id) * FROM orders ORDER BY user_id, created_at DESC`. PostgreSQL groups rows by `user_id` and keeps only the first row according to the order clause—which is the most recent order. It eliminates the need for expensive self-joins or subquery filtering when you need the single latest record per entity.

---

## POWERFUL DML: UPSERTS, RETURNING, AND BULK OPERATIONS

### Part 1
Data manipulation is more than simple inserts and updates. Let us look at high-performance DML patterns in `src/queries/dml.rs`. We begin with upserts using `INSERT ... ON CONFLICT`.

### Part 2
When inserting a record that might already exist—such as updating a user's last login timestamp or profile details—handling unique violations in application code causes race conditions. With `ON CONFLICT (username) DO UPDATE SET last_login = EXCLUDED.last_login`, PostgreSQL handles the conflict atomically at the row level. Notice the special `EXCLUDED` table name: it refers to the proposed row that caused the conflict, allowing you to update specific columns cleanly.

### Part 3
Next is the `RETURNING` clause. In many databases, inserting a row requires a secondary query to discover the auto-generated primary key. In PostgreSQL, adding `RETURNING id, created_at` returns those generated values immediately in the response payload. You eliminate an entire network round trip, cutting database latency in half for write-heavy endpoints.

### Part 4
Now let us inspect bulk inserts using `UNNEST`. When inserting thousands of records at once, naive batching generates queries with thousands of bind parameters, easily exceeding PostgreSQL's limit of sixty-five thousand five hundred and thirty-five parameters. With `UNNEST`, we pass entire arrays: an array of usernames, an array of emails, and an array of countries. The SQL query unrolls those arrays into rows using `SELECT * FROM UNNEST($1, $2, $3)`. The query only uses three bind parameters, regardless of whether you insert ten rows or ten thousand rows.

### Part 5
For raw ingestion speed, nothing matches the PostgreSQL `COPY` protocol. With SQLx, we use `copy_in_raw`. Instead of formatting individual SQL statements, SQLx streams raw binary or CSV data directly into the PostgreSQL storage engine over the connection socket. In Part 2, you will see `COPY` load hundreds of millions of rows orders of magnitude faster than standard insert statements.

---

## DATABASE LOGIC: VIEWS, TRIGGERS, AND LISTEN/NOTIFY

### Part 1
Pushing appropriate business logic into PostgreSQL can simplify your application and guarantee system invariants. Open `src/database/logic.rs`. We will walk through standard views, materialized views, generated columns, triggers, and asynchronous notifications.

### Part 2
First, views. A view is a stored query that acts as a virtual table. It encapsulates complex joins and calculations, providing a clean abstraction for your application code. Materialized views go a step further: they physically store the computed results on disk. When querying complex analytical dashboards, a materialized view answers queries in milliseconds. To refresh it without locking out readers, we create a unique index on the materialized view and execute `REFRESH MATERIALIZED VIEW CONCURRENTLY`.

### Part 3
Next, generated columns. Look at our total price column: `total NUMERIC GENERATED ALWAYS AS (quantity * unit_price) STORED`. PostgreSQL calculates and stores this value automatically whenever quantity or unit price changes. It guarantees that the computed column never drifts out of sync with its source inputs, and allows you to index the generated value directly.

### Part 4
Now let us inspect triggers and PL/pgSQL functions. Look at our automated timestamp trigger: we define a trigger function that sets `NEW.updated_at = now()`, and attach a `BEFORE UPDATE` trigger to our tables. Every time a row is modified—whether through your web application, a background CLI tool, or a manual database console—the update timestamp is refreshed accurately.

### Part 5
Finally, real-time messaging with `LISTEN` and `NOTIFY`. PostgreSQL includes a built-in publish-subscribe messaging bus. In Rust, SQLx provides the `PgListener` type. We configure `PgListener` to subscribe to a channel named `order_events`. Whenever a trigger or transaction executes `pg_notify`, SQLx receives the notification asynchronously over the connection socket and yields it as an event in Rust. You get a lightweight event bus without needing Redis or RabbitMQ for simple messaging needs.

---

## TRANSACTIONS, ISOLATION LEVELS, AND LOCKING

### Part 1
Now we arrive at the heart of relational database safety: transactions and concurrency control. Open `src/transactions/lifecycle.rs`. When multiple users interact with your database simultaneously, naive code leads to race conditions, phantom reads, and deadlocks.

### Part 2
Let us look at transaction management in SQLx. We call `pool.begin().await` to acquire a connection and start a transaction. We perform our mutations, and call `tx.commit().await` to commit the changes. But look at what happens if an error occurs: SQLx transactions implement the `Drop` trait. If a function returns early due to an error before `commit` is called, the transaction object is dropped, and SQLx automatically issues a `ROLLBACK` over the connection. Your database state is never left partially modified.

### Part 3
Next, savepoints and nested transactions. In SQLx, calling `tx.begin().await` on an existing transaction does not open a second physical transaction; instead, it issues a `SAVEPOINT`. If a sub-operation fails, you can roll back to the savepoint without aborting the entire enclosing transaction. This is essential when attempting speculative operations that might fail without invalidating the broader business workflow.

### Part 4
Now let us understand isolation levels. PostgreSQL supports `READ COMMITTED`, `REPEATABLE READ`, and `SERIALIZABLE`. `READ COMMITTED` is the default: each statement sees only rows committed before the statement began. But under `READ COMMITTED`, two consecutive reads within the same transaction can see different data if another transaction commits in between—this is a non-repeatable read. In `REPEATABLE READ`, the entire transaction sees a snapshot of the database taken at the start of the transaction. And in `SERIALIZABLE`, PostgreSQL guarantees that concurrent transactions execute as if they ran in strict sequential order, aborting transactions with serialization failures if conflicting read-write dependencies occur.

### Part 5
Now let us examine row-level locking. When updating a balance or reserving inventory, reading a value and updating it later introduces a race condition. Look at `SELECT balance FROM accounts WHERE id = $1 FOR UPDATE`. `FOR UPDATE` places an exclusive row-level lock on the selected row. Any concurrent transaction attempting to read that row with `FOR UPDATE` will block until your transaction commits or rolls back, guaranteeing consistency.

### Part 6
Let us talk about deadlocks. A deadlock occurs when Transaction A locks Row 1 and waits for Row 2, while Transaction B locks Row 2 and waits for Row 1. Neither can proceed. PostgreSQL runs a deadlock detection cycle in the background and aborts one of the transactions. We demonstrate two solutions: first, always acquiring locks in a consistent primary key order across all application endpoints; and second, using optimistic concurrency control with a version column and compare-and-swap semantics.

---

## APPLICATION PATTERNS: THE EXECUTOR TRAIT AND CLEAN REPOSITORIES

### Part 1
How do you structure database code in a real-world production application? Many Rust codebases suffer from a common architectural flaw: repository functions take `&PgPool`, making it impossible to run multiple repository calls within a single transaction without writing duplicated code. Open `src/repository/user_repo.rs`.

### Part 2
Let us inspect the `Executor` trait. SQLx defines `PgExecutor<'c>`, a trait implemented by `&PgPool`, `&mut PgConnection`, and `&mut PgTransaction`. By writing repository functions that accept any type implementing `PgExecutor`, a single repository function can be called with a connection pool for standalone reads, or with an active transaction handle when part of an atomic multi-step workflow.

### Part 3
Look at the signature on screen: `pub async fn find_by_id<'e, E>(executor: E, id: i64) -> Result<Option<User>, RepoError> where E: PgExecutor<'e>`. Look how clean this is. The caller decides whether to pass `&pool` or `&mut *tx`. There is zero code duplication, and transaction boundaries remain completely decoupled from data access logic.

### Part 4
Now let us look at error handling. When PostgreSQL encounters an error, SQLx returns `sqlx::Error::Database`. Treating all database errors as generic five hundred internal server errors degrades user experience. Inside our error mapping layer, we inspect the underlying database error and extract the PostgreSQL SQLSTATE error code.

### Part 5
Look at our error translation: error code `23505` represents a unique constraint violation. When we see `23505`, we inspect the constraint name. If the constraint is `users_username_key`, we map it to our domain error `UserError::UsernameAlreadyTaken`. If code `23503` occurs, we map it to `UserError::UserNotFound` because a foreign key failed. Clean error mapping keeps database details encapsulated and lets your API layer return clear, actionable HTTP status codes.

---

## SECURITY, ROW-LEVEL SECURITY, TESTING, AND RECAP

### Part 1
Security must be woven into your architecture from the start. Open `src/security/rls.rs`. Many applications connect to PostgreSQL using the `postgres` superuser. That is a critical security vulnerability: a single injection vulnerability gives attackers complete control over the entire database engine.

### Part 2
Always create dedicated application roles with the principle of least privilege. Grant only `SELECT`, `INSERT`, `UPDATE`, and `DELETE` on the specific tables the service requires. For multi-tenant applications, PostgreSQL provides Row-Level Security, or RLS. We execute `ALTER TABLE documents ENABLE ROW LEVEL SECURITY`, and define a security policy: `CREATE POLICY tenant_isolation_policy ON documents USING (tenant_id = current_setting('app.current_tenant_id')::uuid)`.

### Part 3
When our Rust application acquires a connection from the pool to handle a tenant request, it executes `SET LOCAL app.current_tenant_id = $1`. Once set, PostgreSQL's query engine automatically filters all queries on that connection to match the tenant identifier. Even if an engineer writes `SELECT * FROM documents` without a WHERE clause, PostgreSQL will physically refuse to return rows belonging to another tenant.

### Part 4
Now let us look at automated testing. Open `tests/integration_test.rs`. SQLx provides the `#[sqlx::test]` attribute macro. When you annotate a test with `#[sqlx::test]`, SQLx connects to PostgreSQL, creates a completely isolated, temporary test database, applies all your migrations automatically, and injects a dedicated connection pool into your test function.

### Part 5
Every integration test runs against a clean, isolated database schema. Tests can run concurrently without stepping on each other's data. You can also specify fixture files to pre-seed test data. When the test completes, SQLx tears down the temporary database automatically. You get fast, deterministic, reproducible database tests with zero manual cleanup.

### Part 6
Let us look back at what we have accomplished. We started with Docker and SQLx 0.9 setup, mastered schema constraints, conquered queries, compile-time macros, offline preparation, advanced Postgres types, joins, CTEs, window functions, upserts, database logic, transactions, locking, clean repository architectures, row-level security, and test harnesses. 

If this deep dive helped you write cleaner, safer Rust database code, subscribe to Aarambh Dev Hub and leave a comment with the features you want to see next. And now, you are fully equipped for Part 2: where we take all of these foundations and test them against one hundred crore rows in PostgreSQL. I will see you in Part 2.
