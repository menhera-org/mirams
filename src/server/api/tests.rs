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

#[tokio::test]
async fn malformed_authorization_is_handled_without_panicking() {
    let (app, _) = app();
    for (path, expected) in [("/ipv4/assignment_space", 200), ("/user/self", 401)] {
        let req = Request::builder()
            .uri(path)
            .header(
                AUTHORIZATION,
                http::HeaderValue::from_bytes(b"Bearer \xff").unwrap(),
            )
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            app.clone().oneshot(req).await.unwrap().status().as_u16(),
            expected
        );
    }
}

#[tokio::test]
async fn invalid_prefixes_are_rejected_before_allocation_work() {
    let (app, token) = app();
    for (family, max_len) in [("ipv4", 32), ("ipv6", 128)] {
        let root = format!("/{family}/assignment_space");
        let prefix = format!("{family}_prefix");
        let length = format!("{family}_prefix_len");
        for (object, get_path, create_path) in [
            ("assignment_space", format!("{root}/1"), root.clone()),
            (
                "assignment_pool",
                format!("{root}/1/pool/1"),
                format!("{root}/1/pool"),
            ),
            (
                "assignment",
                format!("{root}/1/pool/1/assignment/1"),
                format!("{root}/1/pool/1/assignment"),
            ),
        ] {
            let (_, body) = request(&app, "GET", &get_path, Value::Null, Some(&token)).await;
            let original = body[format!("{family}_{object}")].clone();
            for invalid_len in [-1, max_len + 1, 100_000, i32::MAX] {
                let mut invalid = original.clone();
                invalid[&length] = json!(invalid_len);
                let (status, body) =
                    request(&app, "POST", &create_path, invalid, Some(&token)).await;
                assert_eq!(status, 400, "{family} {object} /{invalid_len}: {body}");
            }
            let mut invalid = original.clone();
            // Host bits are invalid for a prefix shorter than the address width.
            invalid[&length] = json!(max_len - 1);
            let last = invalid[&prefix].as_array().unwrap().len() - 1;
            invalid[&prefix][last] = json!(1);
            assert_eq!(
                request(&app, "POST", &create_path, invalid, Some(&token))
                    .await
                    .0,
                400
            );
        }
        // Both /0 and single-address spaces are valid; spaces can still overlap.
        let (_, body) = request(&app, "GET", &format!("{root}/1"), Value::Null, Some(&token)).await;
        let mut space = body[format!("{family}_assignment_space")].clone();
        space[&prefix] = json!(vec![0; if family == "ipv4" { 4 } else { 16 }]);
        for valid_len in [0, max_len] {
            space[&length] = json!(valid_len);
            assert_eq!(
                request(&app, "POST", &root, space.clone(), Some(&token))
                    .await
                    .0,
                200
            );
        }
    }
}

#[tokio::test]
async fn enclosing_asn_pools_are_rejected() {
    let (app, token) = app();
    let space = json!({"name":"ASN range", "description":"", "space_visibility":"public", "asn_from":64000, "asn_to":65500});
    let (_, body) = request(&app, "POST", "/asn/assignment_space", space, Some(&token)).await;
    let space_id = body["asn_assignment_space"]["id"].as_i64().unwrap();
    let path = format!("/asn/assignment_space/{space_id}/pool");
    let mut pool = json!({"name":"Pool", "description":"", "pool_visibility":"public", "assignment_space_id":space_id, "asn_from":65000, "asn_to":65099});
    assert_eq!(
        request(&app, "POST", &path, pool.clone(), Some(&token))
            .await
            .0,
        200
    );
    for (from, to) in [(64999, 65100), (65020, 65030), (65099, 65100)] {
        pool["asn_from"] = json!(from);
        pool["asn_to"] = json!(to);
        assert_eq!(
            request(&app, "POST", &path, pool.clone(), Some(&token))
                .await
                .0,
            400
        );
    }
    pool["asn_from"] = json!(65100);
    pool["asn_to"] = json!(65199);
    assert_eq!(
        request(&app, "POST", &path, pool, Some(&token)).await.0,
        200
    );
}

#[tokio::test]
async fn nested_routes_enforce_all_parent_ids_without_mutating_objects() {
    let (app, token) = app();
    for family in ["asn", "ipv4", "ipv6"] {
        let root = format!("/{family}/assignment_space");
        let pool_path = format!("{root}/1/pool/1");
        let assignment_path = format!("{pool_path}/assignment/1");
        let (_, original_pool) = request(&app, "GET", &pool_path, Value::Null, Some(&token)).await;
        let (_, original_assignment) =
            request(&app, "GET", &assignment_path, Value::Null, Some(&token)).await;
        let (_, space) =
            request(&app, "GET", &format!("{root}/1"), Value::Null, Some(&token)).await;
        let (_, created) = request(
            &app,
            "POST",
            &root,
            space[format!("{family}_assignment_space")].clone(),
            Some(&token),
        )
        .await;
        let other_space = created[format!("{family}_assignment_space")]["id"]
            .as_i64()
            .unwrap();
        for wrong_space in [other_space, 999] {
            let wrong_pool_path = format!("{root}/{wrong_space}/pool/1");
            let wrong_assignment_path = format!("{wrong_pool_path}/assignment/1");
            for path in [&wrong_pool_path, &wrong_assignment_path] {
                for method in ["GET", "PUT", "DELETE"] {
                    let body =
                        json!({"name":"Wrong parent", "description":"", "visibility":"private"});
                    assert_eq!(
                        request(&app, method, path, body, Some(&token)).await.0,
                        404,
                        "{method} {path}"
                    );
                }
            }
            assert_eq!(
                request(
                    &app,
                    "GET",
                    &format!("{wrong_pool_path}/assignment"),
                    Value::Null,
                    Some(&token)
                )
                .await
                .0,
                404
            );
            assert_eq!(
                request(
                    &app,
                    "POST",
                    &format!("{wrong_pool_path}/assignment"),
                    original_assignment[format!("{family}_assignment")].clone(),
                    Some(&token)
                )
                .await
                .0,
                404
            );
        }
        for method in ["GET", "PUT", "DELETE"] {
            let path = format!("{root}/1/pool/999/assignment/1");
            let body = json!({"name":"Wrong pool", "description":"", "visibility":"private"});
            assert_eq!(
                request(&app, method, &path, body, Some(&token)).await.0,
                404
            );
        }
        assert_eq!(
            request(&app, "GET", &pool_path, Value::Null, Some(&token))
                .await
                .1,
            original_pool
        );
        assert_eq!(
            request(&app, "GET", &assignment_path, Value::Null, Some(&token))
                .await
                .1,
            original_assignment
        );
        // Correct paths still work for updates and cascading deletes.
        let body = json!({"name":"Valid update", "description":"", "visibility":"public"});
        assert_eq!(
            request(&app, "PUT", &assignment_path, body.clone(), Some(&token))
                .await
                .0,
            200
        );
        assert_eq!(
            request(&app, "PUT", &pool_path, body, Some(&token)).await.0,
            200
        );
        assert_eq!(
            request(&app, "DELETE", &pool_path, Value::Null, Some(&token))
                .await
                .0,
            200
        );
        assert_eq!(
            request(&app, "GET", &assignment_path, Value::Null, Some(&token))
                .await
                .0,
            404
        );
    }
}
