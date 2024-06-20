use sqlx::sqlite::SqliteConnectOptions;
use std::env;
use tokio::{fs, time::Instant};

use crate::{LineOut, NeoDebug};

pub struct Database(sqlx::SqlitePool);

impl Database {
    pub async fn init() -> sqlx::Result<Self> {
        let dir = env::temp_dir().join("neo-api-rs");
        fs::create_dir_all(&dir).await?;

        // TODO multiple db files pre cached
        let file = dir.join("fuzzy.db");
        fs::write(&file, []).await?;
        let opts = SqliteConnectOptions::new()
            .filename(file)
            .pragma("journal_mode", "MEMORY");

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
    }

    pub async fn select(
        &self,
        search_query: &str,
        instant: &Instant,
    ) -> sqlx::Result<Vec<LineOut>> {
        let bef_elapsed_ms = instant.elapsed().as_millis();

        let out = if search_query == "" {
            sqlx::query_as::<_, LineOut>("SELECT * FROM all_lines LIMIT 300")
                .bind(search_query)
                .fetch_all(&self.0)
                .await?
        } else {
            sqlx::query_as::<_, LineOut>(
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
            .await?
        };

        let aft_elapsed_ms = instant.elapsed().as_millis();
        NeoDebug::log(format!("select time: {}", aft_elapsed_ms - bef_elapsed_ms)).await;

        Ok(out)
    }

    pub async fn clean_up_tables(&self) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM all_lines_fts")
            .execute(&self.0)
            .await?;

        sqlx::query("DELETE FROM all_lines")
            .execute(&self.0)
            .await?;

        Ok(())
    }

    pub async fn insert_all(&self, lines: &[LineOut]) -> sqlx::Result<()> {
        let mut tx = self.0.begin().await?;
        let mut idx = 0;

        for chunks in lines.chunks(1000) {
            let mut qry_str = "INSERT INTO all_lines (id, text, icon, hl_group) VALUES".to_string();
            let mut qry_fts_str = "INSERT INTO all_lines_fts (id, text) VALUES".to_string();

            for i in 0..chunks.len() {
                if i == 0 {
                    qry_str.push_str("(?, ?, ?, ?)");
                    qry_fts_str.push_str("(?, ?)");
                } else {
                    qry_str.push_str(", (?, ?, ?, ?)");
                    qry_fts_str.push_str(", (?, ?)");
                }
            }

            let mut query = sqlx::query(&qry_str);
            let mut query_fts = sqlx::query(&qry_fts_str);

            for line in chunks {
                query = query
                    .bind(idx)
                    .bind(&line.text)
                    .bind(&line.icon)
                    .bind(&line.hl_group);

                query_fts = query_fts.bind(idx).bind(&line.text);

                idx += 1;
            }

            query.execute(&mut *tx).await?;
            query_fts.execute(&mut *tx).await?;
        }

        tx.commit().await
    }
}
