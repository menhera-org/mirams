
use crate::store::DbConnection;
use crate::server::Server;
use super::AuthHandler;
use super::fallback_handler;
use super::build_json_response;
use super::User;
use super::ApiResponseVariant;
use super::ApiResponse;

use axum::Router;
use axum::body::Body;
use axum::routing::get;
use axum::extract::Extension as ExtensionExtractor;

use http::Response;


pub fn build_router<T>() -> Router<Server<T>>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    let mut router = Router::new();

    router = router.route("/self", get(user_self).layer(AuthHandler::<T>::new_auth_required_layer()));

    router = router.fallback(fallback_handler());

    router
}

pub async fn user_self(ext: Option<ExtensionExtractor<User>>) -> Response<Body> {
    if let Some(ext) = ext {
        let res = ApiResponse {
            error: None,
            result: Some(ApiResponseVariant::User(ext.0)),
        };
        return build_json_response(res, 200);
    } else {
        let res = ApiResponse {
            error: Some("Internal Server Error".to_string()),
            result: None,
        };
        return build_json_response(res, 500);
    }
}
