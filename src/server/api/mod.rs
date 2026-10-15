mod v1_assignment;
mod v1_login;
mod v1_user;

#[cfg(test)]
mod tests;

use crate::store::DbConnection;

use super::Server;

use axum::body::Body;
use axum::handler::Handler;
use axum::routing::{get, post};
use axum::Router;

use futures_util::future::BoxFuture;
use http::StatusCode;
use http::{header::AUTHORIZATION, Method, Request, Response};
use tower::ServiceBuilder;
use tower_http::auth::{AsyncAuthorizeRequest, AsyncRequireAuthorizationLayer};
use tower_http::cors;
use tower_http::timeout::TimeoutLayer;

use serde::{Deserialize, Serialize};

use std::any::TypeId;
use std::marker::PhantomData;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiResponse {
    pub error: Option<String>,

    #[serde(flatten)]
    pub result: Option<ApiResponseVariant>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ApiResponseVariant {
    User(User),
    Users(Vec<User>),
    ApiToken(String),

    Assignment(crate::assignment::Assignment),
    Assignments(Vec<crate::assignment::Assignment>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MetadataUpdateRequest {
    pub name: String,
    pub description: String,
    /// Omit to keep the current visibility.
    #[serde(default)]
    pub visibility: Option<crate::types::ObjectVisibility>,
}

pub fn build_json_response(response: ApiResponse, status: u16) -> Response<Body> {
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&response).unwrap()))
        .unwrap()
}

pub fn build_api_v1_router<T>() -> Router<Server<T>>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    let mut router = Router::new();

    router = router.route("/version", get(handler_version));

    router = router.route("/login", post(v1_login::api_v1_login));

    router = router.nest("/user", v1_user::build_router());

    router = router.merge(v1_assignment::build_router());

    // at the end, define the default route
    router = router.fallback(fallback_handler());

    let cors = cors::CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(cors::Any);

    let timeout =
        TimeoutLayer::with_status_code(StatusCode::GATEWAY_TIMEOUT, Duration::from_secs(30));

    let service = ServiceBuilder::new().layer(cors).layer(timeout);

    router = router.layer(service);

    router
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AuthHandler<T, AuthRequired: ?Sized = ()> {
    _phantom1: PhantomData<T>,
    _phantom2: PhantomData<AuthRequired>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AuthRequired;

#[allow(dead_code)]
impl<T, R> AuthHandler<T, R> {
    const fn new_internal() -> Self {
        AuthHandler {
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }

    const fn new() -> AuthHandler<T, ()> {
        AuthHandler::<T, ()>::new_internal()
    }

    const fn new_auth_required() -> AuthHandler<T, AuthRequired> {
        AuthHandler::<T, AuthRequired>::new_internal()
    }

    pub fn new_layer() -> AsyncRequireAuthorizationLayer<AuthHandler<T, ()>> {
        Self::new().into_layer()
    }

    pub fn new_auth_required_layer() -> AsyncRequireAuthorizationLayer<AuthHandler<T, AuthRequired>>
    {
        Self::new_auth_required().into_layer()
    }

    pub fn into_layer(self) -> AsyncRequireAuthorizationLayer<Self> {
        AsyncRequireAuthorizationLayer::new(self)
    }
}

impl<T, R> AsyncAuthorizeRequest<Body> for AuthHandler<T, R>
where
    T: DbConnection + Clone + Send + Sync + 'static,
    R: ?Sized + 'static,
{
    type RequestBody = Body;
    type ResponseBody = Body;
    type Future = BoxFuture<'static, Result<Request<Body>, Response<Self::ResponseBody>>>;

    fn authorize(&mut self, request: Request<Body>) -> Self::Future {
        let fut = async move {
            let (parts, body) = request.into_parts();
            let server = if let Some(s) = parts.extensions.get::<Server<T>>() {
                s.clone()
            } else {
                let request = Request::from_parts(parts, body);
                return Ok(request);
            };

            let mut request = Request::from_parts(parts, body);

            let authorization = request
                .headers()
                .get(AUTHORIZATION)
                .and_then(|header| header.to_str().ok())
                .map(|value| value.trim().to_owned());

            if let Some(authorization) = authorization {
                let parts: Vec<&str> = authorization.split_whitespace().collect();
                if parts.len() == 2 && parts[0] == "Bearer" {
                    let token = parts[1].to_owned();
                    let store = server.store().clone();
                    let user = tokio::task::spawn_blocking(move || {
                        let user_store = store.users();
                        match user_store.get_user_from_api_key(&token) {
                            Ok(Some(username)) => Some(User { username }),
                            _ => None,
                        }
                    })
                    .await
                    .unwrap();
                    if let Some(user) = user {
                        request.extensions_mut().insert(user);
                        return Ok(request);
                    }
                }
            }

            let type_id = TypeId::of::<R>();
            let auth_required_type_id = TypeId::of::<AuthRequired>();
            if auth_required_type_id == type_id {
                let response = response_unauthorized();
                return Err(response);
            }
            Ok(request)
        };
        Box::pin(fut)
    }
}

pub(crate) fn error_status(error: &crate::Error) -> u16 {
    match error.kind() {
        crate::ErrorKind::InvalidInput => 400,
        crate::ErrorKind::NotFound => 404,
        _ => 500,
    }
}

pub(crate) fn response_unauthorized() -> Response<Body> {
    let response = ApiResponse {
        error: Some("Unauthorized".to_string()),
        result: None,
    };
    build_json_response(response, 401)
}

pub(crate) fn response_not_found() -> Response<Body> {
    let response = ApiResponse {
        error: Some("Not found".to_string()),
        result: None,
    };
    build_json_response(response, 404)
}

pub(crate) fn fallback_handler<S>() -> impl Handler<(), S>
where
    S: Clone + Send + Sync + 'static,
{
    get(|| async { response_not_found() }).post(|| async { response_not_found() })
}

pub(crate) async fn run_blocking_task<T, F, R>(store: crate::store::Store<T>, f: F) -> R
where
    T: DbConnection + Clone + Send + Sync + 'static,
    F: FnOnce(crate::store::Store<T>) -> R + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(move || f(store)).await.unwrap()
}

async fn handler_version() -> Response<Body> {
    let json = serde_json::json!({
        "error": serde_json::Value::Null,
        "version": env!("CARGO_PKG_VERSION"),
    });
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&json).unwrap()))
        .unwrap()
}
