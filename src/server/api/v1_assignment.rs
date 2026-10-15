use super::{
    build_json_response, error_status, run_blocking_task, ApiResponse, ApiResponseVariant,
    AuthHandler, MetadataUpdateRequest, User,
};
use crate::{
    assignment::{Assignment, ResourceType},
    server::Server,
    store::DbConnection,
    Error, ErrorKind, Store,
};
use axum::{
    body::Body,
    extract::{Extension, Json, Path, Query},
    routing::{delete, get, post, put},
    Router,
};
use http::Response;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ListQuery {
    parent_id: Option<i32>,
    resource_type: Option<ResourceType>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MoveRequest {
    #[serde(deserialize_with = "deserialize_parent")]
    parent_id: Option<i32>,
}

fn deserialize_parent<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<i32>, D::Error> {
    Option::deserialize(deserializer)
}

async fn respond<T, F>(server: Server<T>, operation: F) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
    F: FnOnce(Store<T>) -> Result<Option<ApiResponseVariant>, Error> + Send + 'static,
{
    match run_blocking_task(server.store().clone(), operation).await {
        Ok(result) => build_json_response(
            ApiResponse {
                error: None,
                result,
            },
            200,
        ),
        Err(error) => build_json_response(
            ApiResponse {
                error: Some(error.to_string()),
                result: None,
            },
            error_status(&error),
        ),
    }
}

fn require_visible(
    store: &dyn crate::assignment::AssignmentStore,
    id: i32,
    authenticated: bool,
) -> Result<Assignment, Error> {
    let assignment = store.get_assignment(id)?;
    if !authenticated && !store.is_public(id)? {
        return Err(Error::new(ErrorKind::NotFound, "Assignment not found"));
    }
    Ok(assignment)
}

async fn list<T>(
    Extension(server): Extension<Server<T>>,
    user: Option<Extension<User>>,
    Query(query): Query<ListQuery>,
) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    respond(server, move |store| {
        let assignments = store.assignments();
        if let Some(id) = query.parent_id {
            require_visible(&*assignments, id, user.is_some())?;
        }
        let mut children = assignments.get_assignments(query.parent_id, query.resource_type)?;
        if user.is_none() {
            children.retain(|a| a.visibility == crate::types::ObjectVisibility::Public);
        }
        Ok(Some(ApiResponseVariant::Assignments(children)))
    })
    .await
}

async fn get_assignment<T>(
    Extension(server): Extension<Server<T>>,
    user: Option<Extension<User>>,
    Path(id): Path<i32>,
) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    respond(server, move |store| {
        Ok(Some(ApiResponseVariant::Assignment(require_visible(
            &*store.assignments(),
            id,
            user.is_some(),
        )?)))
    })
    .await
}

async fn ancestors<T>(
    Extension(server): Extension<Server<T>>,
    user: Option<Extension<User>>,
    Path(id): Path<i32>,
) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    respond(server, move |store| {
        let assignments = store.assignments();
        require_visible(&*assignments, id, user.is_some())?;
        Ok(Some(ApiResponseVariant::Assignments(
            assignments.get_ancestors(id)?,
        )))
    })
    .await
}

async fn create<T>(
    Extension(server): Extension<Server<T>>,
    Json(assignment): Json<Assignment>,
) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    respond(server, move |store| {
        let assignments = store.assignments();
        let id = assignments.create_assignment(&assignment)?;
        Ok(Some(ApiResponseVariant::Assignment(
            assignments.get_assignment(id)?,
        )))
    })
    .await
}

async fn update<T>(
    Extension(server): Extension<Server<T>>,
    Path(id): Path<i32>,
    Json(update): Json<MetadataUpdateRequest>,
) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    respond(server, move |store| {
        let assignments = store.assignments();
        assignments.update_assignment(id, &update.name, &update.description, update.visibility)?;
        Ok(Some(ApiResponseVariant::Assignment(
            assignments.get_assignment(id)?,
        )))
    })
    .await
}

async fn move_assignment<T>(
    Extension(server): Extension<Server<T>>,
    Path(id): Path<i32>,
    Json(request): Json<MoveRequest>,
) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    respond(server, move |store| {
        let assignments = store.assignments();
        assignments.move_assignment(id, request.parent_id)?;
        Ok(Some(ApiResponseVariant::Assignment(
            assignments.get_assignment(id)?,
        )))
    })
    .await
}

async fn move_targets<T>(
    Extension(server): Extension<Server<T>>,
    Path(id): Path<i32>,
) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    respond(server, move |store| {
        Ok(Some(ApiResponseVariant::Assignments(
            store.assignments().get_move_targets(id)?,
        )))
    })
    .await
}

async fn delete_assignment<T>(
    Extension(server): Extension<Server<T>>,
    Path(id): Path<i32>,
) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    respond(server, move |store| {
        store.assignments().delete_assignment(id)?;
        Ok(None)
    })
    .await
}

pub fn build_router<T>() -> Router<Server<T>>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    Router::new()
        .route(
            "/assignment",
            get(list::<T>).layer(AuthHandler::<T>::new_layer()),
        )
        .route(
            "/assignment",
            post(create::<T>).layer(AuthHandler::<T>::new_auth_required_layer()),
        )
        .route(
            "/assignment/:id",
            get(get_assignment::<T>).layer(AuthHandler::<T>::new_layer()),
        )
        .route(
            "/assignment/:id",
            put(update::<T>).layer(AuthHandler::<T>::new_auth_required_layer()),
        )
        .route(
            "/assignment/:id",
            delete(delete_assignment::<T>).layer(AuthHandler::<T>::new_auth_required_layer()),
        )
        .route(
            "/assignment/:id/ancestors",
            get(ancestors::<T>).layer(AuthHandler::<T>::new_layer()),
        )
        .route(
            "/assignment/:id/move",
            post(move_assignment::<T>).layer(AuthHandler::<T>::new_auth_required_layer()),
        )
        .route(
            "/assignment/:id/move-targets",
            get(move_targets::<T>).layer(AuthHandler::<T>::new_auth_required_layer()),
        )
}
