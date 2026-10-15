use super::*;
use axum::Extension;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

fn app() -> (
    Router,
    String,
    crate::Store<crate::db_sqlite::SqliteConnection>,
) {
    let store = crate::Store::new(crate::db_sqlite::SqliteConnection::open_memory().unwrap());
    store.users().set_password("alice", "password").unwrap();
    let token = store.users().generate_api_key("alice").unwrap();
    let server = Server::new(store.clone());
    (
        build_api_v1_router()
            .layer(Extension(server.clone()))
            .with_state(server),
        token,
        store,
    )
}

async fn request(
    app: &Router,
    method: &str,
    path: &str,
    body: Value,
    token: Option<&str>,
) -> (u16, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("Content-Type", "application/json");
    if let Some(token) = token {
        request = request.header(AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status().as_u16();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn assignment(family: &str) -> Value {
    let mut value = json!({"name":"Assignment", "description":"description", "visibility":"public", "resource_type":family, "parent_id":null});
    match family {
        "asn" => {
            value["asn_from"] = json!(65000);
            value["asn_to"] = json!(65199);
        }
        "ipv4" => {
            value["ipv4_prefix"] = json!([192, 168, 0, 0]);
            value["ipv4_prefix_len"] = json!(16);
        }
        _ => {
            value["ipv6_prefix"] = json!([32, 1, 13, 184, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            value["ipv6_prefix_len"] = json!(32);
        }
    }
    value
}

async fn create(app: &Router, token: &str, body: Value) -> i64 {
    let (status, response) = request(app, "POST", "/assignment", body, Some(token)).await;
    assert_eq!(status, 200, "{response}");
    response["assignment"]["id"].as_i64().unwrap()
}

#[tokio::test]
async fn arbitrary_nesting_containment_and_sibling_overlap_rules() {
    let (app, token, _) = app();
    for family in ["asn", "ipv4", "ipv6"] {
        let original = assignment(family);
        let root = create(&app, &token, original.clone()).await;
        // Identical roots may overlap and remain independent.
        let other_root = create(&app, &token, original.clone()).await;
        assert_ne!(root, other_root);
        let mut parent = root;
        for depth in 0..8 {
            let mut child = original.clone();
            child["parent_id"] = json!(parent);
            child["name"] = json!(format!("Depth {depth}"));
            let id = create(&app, &token, child.clone()).await;
            assert_eq!(
                request(&app, "POST", "/assignment", child, Some(&token))
                    .await
                    .0,
                400
            );
            let (_, ancestors) = request(
                &app,
                "GET",
                &format!("/assignment/{id}/ancestors"),
                Value::Null,
                Some(&token),
            )
            .await;
            assert_eq!(
                ancestors["assignments"].as_array().unwrap().len(),
                depth + 1
            );
            assert_eq!(ancestors["assignments"][0]["id"], root);
            parent = id;
        }
        let mut outside = original.clone();
        outside["parent_id"] = json!(parent);
        match family {
            "asn" => outside["asn_to"] = json!(65200),
            "ipv4" => outside["ipv4_prefix"] = json!([10, 0, 0, 0]),
            _ => outside["ipv6_prefix"] = json!(vec![0; 16]),
        }
        assert_eq!(
            request(&app, "POST", "/assignment", outside, Some(&token))
                .await
                .0,
            400
        );
        let mut wrong_family = assignment(if family == "asn" { "ipv4" } else { "asn" });
        wrong_family["parent_id"] = json!(parent);
        assert_eq!(
            request(&app, "POST", "/assignment", wrong_family, Some(&token))
                .await
                .0,
            400
        );
        let mut orphan = original;
        orphan["parent_id"] = json!(i32::MAX);
        assert_eq!(
            request(&app, "POST", "/assignment", orphan, Some(&token))
                .await
                .0,
            404
        );
    }
}

#[tokio::test]
async fn visibility_inherits_from_every_ancestor_and_metadata_remains_editable() {
    let (app, token, _) = app();
    let root = create(&app, &token, assignment("asn")).await;
    let mut child = assignment("asn");
    child["parent_id"] = json!(root);
    let child = create(&app, &token, child).await;
    let mut leaf = assignment("asn");
    leaf["parent_id"] = json!(child);
    let leaf = create(&app, &token, leaf).await;
    let update = json!({"name":"Private", "description":"Updated", "visibility":"private"});
    assert_eq!(
        request(
            &app,
            "PUT",
            &format!("/assignment/{root}"),
            update.clone(),
            None
        )
        .await
        .0,
        401
    );
    assert_eq!(
        request(
            &app,
            "PUT",
            &format!("/assignment/{root}"),
            update,
            Some(&token)
        )
        .await
        .0,
        200
    );
    for id in [root, child, leaf] {
        assert_eq!(
            request(&app, "GET", &format!("/assignment/{id}"), Value::Null, None)
                .await
                .0,
            404
        );
        assert_eq!(
            request(
                &app,
                "GET",
                &format!("/assignment/{id}/ancestors"),
                Value::Null,
                None
            )
            .await
            .0,
            404
        );
        assert_eq!(
            request(
                &app,
                "GET",
                &format!("/assignment?parent_id={id}"),
                Value::Null,
                None
            )
            .await
            .0,
            404
        );
        assert_eq!(
            request(
                &app,
                "GET",
                &format!("/assignment/{id}"),
                Value::Null,
                Some(&token)
            )
            .await
            .0,
            200
        );
    }
    let (_, roots) = request(&app, "GET", "/assignment", Value::Null, None).await;
    assert_eq!(roots["assignments"].as_array().unwrap().len(), 0);
    let (_, legacy) = request(
        &app,
        "PUT",
        &format!("/assignment/{root}"),
        json!({"name":"Metadata only", "description":""}),
        Some(&token),
    )
    .await;
    assert_eq!(legacy["assignment"]["visibility"], "private");
    assert_eq!(
        request(
            &app,
            "PUT",
            &format!("/assignment/{root}"),
            json!({"name":"Public", "description":"", "visibility":"public"}),
            Some(&token)
        )
        .await
        .0,
        200
    );
    assert_eq!(
        request(
            &app,
            "GET",
            &format!("/assignment/{leaf}"),
            Value::Null,
            None
        )
        .await
        .0,
        200
    );
    // A private intermediate ancestor also hides a public leaf.
    request(
        &app,
        "PUT",
        &format!("/assignment/{child}"),
        json!({"name":"Private child", "description":"", "visibility":"private"}),
        Some(&token),
    )
    .await;
    assert_eq!(
        request(
            &app,
            "GET",
            &format!("/assignment/{leaf}"),
            Value::Null,
            None
        )
        .await
        .0,
        404
    );
}

#[tokio::test]
async fn moving_subtrees_checks_containment_overlap_cycles_and_authentication() {
    let (app, token, _) = app();
    for family in ["asn", "ipv4", "ipv6"] {
        let root_a = create(&app, &token, assignment(family)).await;
        let root_b = create(&app, &token, assignment(family)).await;
        let mut child = assignment(family);
        child["parent_id"] = json!(root_a);
        let child = create(&app, &token, child).await;
        let mut leaf = assignment(family);
        leaf["parent_id"] = json!(child);
        let leaf = create(&app, &token, leaf).await;
        let move_url = format!("/assignment/{child}/move");
        let destination = json!({"parent_id":root_b});
        assert_eq!(
            request(&app, "POST", &move_url, destination.clone(), None)
                .await
                .0,
            401
        );
        assert_eq!(
            request(
                &app,
                "GET",
                &format!("/assignment/{child}/move-targets"),
                Value::Null,
                None
            )
            .await
            .0,
            401
        );
        for target in [child, leaf] {
            assert_eq!(
                request(
                    &app,
                    "POST",
                    &move_url,
                    json!({"parent_id":target}),
                    Some(&token)
                )
                .await
                .0,
                400
            );
        }
        assert_eq!(
            request(&app, "POST", &move_url, json!({}), Some(&token))
                .await
                .0,
            422
        );
        assert_eq!(
            request(
                &app,
                "POST",
                &move_url,
                json!({"parent_id":i32::MAX}),
                Some(&token)
            )
            .await
            .0,
            404
        );
        let (_, targets) = request(
            &app,
            "GET",
            &format!("/assignment/{child}/move-targets"),
            Value::Null,
            Some(&token),
        )
        .await;
        let ids: Vec<_> = targets["assignments"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["id"].as_i64().unwrap())
            .collect();
        assert!(ids.contains(&root_b));
        assert!(!ids.contains(&child));
        assert!(!ids.contains(&leaf));
        let (_, moved) = request(&app, "POST", &move_url, destination, Some(&token)).await;
        assert_eq!(moved["assignment"]["parent_id"], root_b);
        let (_, ancestors) = request(
            &app,
            "GET",
            &format!("/assignment/{leaf}/ancestors"),
            Value::Null,
            Some(&token),
        )
        .await;
        assert_eq!(ancestors["assignments"][0]["id"], root_b);
        assert_eq!(ancestors["assignments"][1]["id"], child);
        // The occupied destination rejects another overlapping subtree.
        let mut conflict = assignment(family);
        conflict["parent_id"] = json!(root_a);
        let conflict = create(&app, &token, conflict).await;
        assert_eq!(
            request(
                &app,
                "POST",
                &format!("/assignment/{conflict}/move"),
                json!({"parent_id":root_b}),
                Some(&token)
            )
            .await
            .0,
            400
        );
        let (_, targets) = request(
            &app,
            "GET",
            &format!("/assignment/{conflict}/move-targets"),
            Value::Null,
            Some(&token),
        )
        .await;
        assert!(!targets["assignments"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["id"] == root_b));
        // Promotion to root permits overlap and retains every descendant.
        assert_eq!(
            request(
                &app,
                "POST",
                &move_url,
                json!({"parent_id":null}),
                Some(&token)
            )
            .await
            .0,
            200
        );
        let (_, ancestors) = request(
            &app,
            "GET",
            &format!("/assignment/{leaf}/ancestors"),
            Value::Null,
            Some(&token),
        )
        .await;
        assert_eq!(ancestors["assignments"].as_array().unwrap().len(), 1);
        // No-op moves are allowed.
        assert_eq!(
            request(
                &app,
                "POST",
                &move_url,
                json!({"parent_id":null}),
                Some(&token)
            )
            .await
            .0,
            200
        );
    }
}

#[tokio::test]
async fn moves_reject_outside_ranges_and_other_resource_types_and_inherit_privacy() {
    let (app, token, _) = app();
    let asn = create(&app, &token, assignment("asn")).await;
    let mut narrow = assignment("asn");
    narrow["asn_to"] = json!(65050);
    let narrow = create(&app, &token, narrow).await;
    let ipv4 = create(&app, &token, assignment("ipv4")).await;
    for parent in [narrow, ipv4] {
        assert_eq!(
            request(
                &app,
                "POST",
                &format!("/assignment/{asn}/move"),
                json!({"parent_id":parent}),
                Some(&token)
            )
            .await
            .0,
            400
        );
    }
    let mut private = assignment("asn");
    private["visibility"] = json!("private");
    let private = create(&app, &token, private).await;
    assert_eq!(
        request(
            &app,
            "POST",
            &format!("/assignment/{asn}/move"),
            json!({"parent_id":private}),
            Some(&token)
        )
        .await
        .0,
        200
    );
    assert_eq!(
        request(
            &app,
            "GET",
            &format!("/assignment/{asn}"),
            Value::Null,
            None
        )
        .await
        .0,
        404
    );
}

#[tokio::test]
async fn malformed_headers_invalid_ranges_and_missing_objects_are_safe() {
    let (app, token, _) = app();
    for (path, status) in [("/assignment", 200), ("/user/self", 401)] {
        let request = Request::builder()
            .uri(path)
            .header(
                AUTHORIZATION,
                http::HeaderValue::from_bytes(b"Bearer \xff").unwrap(),
            )
            .body(Body::empty())
            .unwrap();
        assert_eq!(app.clone().oneshot(request).await.unwrap().status(), status);
    }
    assert_eq!(
        request(&app, "POST", "/assignment", assignment("asn"), None)
            .await
            .0,
        401
    );
    for (family, max_len) in [("ipv4", 32), ("ipv6", 128)] {
        for len in [-1, max_len + 1, 100_000, i32::MAX] {
            let mut value = assignment(family);
            value[format!("{family}_prefix_len")] = json!(len);
            assert_eq!(
                request(&app, "POST", "/assignment", value, Some(&token))
                    .await
                    .0,
                400
            );
        }
        let mut host_bits = assignment(family);
        let last = if family == "ipv4" { 3 } else { 15 };
        host_bits[format!("{family}_prefix")][last] = json!(1);
        assert_eq!(
            request(&app, "POST", "/assignment", host_bits, Some(&token))
                .await
                .0,
            400
        );
    }
    let mut invalid = assignment("asn");
    invalid["asn_from"] = json!(65200);
    assert_eq!(
        request(&app, "POST", "/assignment", invalid, Some(&token))
            .await
            .0,
        400
    );
    for method in ["GET", "PUT", "DELETE"] {
        assert_eq!(
            request(
                &app,
                method,
                "/assignment/9999",
                json!({"name":"", "description":""}),
                Some(&token)
            )
            .await
            .0,
            404
        );
    }
}

#[tokio::test]
async fn deleting_a_deep_subtree_keeps_other_roots_and_children() {
    let (app, token, store) = app();
    let root = create(&app, &token, assignment("asn")).await;
    let other = create(&app, &token, assignment("asn")).await;
    let mut node = store.assignments().get_assignment(root as i32).unwrap();
    let mut ids = vec![root];
    // More than SQLite's cascading-trigger recursion limit.
    for _ in 0..1100 {
        node.parent_id = Some(*ids.last().unwrap() as i32);
        ids.push(store.assignments().create_assignment(&node).unwrap() as i64);
    }
    assert_eq!(
        store
            .assignments()
            .get_ancestors(*ids.last().unwrap() as i32)
            .unwrap()
            .len(),
        1100
    );
    assert!(store
        .assignments()
        .is_public(*ids.last().unwrap() as i32)
        .unwrap());
    assert_eq!(
        request(
            &app,
            "DELETE",
            &format!("/assignment/{root}"),
            Value::Null,
            None
        )
        .await
        .0,
        401
    );
    assert_eq!(
        request(
            &app,
            "DELETE",
            &format!("/assignment/{root}"),
            Value::Null,
            Some(&token)
        )
        .await
        .0,
        200
    );
    assert!(store
        .assignments()
        .get_assignment(*ids.last().unwrap() as i32)
        .is_err());
    assert!(store.assignments().get_assignment(other as i32).is_ok());
}

#[tokio::test]
async fn adjacent_siblings_are_allowed_but_contained_and_enclosing_ranges_overlap() {
    let (app, token, _) = app();
    for family in ["asn", "ipv4", "ipv6"] {
        let root = create(&app, &token, assignment(family)).await;
        let mut left = assignment(family);
        left["parent_id"] = json!(root);
        let mut right = left.clone();
        let mut enclosing = left.clone();
        let mut contained = left.clone();
        match family {
            "asn" => {
                left["asn_to"] = json!(65049);
                right["asn_from"] = json!(65050);
                right["asn_to"] = json!(65099);
                enclosing["asn_to"] = json!(65100);
                contained["asn_from"] = json!(65010);
                contained["asn_to"] = json!(65020);
            }
            "ipv4" => {
                left["ipv4_prefix_len"] = json!(25);
                right["ipv4_prefix"] = json!([192, 168, 0, 128]);
                right["ipv4_prefix_len"] = json!(25);
                enclosing["ipv4_prefix_len"] = json!(24);
                contained["ipv4_prefix_len"] = json!(26);
            }
            _ => {
                left["ipv6_prefix_len"] = json!(65);
                right["ipv6_prefix"][8] = json!(128);
                right["ipv6_prefix_len"] = json!(65);
                enclosing["ipv6_prefix_len"] = json!(64);
                contained["ipv6_prefix_len"] = json!(66);
            }
        }
        create(&app, &token, left).await;
        create(&app, &token, right).await;
        for overlap in [enclosing, contained] {
            assert_eq!(
                request(&app, "POST", "/assignment", overlap, Some(&token))
                    .await
                    .0,
                400
            );
        }
        let (_, children) = request(
            &app,
            "GET",
            &format!("/assignment?parent_id={root}"),
            Value::Null,
            Some(&token),
        )
        .await;
        assert_eq!(children["assignments"].as_array().unwrap().len(), 2);
    }
}
