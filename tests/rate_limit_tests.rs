use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use dodo_payments_assignment::presentation::middleware::rate_limit::RateLimitLayer;
use tower::ServiceExt; // for oneshot

#[tokio::test]
async fn test_rate_limiting() {
    // 1. Create a dummy router with rate limiting
    // Limit: 2 requests per hour
    let rate_limit_layer = RateLimitLayer::new(2);

    let app = Router::new()
        .route("/", axum::routing::get(|| async { "OK" }))
        .layer(axum::middleware::from_fn_with_state(rate_limit_layer, RateLimitLayer::handle));

    // 2. Helper to send request
    // Note: RateLimitLayer uses ConnectInfo<SocketAddr>. In tests using `oneshot`, ConnectInfo might be missing.
    // We need to inject ConnectInfo via `ConnectInfo::layer` or manually extension.
    // However, `oneshot` doesn't run the `into_make_service_with_connect_info` logic.
    // We must manually add `ConnectInfo` extension to the Request.
    
    let send_request = |app: &Router| {
        let req = Request::builder()
            .uri("/")
            .extension(axum::extract::ConnectInfo(std::net::SocketAddr::from(([127, 0, 0, 1], 1234)))) // Mock IP
            .body(Body::empty())
            .unwrap();
        app.clone().oneshot(req)
    };

    // 3. Request 1 (Allowed)
    let res1 = send_request(&app).await.unwrap();
    assert_eq!(res1.status(), StatusCode::OK);

    // 4. Request 2 (Allowed)
    let res2 = send_request(&app).await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);

    // 5. Request 3 (Blocked)
    let res3 = send_request(&app).await.unwrap();
    assert_eq!(res3.status(), StatusCode::TOO_MANY_REQUESTS);
}
