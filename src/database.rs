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

            sqlx::query(
                "CREATE TABLE IF NOT EXISTS sorted_lines (
                    line_id     INTEGER PRIMARY KEY,
                    score       INTEGER NOT NULL
                )",
            )
            .execute(&pool)
            .await?;

            Ok(Self(pool))
        })
    }

    /// TODO stream instead
    pub async fn select(
        &mut self,
        search_query: &str,
        instant: &Instant,
    ) -> sqlx::Result<Vec<LineOut>> {
        // Select only (id, text) -> then stream apply levenstein score.
        // Immediately push back into database into table sorted_lines.
        // Sort by score and join all_lines with sorted_lines, return LineOut
        // profit

        if search_query == "%" {
            return sqlx::query_as::<_, LineOut>("SELECT * FROM all_lines LIMIT 300")
                .fetch_all(&self.0)
                .await;
        }

        let bef_elapsed_ms = instant.elapsed().as_millis();

        let mut stream =
            sqlx::query_as::<_, (u32, String)>("SELECT id, text FROM all_lines WHERE text like ?")
                .bind(search_query)
                .fetch(&self.0);

        let aft_elapsed_ms = instant.elapsed().as_millis();
        NeoDebug::log(format!("select time: {}", aft_elapsed_ms - bef_elapsed_ms)).await;

        let mut out = vec![];

        while let Ok(Some(line)) = stream.try_next().await {
            let score = levenshtein(search_query, &line.1) as u32;
            out.push((line.0, score));
        }

        drop(stream);

        //NeoDebug::log(format!("len: {}", out.len())).await;
        let bef_elapsed_ms = instant.elapsed().as_millis();

        sqlx::query("DELETE FROM sorted_lines")
            .execute(&self.0)
            .await?;

        let aft_elapsed_ms = instant.elapsed().as_millis();
        NeoDebug::log(format!("delete time: {}", aft_elapsed_ms - bef_elapsed_ms)).await;

        let bef_elapsed_ms = instant.elapsed().as_millis();

        let mut tx = self.0.begin().await?;

        for item in out.iter() {
            sqlx::query("INSERT into sorted_lines (line_id, score) VALUES (?, ?)")
                .bind(item.0)
                .bind(item.1)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        let aft_elapsed_ms = instant.elapsed().as_millis();
        NeoDebug::log(format!("insert time: {}", aft_elapsed_ms - bef_elapsed_ms)).await;

        let bef_elapsed_ms = instant.elapsed().as_millis();

        let lines = sqlx::query_as::<_, LineOut>(
            "SELECT id, text, icon, hl_group FROM all_lines INNER JOIN sorted_lines ON
            sorted_lines.line_id = all_lines.id ORDER BY score LIMIT 300",
        )
        .fetch_all(&self.0)
        .await?;

        let aft_elapsed_ms = instant.elapsed().as_millis();
        NeoDebug::log(format!(
            "final select time: {}",
            aft_elapsed_ms - bef_elapsed_ms
        ))
        .await;

        Ok(lines)
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
