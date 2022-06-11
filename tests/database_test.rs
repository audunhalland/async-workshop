mod test_db;

use test_db::create_test_db;

use async_workshop::database::*;
use async_workshop::model::TodoFilter;

#[tokio::test]
async fn should_get_empty_todo_list_on_empty_database() {
    let db = create_test_db().await;
    let todo_items = db
        .list_todo_items(TodoFilter { range: 0..10 })
        .await
        .unwrap();

    assert_eq!(todo_items, vec![]);
}

#[tokio::test]
async fn should_insert_a_new_todo_item_and_then_fetch_it() {
    let db = create_test_db().await;

    let inserted_todo_item = db.insert_todo_item("foobar").await.unwrap();
    let todo_items = db
        .list_todo_items(TodoFilter { range: 0..10 })
        .await
        .unwrap();

    assert_eq!(todo_items, vec![inserted_todo_item]);
}

#[tokio::test]
async fn should_set_item_to_done() {
    let db = create_test_db().await;

    let item = db.insert_todo_item("foo").await.unwrap();
    assert_eq!(item.done, false);

    let success = db.set_done(item.id).await.unwrap();
    assert!(success);

    let items = db
        .list_todo_items(TodoFilter { range: 0..1 })
        .await
        .unwrap();

    assert_eq!(items[0].done, true);
}
