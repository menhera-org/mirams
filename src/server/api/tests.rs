use super::*;
use axum::Extension;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

fn app() -> (Router, String) {
    let db = crate::db_sqlite::SqliteConnection::open_memory().unwrap();
    let store = crate::Store::new(db);
    crate::example_data::add_example_data(store.clone());
    store
        .users()
        .set_password("alice", "test password")
        .unwrap();
    let token = store.users().generate_api_key("alice").unwrap();
    let server = Server::new(store);
    let app = build_api_v1_router()
        .layer(Extension(server.clone()))
        .with_state(server);
    (app, token)
}

async fn request(
    app: &Router,
    method: &str,
    path: &str,
    body: Value,
    token: Option<&str>,
) -> (u16, Value) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
    }
    let req = builder
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status().as_u16();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

#[tokio::test]
async fn visibility_updates_and_anonymous_access() {
    let (app, token) = app();
    for family in ["asn", "ipv4", "ipv6"] {
        let space_path = format!("/{family}/assignment_space/1");
        let pool_path = format!("{space_path}/pool/1");
        let assignment_path = format!("{pool_path}/assignment/1");
        for (path, variant, field, list_path, list_variant) in [
            (
                &space_path,
                format!("{family}_assignment_space"),
                "space_visibility",
                format!("/{family}/assignment_space"),
                format!("{family}_assignment_spaces"),
            ),
            (
                &pool_path,
                format!("{family}_assignment_pool"),
                "pool_visibility",
                format!("{space_path}/pool"),
                format!("{family}_assignment_pools"),
            ),
            (
                &assignment_path,
                format!("{family}_assignment"),
                "assignment_visibility",
                format!("{pool_path}/assignment"),
                format!("{family}_assignments"),
            ),
        ] {
            let update = json!({"name": "Updated", "description": "New description", "visibility": "private"});
            assert_eq!(
                request(&app, "PUT", path, update.clone(), None).await.0,
                401
            );
            let (status, body) = request(&app, "PUT", path, update, Some(&token)).await;
            assert_eq!(status, 200, "{family}: {body}");
            assert_eq!(body[&variant][field], "private");
            assert_eq!(body[&variant]["name"], "Updated");
            assert_eq!(body[&variant]["description"], "New description");
            assert_eq!(request(&app, "GET", path, Value::Null, None).await.0, 404);
            let (status, list) = request(&app, "GET", &list_path, Value::Null, None).await;
            assert_eq!(status, 200);
            assert_eq!(list[&list_variant].as_array().unwrap().len(), 0);

            // A private parent hides public descendants, including direct assignment URLs.
            if path == &space_path || path == &pool_path {
                assert_eq!(
                    request(&app, "GET", &assignment_path, Value::Null, None)
                        .await
                        .0,
                    404
                );
                assert_eq!(
                    request(
                        &app,
                        "GET",
                        &format!("{pool_path}/assignment"),
                        Value::Null,
                        None
                    )
                    .await
                    .0,
                    404
                );
            }

            // Older metadata-only requests must preserve the stored visibility.
            let legacy = json!({"name": "Legacy", "description": "Legacy description"});
            let (status, body) = request(&app, "PUT", path, legacy, Some(&token)).await;
            assert_eq!(status, 200);
            assert_eq!(body[&variant][field], "private");
            let (_, body) = request(&app, "GET", path, Value::Null, Some(&token)).await;
            assert_eq!(body[&variant][field], "private");

            let invalid = json!({"name": "Invalid", "description": "", "visibility": "invalid"});
            assert_eq!(
                request(&app, "PUT", path, invalid, Some(&token)).await.0,
                422
            );
            let public = json!({"name": "Public again", "description": "", "visibility": "public"});
            assert_eq!(
                request(&app, "PUT", path, public, Some(&token)).await.0,
                200
            );
            let (status, body) = request(&app, "GET", path, Value::Null, None).await;
            assert_eq!(status, 200);
            assert_eq!(body[&variant][field], "public");
        }
    }
}

