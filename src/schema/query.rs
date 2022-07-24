use crate::database;
use crate::model::{AppError, TodoFilter};

use super::todo_item::TodoItem;

///
/// The root of the GraphQL 'Query' type
///
#[derive(Debug)]
pub struct Query<A>(std::marker::PhantomData<A>);

impl<A> Default for Query<A> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[async_graphql::Object]
impl<A> Query<A>
where
    A: database::ListTodoItems + Send + Sync + 'static,
{
    /// Query our todo items.
    async fn todo_items(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<TodoItem>, AppError> {
        let app = ctx.data_unchecked::<A>();
        let todo_items: Vec<TodoItem> = app.list_todo_items(TodoFilter { range: 0..20 }).await?;

        Ok(todo_items)
    }
}

#[cfg(test)]
mod tests {
    use crate::database;

    use super::*;
    use async_graphql::{value, EmptyMutation, EmptySubscription};
    use unimock::*;

    #[tokio::test]
    async fn fetching_a_list_of_todo_items_should_work() {
        let response = test_execute(
            mock(Some(
                database::list_todo_items::Fn
                    .next_call(matching!(_))
                    .returns(Ok(vec![test_todo_item()]))
                    .once()
                    .in_order(),
            )),
            "
            {
                todoItems {
                    id
                    description
                }
            }
        ",
        )
        .await;

        assert_eq!(
            response.data,
            value!({
                "todoItems": [{
                    "id": uuid::Uuid::nil().to_string(),
                    "description": "test"
                }]
            })
        );
    }

    fn test_todo_item() -> TodoItem {
        TodoItem {
            id: uuid::Uuid::nil(),
            description: "test".to_string(),
            done: false,
        }
    }

    async fn test_execute(mock_app: unimock::Unimock, query: &str) -> async_graphql::Response {
        async_graphql::Schema::build(
            Query::<unimock::Unimock>::default(),
            EmptyMutation,
            EmptySubscription,
        )
        .data(mock_app)
        .finish()
        .execute(query)
        .await
    }
}
