use crate::model::{AppError, TodoFilter};
use crate::schema::todo_item::TodoItem;

///
/// A mockable Repository abstraction.
/// The `faux::create` macro generates `::faux()` static method.
/// It returns a mockable version of the struct.
///
/// `cfg_attr` is a way of doing conditional compilation. `test` means
/// that this is gated on being a _test build_ (avoid paying this cost in production)
///
/// `faux` works by actually modifying the type, and when it's active
/// it modifies the type so that it may be one of two variants at runtime:
/// A mocked instance instatiated by `::faux()` or a real instance
/// instatnated by `::new()`.
///
#[cfg_attr(test, faux::create)]
#[derive(Debug)]
pub struct Repository {
    pool: sqlx::PgPool,
}

#[cfg_attr(test, faux::methods)]
impl Repository {
    /// Construct a new Repository.
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Fetch todo items.
    pub async fn list_todo_items(&self, filter: TodoFilter) -> Result<Vec<TodoItem>, AppError> {
        let rows = sqlx::query_as!(
            TodoItem,
            "
            SELECT id, description, done
            FROM todo_item
            OFFSET $1
            LIMIT $2
            ",
            filter.range.start as i64,
            filter.range.end as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Create and insert a new TODO item, and return it
    pub async fn insert_todo_item(&self, description: &str) -> Result<TodoItem, AppError> {
        let row = sqlx::query_as!(
            TodoItem,
            "
            INSERT INTO todo_item
            (id, description, done)
            VALUES (uuid_generate_v4(), $1, false)
            RETURNING id, description, done
            ",
            description
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn set_done(&self, id: uuid::Uuid) -> Result<bool, AppError> {
        let result = sqlx::query!(
            "
            UPDATE todo_item
            SET done = true
            WHERE id = $1
            AND done = false
            ",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
