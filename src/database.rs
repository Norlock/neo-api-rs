use std::{path::PathBuf, str::FromStr};
use tokio::fs;

use crate::{LineOut, NeoDebug, NeoUtils, RTM};

pub struct Database {
    mem_db: sqlx::SqlitePool,
    file_db: sqlx::SqlitePool,
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

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
        let tmp = NeoUtils::home_directory().join(".local/share/neo-api-rs");
        let extension_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/extensions/fuzzy")
            .to_string_lossy()
            .to_string();

        if !tmp.is_dir() {
            fs::create_dir(&tmp).await?;
        }

        let mem_options = sqlx::sqlite::SqliteConnectOptions::from_str(":memory:")
            .unwrap()
            .extension(extension_path.clone());

        let file_options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(tmp.join("fuzzy_search.sqlite"))
            .create_if_missing(true)
            .extension(extension_path);

        let mem = sqlx::SqlitePool::connect_with(mem_options).await?;
        let file = sqlx::SqlitePool::connect_with(file_options).await?;

        sqlx::query(
            "CREATE TABLE all_lines (
                icon        TEXT NOT NULL,
                hl_group    TEXT NOT NULL,
                path_prefix TEXT NOT NULL,
                path_suffix TEXT PRIMARY KEY,
                line_nr     INTEGER
            )",
        )
        .execute(&mem)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS recent_directories (
                path_prefix    TEXT NOT NULL,
                path_suffix    TEXT NOT NULL UNIQUE
            )",
        )
        .execute(&file)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS settings (
                directory_tab INTEGER NOT NULL
            )",
        )
        .execute(&file)
        .await?;

        NeoDebug::log("Databases initialized").await;

        Ok(Self { file_db: file, mem_db: mem })
    }

    pub async fn all_lines_is_empty(&self) -> bool {
        match sqlx::query_scalar::<_, u32>("SELECT COUNT(*) FROM all_lines")
            .fetch_one(&self.mem_db)
            .await
        {
            Ok(count) => count == 0,
            Err(e) => {
                NeoDebug::log(e).await;
                true
            }
        }
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
            WHERE path_suffix like ?
            ORDER BY fuzzy_score(?, path_suffix) LIMIT 300
            ",
        )
        .bind(like_query)
        .bind(search_query)
        .fetch_all(&self.mem_db)
        .await;

        match out {
            Ok(out) => {
                //NeoDebug::log_dbg(&out).await;
                Ok(out)
            }
            Err(e) => {
                NeoDebug::log(&e).await;
                Err(e)
            }
        }
    }

    pub async fn search_project_lines(
        &self,
        search_text: &str,
        path_prefix: &str,
    ) -> Vec<LineOut> {
        let mut path_suffix_query = '%'.to_string();

        for char in search_text.chars() {
            path_suffix_query.push(char);
            path_suffix_query.push('%');
        }

        let out = sqlx::query_as::<_, LineOut>(
            "
            SELECT 
                *
            FROM 
                all_lines 
            WHERE path_prefix = ? AND path_suffix like ? 
            ORDER BY fuzzy_score(?, path_suffix) LIMIT 300
            ",
        )
        .bind(path_prefix)
        .bind(path_suffix_query)
        .bind(search_text)
        .fetch_all(&self.mem_db)
        .await;

        match out {
            Ok(out) => out,
            Err(err) => {
                NeoDebug::log(&err).await;
                vec![]
            }
        }
    }

    pub async fn insert_recent_directory(
        &self,
        path_prefix: &str,
        path_suffix: &str,
    ) {
        if let Err(e) = sqlx::query(
            "INSERT OR IGNORE INTO recent_directories (path_prefix, path_suffix) 
                VALUES (?, ?)",
        )
        .bind(path_prefix.trim())
        .bind(path_suffix.trim())
        .execute(&self.file_db)
        .await
        {
            NeoDebug::log(e).await;
        }
    }

    pub async fn search_recent_directories(
        &self,
        search_query: &str,
    ) -> sqlx::Result<Vec<LineOut>> {
        let mut like_query = '%'.to_string();

        for char in search_query.chars() {
            like_query.push(char);
            like_query.push('%');
        }

        let out: Vec<(Box<str>, Box<str>)> = sqlx::query_as(
            "
            SELECT 
                *
            FROM 
                recent_directories 
            WHERE path_suffix like ? 
            ORDER BY fuzzy_score(?, path_suffix) LIMIT 300
            ",
        )
        .bind(like_query)
        .bind(search_query)
        .fetch_all(&self.file_db)
        .await?;

        Ok(out.into_iter().map(|p| LineOut::directory(&p.0, &p.1)).collect())
    }

    pub async fn delete_recent_directory(&self, path_suffix: &str) {
        if let Err(err) = sqlx::query("DELETE FROM recent_directories WHERE path_suffix = ?")
            .bind(path_suffix)
            .execute(&self.file_db)
            .await
        {
            NeoDebug::log(err).await;
        }
    }

    pub async fn empty_lines(&self) {
        if let Err(err) = sqlx::query("DELETE FROM all_lines")
            .execute(&self.mem_db)
            .await
        {
            NeoDebug::log(err).await;
        }
    }

    pub async fn insert_all(&self, lines: &[LineOut]) -> sqlx::Result<()> {
        let mut tx = self.mem_db.begin().await?;

        for chunks in lines.chunks(1000) {
            let mut qry_str =
                "INSERT INTO all_lines (icon, hl_group, path_prefix, path_suffix, line_nr) VALUES"
                    .to_string();

            for i in 0..chunks.len() {
                if i == 0 {
                    qry_str.push_str("(?, ?, ?, ?, ?)");
                } else {
                    qry_str.push_str(", (?, ?, ?, ?, ?)");
                }
            }

            let mut query = sqlx::query(&qry_str);

            for line in chunks {
                query = query
                    .bind(&line.icon)
                    .bind(&line.hl_group)
                    .bind(&line.path_prefix)
                    .bind(&line.path_suffix)
                    .bind(&line.line_nr);
            }

            query.execute(&mut *tx).await?;
        }

        tx.commit().await
    }
}
