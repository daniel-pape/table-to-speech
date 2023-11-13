use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow)]
pub struct TestTableRow {
    pub id: u32,
    pub text: String,
    pub last_update: DateTime<Utc>,
}
