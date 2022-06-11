use crate::database;
use crate::model::AppError;

use super::todo_item::TodoItem;

///
/// The root of the GraphQL 'Query' type
///
pub struct Mutation<A>(std::marker::PhantomData<A>);

impl<A> Default for Mutation<A> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[async_graphql::Object]
impl<A> Mutation<A>
where
    A: database::InsertTodoItem + database::SetDone + Send + Sync + 'static,
{
    async fn create_todo_item(
        &self,
        ctx: &async_graphql::Context<'_>,
        description: String,
    ) -> Result<TodoItem, AppError> {
        let app = ctx.data_unchecked::<A>();
        let item = app.insert_todo_item(&description).await?;

        Ok(item)
    }

    async fn set_done(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: uuid::Uuid,
    ) -> Result<bool, AppError> {
        let app = ctx.data_unchecked::<A>();
        let success = app.set_done(id).await?;

        Ok(success)
    }
}
