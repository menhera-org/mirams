use super::*;
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn production_assets_are_served_at_the_frontends_urls() {
    let app = build_frontend_router::<()>();
    for (path, content_type, expected_content) in [
        ("/main.css", "text/css", "--theme-primary-text-color"),
        ("/icon.svg", "image/svg+xml", "<svg"),
        ("/header.svg", "image/svg+xml", "<svg"),
    ] {
        let request = Request::builder().uri(path).body(Body::empty()).unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), 200, "{path}");
        assert_eq!(response.headers()["Content-Type"], content_type, "{path}");
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert!(
            body.contains(expected_content),
            "{path} returned the wrong content"
        );
    }

    let request = Request::builder()
        .uri("/favicon.ico")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), 301);
    assert_eq!(response.headers()["Location"], "/icon.svg");
}

#[tokio::test]
async fn production_assets_include_brotli_variants() {
    let app = build_frontend_router::<()>();
    for (path, content_type) in [
        ("/main.css", "text/css"),
        ("/icon.svg", "image/svg+xml"),
        ("/header.svg", "image/svg+xml"),
    ] {
        let request = Request::builder()
            .uri(path)
            .header("Accept-Encoding", "br")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), 200, "{path}");
        assert_eq!(response.headers()["Content-Type"], content_type, "{path}");
        assert_eq!(response.headers()["Content-Encoding"], "br", "{path}");
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(!body.is_empty(), "{path}");
    }
}
