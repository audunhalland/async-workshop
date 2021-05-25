#[derive(async_graphql::SimpleObject, Clone, Debug, Eq, PartialEq)]
pub struct TodoItem {
    pub id: uuid::Uuid,
    pub description: String,
    pub done: bool,
}
