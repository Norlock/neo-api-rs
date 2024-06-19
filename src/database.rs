use std::env;

use futures::TryStreamExt;
use sqlx::sqlite::SqliteConnectOptions;
use tokio::{fs, time::Instant};

use crate::{levenshtein, LineOut, NeoDebug, RTM};

pub struct Database(sqlx::SqlitePool);

pub trait DatabaseQuery {
    fn insert(&self) -> String;
}

impl Database {
    pub fn init() -> sqlx::Result<Self> {
        RTM.block_on(async {
            let dir = env::temp_dir().join("neo-api-rs");
            fs::create_dir_all(&dir).await?;

            let opts = SqliteConnectOptions::new()
                .create_if_missing(true)
                .filename(dir.join("fuzzy.db"));

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

            Ok(Self(pool))
        })
    }

    /// TODO stream instead
    pub async fn select(
        &self,
        search_query: &str,
        instant: &Instant,
    ) -> sqlx::Result<Vec<LineOut>> {
        if search_query == "%" {
            return sqlx::query_as::<_, LineOut>("SELECT * FROM all_lines LIMIT 300")
                .fetch_all(&self.0)
                .await;
        }

        let bef_elapsed_ms = instant.elapsed().as_millis();

        let mut stream = sqlx::query_as::<_, LineOut>("SELECT * FROM all_lines WHERE text like ?")
            .bind(search_query)
            .fetch(&self.0);

        let mut out = vec![];

        while let Ok(Some(line)) = stream.try_next().await {
            let score = levenshtein(search_query, &line.text) as u32;
            out.push((score, line));
        }

        let aft_elapsed_ms = instant.elapsed().as_millis();
        NeoDebug::log(format!("select time: {}", aft_elapsed_ms - bef_elapsed_ms)).await;

        drop(stream);

        let bef_elapsed_ms = instant.elapsed().as_millis();

        out.sort_by_key(|line| line.0);

        let aft_elapsed_ms = instant.elapsed().as_millis();
        NeoDebug::log(format!("sort time: {}", aft_elapsed_ms - bef_elapsed_ms)).await;

        if 300 < out.len() {
            Ok(out.drain(..300).map(|line| line.1).collect())
        } else {
            Ok(out.into_iter().map(|line| line.1).collect())
        }
    }

    /// TODO insert stream wise one element at a time.
    pub async fn insert_all(&mut self, lines: &[LineOut]) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM all_lines")
            .execute(&self.0)
            .await?;

        let mut tx = self.0.begin().await?;

        for line in lines {
            let _stmt =
                sqlx::query("INSERT into all_lines (text, icon, hl_group) VALUES (?1, ?2, ?3)")
                    .bind(&line.text)
                    .bind(&line.icon)
                    .bind(&line.hl_group)
                    .execute(&mut *tx)
                    .await?;
        }

        tx.commit().await
    }
}
