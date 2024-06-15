use futures::{stream::BoxStream, TryStreamExt};
use sqlx::{sqlite::SqliteConnectOptions, Connection};
use std::{env, ops::Range};
use tokio::fs;

use crate::{levenshtein, LineOut, NeoDebug, RTM};

pub struct Database(sqlx::SqliteConnection);

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

            let mut conn = sqlx::SqliteConnection::connect_with(&opts).await?;

            sqlx::query(
                "CREATE TABLE IF NOT EXISTS all_lines (
                    id          INTEGER PRIMARY KEY,
                    text        TEXT NOT NULL,
                    icon        TEXT NOT NULL,
                    hl_group    TEXT NOT NULL
                )",
            )
            .execute(&mut conn)
            .await?;

            sqlx::query(
                "CREATE TABLE IF NOT EXISTS sorted_lines (
                    line_id     INTEGER PRIMARY KEY,
                    score       INTEGER NOT NULL
                )",
            )
            .execute(&mut conn)
            .await?;

            Ok(Self(conn))
        })
    }

    /// TODO stream instead
    pub async fn select(&mut self, search_query: &str) -> sqlx::Result<Vec<LineOut>> {
        // Select only (id, text) -> then stream apply levenstein score.
        // Immediately push back into database into table sorted_lines.
        // Sort by score and join all_lines with sorted_lines, return LineOut
        // profit
        let mut out = vec![];

        {
            let mut stream = sqlx::query_as::<_, (u32, String)>(
                "SELECT id, text FROM all_lines WHERE text like ?",
            )
            .bind(search_query)
            .fetch(&mut self.0);

            while let Ok(Some(line)) = stream.try_next().await {
                let score = levenshtein(search_query, &line.1) as u32;
                //self.insert_sorted_line(line.id, score).await?;
                out.push((line.0, score));
            }
        }

        NeoDebug::log(format!("len: {}", out.len())).await;

        sqlx::query("DELETE FROM sorted_lines")
            .execute(&mut self.0)
            .await?;

        for item in out.iter() {
            sqlx::query("INSERT into sorted_lines (line_id, score) VALUES (?, ?)")
                .bind(item.0)
                .bind(item.1)
                .execute(&mut self.0)
                .await?;
        }

        let lines = sqlx::query_as::<_, LineOut>(
            "SELECT id, text, icon, hl_group FROM all_lines INNER JOIN sorted_lines ON
            sorted_lines.line_id = all_lines.id ORDER BY score",
        )
        .bind(search_query)
        .fetch_all(&mut self.0)
        .await?;

        Ok(lines)
    }

    /// TODO insert stream wise one element at a time.
    pub async fn insert_all(&mut self, lines: &[LineOut]) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM all_lines")
            .execute(&mut self.0)
            .await?;

        for line in lines {
            let _stmt =
                sqlx::query("INSERT into all_lines (text, icon, hl_group) VALUES (?1, ?2, ?3)")
                    .bind(&line.text)
                    .bind(&line.icon)
                    .bind(&line.hl_group)
                    .execute(&mut self.0)
                    .await?;
        }

        Ok(())
    }
}
