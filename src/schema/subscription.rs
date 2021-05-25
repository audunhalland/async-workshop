use futures::Stream;
use futures::TryStreamExt;

use crate::bus::EventBus;

use super::todo_item::TodoItem;

///
/// The root of the GraphQL 'Subscription' type
///
pub struct Subscription;

#[async_graphql::Subscription]
impl Subscription {
    async fn new_todo_items(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> impl Stream<Item = Result<TodoItem, async_graphql::Error>> {
        let receiver = ctx.data_unchecked::<EventBus>().sender().subscribe();

        tokio_stream::wrappers::BroadcastStream::new(receiver).map_err(|err| {
            async_graphql::Error::new(format!("Failed to generate next event: {:?}", err))
        })
    }
}
