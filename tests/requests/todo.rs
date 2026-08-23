use loco_rs::testing::prelude::*;
use serial_test::serial;
use todo::app::App;

#[tokio::test]
#[serial]
async fn can_get_todos() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/api/todos/").await;
        assert_eq!(res.status_code(), 200);

        // you can assert content like this:
        // assert_eq!(res.text(), "content");
    })
    .await;
}
