use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use axum::{extract::State, routing::post, Json, Router};
use loco_rs::testing::prelude::*;
use sea_orm::EntityTrait;
use serde_json::{json, Value};
use serial_test::serial;
use todo::{
    app::App,
    models::users,
    services::{
        chat::{self, ChatInputMessage, ChatProgress, ChatRequest},
        tools::ToolContext,
    },
};

async fn fake_ollama(
    State(calls): State<Arc<AtomicUsize>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let call = calls.fetch_add(1, Ordering::SeqCst);
    if call == 0 {
        assert!(body["tools"]
            .as_array()
            .is_some_and(|tools| !tools.is_empty()));
        return Json(json!({
            "model": "test-model",
            "message": {
                "role": "assistant",
                "content": "",
                "thinking": "Ich brauche die Projektliste.",
                "tool_calls": [{
                    "type": "function",
                    "function": { "name": "get_projects", "arguments": {} }
                }]
            },
            "eval_count": 4,
            "total_duration": 10
        }));
    }

    assert!(body["messages"].as_array().is_some_and(|messages| {
        messages
            .iter()
            .any(|message| message["role"] == "tool" && message["tool_name"] == "get_projects")
    }));
    Json(json!({
        "model": "test-model",
        "message": {
            "role": "assistant",
            "content": "Du hast aktuell keine Projekte.",
            "thinking": "Das Tool hat eine leere Liste geliefert."
        },
        "eval_count": 6,
        "total_duration": 20
    }))
}

#[tokio::test]
#[serial]
async fn agent_loop_executes_the_shared_tool_and_returns_the_final_answer() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let app = Router::new()
        .route("/api/chat", post(fake_ollama))
        .with_state(calls.clone());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let boot = boot_test::<App>().await.unwrap();
    seed::<App>(&boot.app_context).await.unwrap();
    let mut ctx = boot.app_context;
    ctx.config.settings = Some(json!({
        "ai": {
            "default_provider": "fake",
            "providers": [{
                "id": "fake",
                "name": "Fake Ollama",
                "kind": "ollama",
                "base_url": format!("http://{address}"),
                "default_model": "test-model"
            }]
        }
    }));
    let user = users::Entity::find_by_id(1)
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
    let progress = Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured_progress = progress.clone();

    let result = chat::run(
        &ctx,
        &user,
        ChatRequest {
            messages: vec![ChatInputMessage {
                role: "user".into(),
                content: "Welche Projekte habe ich?".into(),
            }],
            context: ToolContext::default(),
        },
        move |event| captured_progress.lock().unwrap().push(event),
    )
    .await
    .unwrap();

    assert_eq!(result.response, "Du hast aktuell keine Projekte.");
    assert_eq!(result.provider_id, "fake");
    assert_eq!(result.eval_count, Some(10));
    assert_eq!(result.tools.len(), 1);
    assert_eq!(result.tools[0].name, "get_projects");
    assert!(result.tools[0].success);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert!(progress.lock().unwrap().iter().any(
        |event| matches!(event, ChatProgress::RunningTool { name } if name == "get_projects")
    ));

    server.abort();
}
