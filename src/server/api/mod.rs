
pub mod login;

use crate::store::DbConnection;

use super::Server;

use axum::Router;
use axum::body::Body;
use axum::routing::{get, post};

use tower_http::auth::{AsyncRequireAuthorizationLayer, AsyncAuthorizeRequest};
use http::{Request, Response, StatusCode, header::AUTHORIZATION, Method};
use tower::ServiceBuilder;
use futures_util::future::BoxFuture;
use tower_http::cors;
use tower_http::timeout::TimeoutLayer;
use std::time::Duration;

use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub struct ApiResponse {
    pub error: Option<String>,

    #[serde(flatten)] 
    pub result: Option<ApiResponseVariant>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")] 
pub enum ApiResponseVariant {
    User(User),
    Users(Vec<User>),
    ApiToken(String),
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
}

pub fn build_json_response(response: ApiResponse, status: u16) -> Response<Body> {
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&response).unwrap()))
        .unwrap()
}


pub fn build_api_v1_router<T>(state: Server<T>) -> Router<Server<T>>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    let mut router = Router::new();

    router = router.route("/login", post(login::api_v1_login));

    router = router.nest("/authenticated", build_authenticated_router(state.clone()));

    // at the end, define the default route
    router = router.fallback(get(|| async move {
        let response = ApiResponse {
            error: Some("Not found".to_string()),
            result: None,
        };
        build_json_response(response, 404)
    }));

    let cors = cors::CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(cors::Any);

    let timeout = TimeoutLayer::new(Duration::from_secs(30));

    let service = ServiceBuilder::new()
        .layer(cors)
        .layer(timeout);

    router = router.layer(service);

    router
}

fn build_authenticated_router<T>(state: Server<T>) -> Router<Server<T>>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    let mut router = Router::new();

    // authenticated endpoints here

    router = router.layer(AsyncRequireAuthorizationLayer::new(AuthHandler::new(state.clone())));

    // at the end, define the default route
    router = router.fallback(get(|| async move {
        let response = ApiResponse {
            error: Some("Not found".to_string()),
            result: None,
        };
        build_json_response(response, 404)
    }));
    router
}

#[derive(Debug, Clone)]
pub struct AuthHandler<S> {
    inner: S,
}

impl<S> AuthHandler<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<T> AsyncAuthorizeRequest<Body> for AuthHandler<Server<T>>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    type RequestBody = Body;
    type ResponseBody = Body;
    type Future = BoxFuture<'static, Result<Request<Body>, Response<Self::ResponseBody>>>;

    fn authorize(&mut self, request: Request<Body>) -> Self::Future {
        let inner = self.inner.clone();
        let authorization = request.headers().get(AUTHORIZATION)
            .map(|header| header.to_str().unwrap().trim().to_owned());
        let fut = async move {
            if let Some(authorization) = authorization {
                let parts: Vec<&str> = authorization.split_whitespace().collect();
                if parts.len() == 2 && parts[0] == "Bearer" {
                    let token = parts[1].to_owned();
                    let store = inner.store().clone();
                    let user = tokio::task::spawn_blocking(move || {
                        let user_store = store.users();
                        user_store.get_user_from_api_key(&token)
                    }).await.unwrap().unwrap();
                    if let Some(_) = user {
                        return Ok(request);
                    }
                }
            }
            let response = ApiResponse {
                error: Some("Unauthorized".to_string()),
                result: None,
            };
            Err(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&response).unwrap()))
                .unwrap())
        };
        Box::pin(fut)
    }
}
