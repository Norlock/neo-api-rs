use std::path::PathBuf;
use tokio::{fs, sync::Mutex, time::Instant};

use crate::{LineOut, NeoDebug, RTM};

pub struct Database(sqlx::SqlitePool);

impl Database {
    pub fn new() -> Mutex<Self> {
        RTM.block_on(async move {
            Mutex::new({
                let result = Database::init().await;

                if let Err(err) = result {
                    NeoDebug::log(err.to_string()).await;
                    panic!("");
                };

                result.unwrap()
            })
        })
    }

    pub async fn init() -> sqlx::Result<Self> {
        let tmp = std::env::temp_dir().join("neo-api-rs");
        let file = tmp.join("fuzzy.db");

        fs::create_dir_all(&tmp).await?;
        fs::write(&file, []).await?;

        let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/extensions/fuzzy")
            .to_string_lossy()
            .to_string();

        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(file)
            .pragma("journal_mode", "MEMORY")
            .pragma("case_sensitive_like", "ON")
            .extension(crate_path);

        let pool = sqlx::SqlitePool::connect_with(options).await?;

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
    }

    pub async fn select(
        &self,
        sql_search_query: &str,
        search_query: &str,
        instant: &Instant,
    ) -> sqlx::Result<Vec<LineOut>> {
        let bef_elapsed_ms = instant.elapsed().as_millis();

        let out = sqlx::query_as::<_, LineOut>(
            "SELECT 
                * 
            FROM 
                all_lines 
            WHERE text like ? ORDER BY fuzzy_score(?, text)
            LIMIT 300",
        )
        .bind(sql_search_query)
        .bind(search_query)
        .fetch_all(&self.0)
        .await?;

        let aft_elapsed_ms = instant.elapsed().as_millis();
        NeoDebug::log(format!("select time: {}", aft_elapsed_ms - bef_elapsed_ms)).await;

        Ok(out)
    }

    pub async fn clean_up_tables(&self) -> sqlx::Result<()> {
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

            for i in 0..chunks.len() {
                if i == 0 {
                    qry_str.push_str("(?, ?, ?, ?)");
                } else {
                    qry_str.push_str(", (?, ?, ?, ?)");
                }
            }

            let mut query = sqlx::query(&qry_str);

            for line in chunks {
                query = query
                    .bind(idx)
                    .bind(&line.text)
                    .bind(&line.icon)
                    .bind(&line.hl_group);

                idx += 1;
            }

            query.execute(&mut *tx).await?;
        }

        tx.commit().await
    }
}
