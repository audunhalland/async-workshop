use crate::model::{AppError, TodoFilter};
use crate::repository::Repository;

use super::todo_item::TodoItem;

///
/// The root of the GraphQL 'Query' type
///
#[derive(Debug)]
pub struct Query;

#[async_graphql::Object]
impl Query {
    /// Query our todo items.
    async fn todo_items(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<TodoItem>, AppError> {
        let repository = ctx.data_unchecked::<Repository>();
        let todo_items: Vec<TodoItem> = repository
            .list_todo_items(TodoFilter {
                range: 0..20,
            })
            .await?;

        Ok(todo_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_graphql::{value, EmptyMutation, EmptySubscription};

    #[tokio::test]
    async fn fetching_a_list_of_todo_items_should_work() {
        let mut mock_repo = Repository::faux();
        faux::when!(
            mock_repo.list_todo_items(_))
                .then_return(Ok(vec![test_todo_item()])
        );

        let response = test_execute(mock_repo, "
            {
                todoItems {
                    id
                    description
                }
            }
        ").await;

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

    async fn test_execute(repository: Repository, query: &str) -> async_graphql::Response {
        async_graphql::Schema::build(Query, EmptyMutation, EmptySubscription)
            .data(repository)
            .finish()
            .execute(query)
            .await
    }
}
