use crate::app::GetPgPool;
use crate::model::{AppError, TodoFilter};
use crate::schema::todo_item::TodoItem;

use entrait::entrait_export as entrait;

#[entrait(pub ListTodoItems, mock_api=ListTodoItemsMock)]
async fn list_todo_items(
    deps: &impl GetPgPool,
    filter: TodoFilter,
) -> Result<Vec<TodoItem>, AppError> {
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
    .fetch_all(deps.get_pg_pool())
    .await?;

    Ok(rows)
}

#[entrait(pub InsertTodoItem)]
async fn insert_todo_item(deps: &impl GetPgPool, description: &str) -> Result<TodoItem, AppError> {
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
    .fetch_one(deps.get_pg_pool())
    .await?;

    Ok(row)
}

#[entrait(pub SetDone)]
async fn set_done(deps: &impl GetPgPool, id: uuid::Uuid) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "
        UPDATE todo_item
        SET done = true
        WHERE id = $1
        AND done = false
        ",
        id
    )
    .execute(deps.get_pg_pool())
    .await?;

    Ok(result.rows_affected() > 0)
}