#[tokio::test]
async fn overlapping_spaces_have_independent_pools_and_assignments() {
    let (app, token) = app();
    for family in ["asn", "ipv4", "ipv6"] {
        let spaces_path = format!("/{family}/assignment_space");
        let space_path = format!("{spaces_path}/1");
        let pool_path = format!("{space_path}/pool/1");
        let (_, body) = request(&app, "GET", &space_path, Value::Null, Some(&token)).await;
        let space = body[format!("{family}_assignment_space")].clone();
        let (_, body) = request(&app, "GET", &pool_path, Value::Null, Some(&token)).await;
        let pool = body[format!("{family}_assignment_pool")].clone();
        let (_, body) = request(
            &app,
            "GET",
            &format!("{pool_path}/assignment/1"),
            Value::Null,
            Some(&token),
        )
        .await;
        let assignment = body[format!("{family}_assignment")].clone();

        // Identical, narrower, and wider spaces are all permitted.
        for kind in ["identical", "narrower", "wider"] {
            let mut new_space = space.clone();
            new_space["name"] = json!(kind);
            if family == "asn" {
                if kind == "narrower" {
                    new_space["asn_to"] = json!(65099);
                }
                if kind == "wider" {
                    new_space["asn_to"] = json!(65500);
                }
            } else {
                let field = format!("{family}_prefix_len");
                let len = space[&field].as_i64().unwrap();
                if kind == "narrower" {
                    new_space[&field] = json!(len + 1);
                }
                if kind == "wider" {
                    new_space[&field] = json!(len - 1);
                }
            }
            let (status, body) = request(&app, "POST", &spaces_path, new_space, Some(&token)).await;
            assert_eq!(status, 200, "{family} {kind}: {body}");
            let space_id = body[format!("{family}_assignment_space")]["id"]
                .as_i64()
                .unwrap();
            let pools_path = format!("{spaces_path}/{space_id}/pool");
            let mut new_pool = pool.clone();
            new_pool["assignment_space_id"] = json!(space_id);
            let (status, body) =
                request(&app, "POST", &pools_path, new_pool.clone(), Some(&token)).await;
            assert_eq!(status, 200, "{family}: {body}");
            let pool_id = body[format!("{family}_assignment_pool")]["id"]
                .as_i64()
                .unwrap();
            let assignments_path = format!("{pools_path}/{pool_id}/assignment");
            let mut new_assignment = assignment.clone();
            new_assignment["assignment_pool_id"] = json!(pool_id);
            let (status, body) = request(
                &app,
                "POST",
                &assignments_path,
                new_assignment.clone(),
                Some(&token),
            )
            .await;
            assert_eq!(status, 200, "{family}: {body}");

            // Overlap protection still applies within each parent.
            assert_ne!(
                request(&app, "POST", &pools_path, new_pool.clone(), Some(&token))
                    .await
                    .0,
                200
            );
            assert_ne!(
                request(
                    &app,
                    "POST",
                    &assignments_path,
                    new_assignment.clone(),
                    Some(&token)
                )
                .await
                .0,
                200
            );
            // Containment remains enforced even when other spaces overlap.
            if family == "asn" {
                new_pool["asn_from"] = json!(0);
                new_assignment["asn"] = json!(0);
            } else {
                new_pool[format!("{family}_prefix_len")] = json!(0);
                new_assignment[format!("{family}_prefix_len")] = json!(0);
            }
            assert_ne!(
                request(&app, "POST", &pools_path, new_pool, Some(&token))
                    .await
                    .0,
                200
            );
            assert_ne!(
                request(
                    &app,
                    "POST",
                    &assignments_path,
                    new_assignment,
                    Some(&token)
                )
                .await
                .0,
                200
            );
            let (_, body) =
                request(&app, "GET", &assignments_path, Value::Null, Some(&token)).await;
            assert_eq!(
                body[format!("{family}_assignments")]
                    .as_array()
                    .unwrap()
                    .len(),
                1
            );
        }
    }
}
