use sqlx::sqlite::SqliteJournalMode;
use std::{
    env,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::fs;

use crate::{LineOut, NeoApi, NeoDebug, RTM};

pub struct Database(sqlx::SqlitePool);

impl Database {
    pub fn new() -> Self {
        RTM.block_on(async move {
            let result = Database::init().await;

            if let Err(err) = result {
                NeoDebug::log(err.to_string()).await;
                std::process::exit(0);
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
                text        TEXT PRIMARY KEY,
                icon        TEXT NOT NULL,
                hl_group    TEXT NOT NULL,
                git_root    TEXT
            )",
        )
        .execute(&pool)
        .await?;

        Ok(Self(pool))
    }

    pub async fn search_lines(&self, search_query: &str) -> sqlx::Result<Vec<LineOut>> {
        let mut like_query = '%'.to_string();

        for char in search_query.chars() {
            like_query.push(char);
            like_query.push('%');
        }

        let out = sqlx::query_as::<_, LineOut>(
            "
            SELECT 
                *
            FROM 
                all_lines 
            WHERE text like ?
            ORDER BY fuzzy_score(?, text) LIMIT 300
            ",
        )
        .bind(like_query)
        .bind(search_query)
        .fetch_all(&self.0)
        .await;

        if let Err(err) = out {
            NeoDebug::log(&err).await;
            return Err(err);
        } else {
            Ok(out.unwrap())
        }
    }

    pub async fn search_project_lines(
        &self,
        search_query: &str,
        git_root: &Option<PathBuf>,
    ) -> sqlx::Result<Vec<LineOut>> {
        let mut like_query = '%'.to_string();

        NeoDebug::log_dbg(git_root).await;

        for char in search_query.chars() {
            like_query.push(char);
            like_query.push('%');
        }

        let git_root = git_root.clone();
        let git_root = git_root.map(|g| g.to_string_lossy().to_string());

        let out = sqlx::query_as::<_, LineOut>(
            "
            SELECT 
                *
            FROM 
                all_lines 
            WHERE text like ? AND git_root like ?
            ORDER BY fuzzy_score(?, text) LIMIT 300
            ",
        )
        .bind(like_query)
        .bind(git_root)
        .bind(search_query)
        .fetch_all(&self.0)
        .await;

        match out {
            Ok(out) => Ok(out),
            Err(err) => {
                NeoDebug::log(&err).await;
                Err(err)
            }
        }
    }

    pub async fn empty_lines(&self) {
        if let Err(err) = sqlx::query("DELETE FROM all_lines").execute(&self.0).await {
            NeoDebug::log(err).await;
        }
    }

    pub async fn insert_all(&self, lines: &[LineOut]) -> sqlx::Result<()> {
        let mut tx = self.0.begin().await?;

        for chunks in lines.chunks(1000) {
            let mut qry_str =
                "INSERT INTO all_lines (text, icon, hl_group, git_root) VALUES".to_string();

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
                    .bind(&line.text)
                    .bind(&line.icon)
                    .bind(&line.hl_group)
                    .bind(&line.git_root);
            }

            query.execute(&mut *tx).await?;
        }

        tx.commit().await
    }
}
