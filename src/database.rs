use sqlx::{sqlite::SqliteConnectOptions, Connection};
use std::ops::Range;

use crate::{LineOut, RTM};

pub struct Database(sqlx::SqliteConnection);

pub trait DatabaseQuery {
    fn insert(&self) -> String;
}

impl Database {
    pub fn init() -> sqlx::Result<Self> {
        RTM.block_on(async {
            //let opts = sqlx::SqliteConnection::
            let opts = SqliteConnectOptions::new()
                .create_if_missing(true)
                .filename("/tmp/fuzzy.db");

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

            Ok(Self(conn))
        })
    }

    pub async fn select(&mut self, range: Range<u32>) -> sqlx::Result<Vec<LineOut>> {
        sqlx::query_as::<_, LineOut>(
            "SELECT text, icon, hl_group FROM all_lines WHERE ? <= id and id < ?",
        )
        .bind(range.start)
        .bind(range.end)
        .fetch_all(&mut self.0)
        .await
    }

    pub async fn insert_all(&mut self, lines: &[LineOut]) -> sqlx::Result<()> {
        for line in lines {
            let _stmt = sqlx::query("INSERT into all_lines (text, icon, hl_group) VALUES (?1, ?2, ?3)")
                .bind(&line.text)
                .bind(&line.icon)
                .bind(&line.hl_group)
                .execute(&mut self.0).await?;
        }

        Ok(())
    }
}
