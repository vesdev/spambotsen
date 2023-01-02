use sqlx::{FromRow, PgPool};
use std::fmt::Write;

#[derive(FromRow)]
struct Guild {
    id: u64,
}
