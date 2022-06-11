use crate::model::AppError;
use crate::repository::Repository;

use super::todo_item::TodoItem;

///
/// The root of the GraphQL 'Query' type
///
pub struct Mutation;

#[async_graphql::Object]
impl Mutation {
    async fn create_todo_item(
        &self,
        ctx: &async_graphql::Context<'_>,
        description: String,
    ) -> Result<TodoItem, AppError> {
        let repository = ctx.data_unchecked::<Repository>();
        let item = repository.insert_todo_item(&description).await?;

        Ok(item)
    }

    async fn set_done(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: uuid::Uuid,
    ) -> Result<bool, AppError> {
        let repository = ctx.data_unchecked::<Repository>();
        let success = repository.set_done(id).await?;

        Ok(success)
    }
}
