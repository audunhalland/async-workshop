use crate::schema::todo_item::TodoItem;

#[derive(Clone)]
pub struct EventBus {
    sender: tokio::sync::broadcast::Sender<TodoItem>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(42);
        Self { sender }
    }

    pub fn sender(&self) -> &tokio::sync::broadcast::Sender<TodoItem> {
        &self.sender
    }
}
