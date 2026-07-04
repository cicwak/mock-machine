use anyhow::Context;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement, TransactionTrait};
use tracing::info;

struct Migration {
    version: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: "0001_initial_schema",
        sql: include_str!("../migrations/0001_initial_schema.sql"),
    },
    Migration {
        version: "0002_seed",
        sql: include_str!("../migrations/0002_seed.sql"),
    },
    Migration {
        version: "0003_profiles",
        sql: include_str!("../migrations/0003_profiles.sql"),
    },
    Migration {
        version: "0004_projects",
        sql: include_str!("../migrations/0004_projects.sql"),
    },
    Migration {
        version: "0005_project_keys",
        sql: include_str!("../migrations/0005_project_keys.sql"),
    },
    Migration {
        version: "0006_project_default_proxy_url",
        sql: include_str!("../migrations/0006_project_default_proxy_url.sql"),
    },
];

pub async fn run(db: &DatabaseConnection) -> anyhow::Result<()> {
    db.execute_unprepared("SELECT pg_advisory_lock(hashtext('mock-machine-schema-migrations'))")
        .await
        .context("failed to acquire migration advisory lock")?;

    let result = run_locked(db).await;

    let unlock_result = db
        .execute_unprepared("SELECT pg_advisory_unlock(hashtext('mock-machine-schema-migrations'))")
        .await
        .context("failed to release migration advisory lock");

    result.and(unlock_result.map(|_| ()))
}

async fn run_locked(db: &DatabaseConnection) -> anyhow::Result<()> {
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )
        "#,
    )
    .await
    .context("failed to create schema_migrations table")?;

    baseline_existing_schema(db).await?;

    for migration in MIGRATIONS {
        if is_applied(db, migration.version).await? {
            continue;
        }

        let tx = db
            .begin()
            .await
            .with_context(|| format!("failed to begin migration {}", migration.version))?;

        tx.execute_unprepared(migration.sql)
            .await
            .with_context(|| format!("failed to apply migration {}", migration.version))?;
        tx.execute(Statement::from_sql_and_values(
            tx.get_database_backend(),
            "INSERT INTO schema_migrations (version) VALUES ($1)",
            vec![migration.version.into()],
        ))
        .await
        .with_context(|| format!("failed to record migration {}", migration.version))?;

        tx.commit()
            .await
            .with_context(|| format!("failed to commit migration {}", migration.version))?;
        info!(version = migration.version, "applied database migration");
    }

    Ok(())
}

async fn baseline_existing_schema(db: &DatabaseConnection) -> anyhow::Result<()> {
    if !table_exists(db, "mock_routes").await? {
        return Ok(());
    }

    for version in ["0001_initial_schema", "0002_seed"] {
        db.execute(Statement::from_sql_and_values(
            db.get_database_backend(),
            r#"
            INSERT INTO schema_migrations (version)
            VALUES ($1)
            ON CONFLICT (version) DO NOTHING
            "#,
            vec![version.into()],
        ))
        .await
        .with_context(|| format!("failed to baseline migration {version}"))?;
    }

    Ok(())
}

async fn is_applied(db: &DatabaseConnection, version: &str) -> anyhow::Result<bool> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            db.get_database_backend(),
            "SELECT 1 FROM schema_migrations WHERE version = $1",
            vec![version.into()],
        ))
        .await
        .with_context(|| format!("failed to check migration {version}"))?;

    Ok(row.is_some())
}

async fn table_exists(db: &DatabaseConnection, table_name: &str) -> anyhow::Result<bool> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            db.get_database_backend(),
            r#"
            SELECT 1
            FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = $1
            "#,
            vec![table_name.into()],
        ))
        .await
        .with_context(|| format!("failed to check table {table_name}"))?;

    Ok(row.is_some())
}
