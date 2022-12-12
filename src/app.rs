use entrait::entrait_export as entrait;

pub struct App {
    pub pg_pool: sqlx::PgPool,
}

#[entrait(pub GetPgPool, mock_api = GetPgPoolMock)]
pub fn get_pg_pool(app: &App) -> &sqlx::PgPool {
    &app.pg_pool
}
