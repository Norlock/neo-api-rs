use sqlx::sqlite::SqliteConnectOptions;
use std::env;
use tokio::{fs, time::Instant};

use crate::{LineOut, NeoDebug, RTM};

pub struct Database(sqlx::SqlitePool);

pub trait DatabaseQuery {
    fn insert(&self) -> String;
}

impl Database {
    pub fn init() -> sqlx::Result<Self> {
        RTM.block_on(async {
            let dir = env::temp_dir().join("neo-api-rs");
            fs::create_dir_all(&dir).await?;

            let file = dir.join("fuzzy.db");
            fs::write(&file, []).await?;
            let opts = SqliteConnectOptions::new().filename(file);

            let pool = sqlx::SqlitePool::connect_with(opts).await?;

            sqlx::query(
                "CREATE TABLE IF NOT EXISTS all_lines (
                    id          INTEGER PRIMARY KEY,
                    text        TEXT NOT NULL,
                    icon        TEXT NOT NULL,
                    hl_group    TEXT NOT NULL
                )",
            )
            .execute(&pool)
            .await?;

            sqlx::query(
                "CREATE VIRTUAL TABLE IF NOT EXISTS all_lines_fts USING fts5 (
                    id, text
                )",
            )
            .execute(&pool)
            .await?;

            Ok(Self(pool))
        })
    }

    pub async fn select(
        &self,
        search_query: &str,
        instant: &Instant,
    ) -> sqlx::Result<Vec<LineOut>> {
        let bef_elapsed_ms = instant.elapsed().as_millis();

        let out = sqlx::query_as::<_, LineOut>(
            "SELECT 
                * 
            FROM 
                all_lines a
                INNER JOIN all_lines_fts f ON a.id = f.id
            WHERE 
                a.text LIKE ? ORDER BY bm25(all_lines_fts, 0, 1) LIMIT 300",
        )
        .bind(search_query)
        .fetch_all(&self.0)
        .await?;

        let aft_elapsed_ms = instant.elapsed().as_millis();
        NeoDebug::log(format!("select time: {}", aft_elapsed_ms - bef_elapsed_ms)).await;

        Ok(out)
    }

    pub async fn insert_all(&mut self, lines: &[LineOut]) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM all_lines_fts")
            .execute(&self.0)
            .await?;

        sqlx::query("DELETE FROM all_lines")
            .execute(&self.0)
            .await?;

        let mut tx = self.0.begin().await?;

        for (i, line) in lines.iter().enumerate() {
            let _stmt =
                sqlx::query("INSERT into all_lines (id, text, icon, hl_group) VALUES (?, ?, ?, ?)")
                    .bind(i as u32)
                    .bind(&line.text)
                    .bind(&line.icon)
                    .bind(&line.hl_group)
                    .execute(&mut *tx)
                    .await?;

            sqlx::query("INSERT into all_lines_fts (id, text) VALUES (?, ?)")
                .bind(i as u32)
                .bind(&line.text)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await
    }
}
