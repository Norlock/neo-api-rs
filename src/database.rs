use sqlx::sqlite::SqliteJournalMode;
use std::{
    env,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::fs;

use crate::{LineOut, NeoDebug, CONTAINER, RTM};

pub struct Database(sqlx::SqlitePool);

impl Database {
    pub fn new() -> Self {
        RTM.block_on(async move {
            let result = Database::init().await;

            if let Err(err) = result {
                NeoDebug::log(err.to_string()).await;
                //std::process::exit(0);
                panic!();
            };

            result.unwrap()
        })
    }

    pub async fn init() -> sqlx::Result<Self> {
        let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/extensions/fuzzy")
            .to_string_lossy()
            .to_string();

        let tmp = env::temp_dir().join("neo-api-rs");

        if let Ok(false) = fs::try_exists(&tmp).await {
            fs::create_dir(&tmp).await?;
        }

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let filename = format!("fuzzy_{}.sqlite", since_the_epoch.as_secs());

        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(tmp.join(filename))
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
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

    pub async fn select(&self, search_query: &str) -> sqlx::Result<Vec<LineOut>> {
        let mut like_query = '%'.to_string();

        for char in search_query.chars() {
            like_query.push(char);
            like_query.push('%');
        }

        let out = sqlx::query_as::<_, LineOut>(
            "
            WITH score AS (
                SELECT id, fuzzy_score(?, text) fs FROM all_lines
                WHERE text like ? AND fs < 4096 ORDER BY fs LIMIT 300
            )

            SELECT 
                l.* 
            FROM 
                score s LEFT JOIN all_lines l ON l.id = s.id
            ",
        )
        .bind(search_query)
        .bind(like_query)
        .fetch_all(&self.0)
        .await;

        if let Err(err) = out {
            NeoDebug::log(&err).await;
            return Err(err);
        } else {
            Ok(out.unwrap())
        }
    }

    pub async fn empty_lines(&self) {
        if let Err(err) = sqlx::query("DELETE FROM all_lines").execute(&self.0).await {
            NeoDebug::log(err).await;
        }

        CONTAINER.search_state.write().await.db_count = 0;
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
