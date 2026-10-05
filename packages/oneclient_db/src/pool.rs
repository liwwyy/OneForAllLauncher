use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use crate::DbError;

pub type DbPool = SqlitePool;

// Bundle installs write metadata concurrently; slow disks and CPU pressure can
// hold a writer longer than five seconds. Wait before dropping an installed mod.
const BUSY_TIMEOUT: Duration = Duration::from_secs(30);

#[tracing::instrument(
    skip(database_path),
    fields(database_path = %database_path.as_ref().display())
)]
pub async fn connect(database_path: impl AsRef<Path>) -> Result<DbPool, DbError> {
    let database_path = database_path.as_ref();
    if let Some(parent) = database_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let options = SqliteConnectOptions::from_str(&format!(
        "sqlite://{}?mode=rwc",
        dunce::canonicalize(database_path)
            .unwrap_or_else(|_| database_path.to_path_buf())
            .display()
    ))?
    .create_if_missing(true)
    .journal_mode(SqliteJournalMode::Wal)
    .busy_timeout(BUSY_TIMEOUT);

    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .acquire_timeout(Duration::from_secs(60))
        .connect_with(options)
        .await?;

    crate::migrate::run(&pool, database_path).await?;

    tracing::info!("database ready");

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_contended_write_can_outlast_five_seconds() {
        let dir = async_tempfile::TempDir::new().await.unwrap();
        let pool = connect(dir.dir_path().join("contention.db")).await.unwrap();
        sqlx::query("CREATE TABLE contention (value INTEGER)")
            .execute(&pool)
            .await
            .unwrap();

        let mut writer = pool.acquire().await.unwrap();
        sqlx::query("BEGIN IMMEDIATE")
            .execute(&mut *writer)
            .await
            .unwrap();
        sqlx::query("INSERT INTO contention VALUES (1)")
            .execute(&mut *writer)
            .await
            .unwrap();

        // Acquire a second connection before starting the timer, so the write is
        // actually contending with the first transaction rather than waiting for a pool slot.
        let mut waiting = pool.acquire().await.unwrap();
        let (result, release) = tokio::join!(
            sqlx::query("INSERT INTO contention VALUES (2)").execute(&mut *waiting),
            async {
                tokio::time::sleep(Duration::from_secs(7)).await;
                sqlx::query("COMMIT").execute(&mut *writer).await
            }
        );
        release.unwrap();
        result.expect("a transient writer lock must not drop the second write");
        drop(writer);
        drop(waiting);
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM contention")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 2);
        pool.close().await;
    }
}
