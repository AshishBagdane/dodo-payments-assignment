use dodo_payments_assignment::domain::services::WebhookDispatcher;
use dodo_payments_assignment::infrastructure::http_client::ReqwestWebhookDispatcher;
use serde_json::json;
use std::sync::{Arc, Mutex};
use axum::{routing::post, Router};
use tokio::net::TcpListener;

// Helper to start a mock server that responds with a sequence of status codes
async fn start_mock_server(response_sequence: Vec<u16>) -> (String, Arc<Mutex<Vec<u16>>>) {
    let sequence = Arc::new(Mutex::new(response_sequence));
    let sequence_clone = sequence.clone();

    let app = Router::new().route("/webhook", post(move || {
        let seq = sequence_clone.clone();
        async move {
            let mut guard = seq.lock().unwrap();
            let code = if !guard.is_empty() { guard.remove(0) } else { 200 };
            axum::http::StatusCode::from_u16(code).unwrap()
        }
    }));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}/webhook", addr);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (url, sequence)
}

#[tokio::test]
async fn test_dispatch_success_first_attempt() {
    let (url, _) = start_mock_server(vec![200]).await;
    let dispatcher = ReqwestWebhookDispatcher::new(3, 10);
    let payload = json!({"event": "test"});
    
    let result = dispatcher.dispatch(&url, &payload, "secret").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_retries_on_failure_and_succeeds() {
    // Fail first 2 times with 500, then succeed with 200
    let (url, _) = start_mock_server(vec![500, 500, 200]).await;
    
    let dispatcher = ReqwestWebhookDispatcher::new(3, 10);
    let payload = json!({"event": "test"});
    
    let result = dispatcher.dispatch(&url, &payload, "secret").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_fails_after_max_retries() {
    // Fail more times than max retries
    let (url, _) = start_mock_server(vec![500, 500, 500]).await;
    
    let max_retries = 2;
    let dispatcher = ReqwestWebhookDispatcher::new(max_retries, 10);
    let payload = json!({"event": "test"});
    
    let result = dispatcher.dispatch(&url, &payload, "secret").await;
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), format!("Webhook dispatch failed after {} attempts", max_retries));
}
