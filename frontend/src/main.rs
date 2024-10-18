#![allow(non_snake_case)]

pub mod component;
pub mod fetch;
pub mod inet;

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};

use serde::{Deserialize, Serialize};

pub const GITHUB_LINK: &str = "https://github.com/menhera-org/mirams";

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(Wrapper)]

    // Home

    #[route("/")]
    Home {},


    // Login

    #[route("/login/")]
    Login {},


    // ASN assignments

    #[route("/asn/")]
    AsnSpaceList {},

    #[route("/asn/space/:space_id/")]
    AsnSpace { space_id: i32 },

    #[route("/asn/space/:space_id/pool/:pool_id/")]
    AsnPool { space_id: i32, pool_id: i32 },

    #[route("/asn/space/:space_id/pool/:pool_id/assignment/:assignment_id/")]
    AsnAssignment { space_id: i32, pool_id: i32, assignment_id: i32 },


    // IPv4 assignments

    #[route("/ipv4/")]
    Ipv4SpaceList {},

    #[route("/ipv4/space/:space_id/")]
    Ipv4Space { space_id: i32 },

    #[route("/ipv4/space/:space_id/pool/:pool_id/")]
    Ipv4Pool { space_id: i32, pool_id: i32 },

    #[route("/ipv4/space/:space_id/pool/:pool_id/assignment/:assignment_id/")]
    Ipv4Assignment { space_id: i32, pool_id: i32, assignment_id: i32 },


    // IPv6 assignments

    #[route("/ipv6/")]
    Ipv6SpaceList {},

    #[route("/ipv6/space/:space_id/")]
    Ipv6Space { space_id: i32 },

    #[route("/ipv6/space/:space_id/pool/:pool_id/")]
    Ipv6Pool { space_id: i32, pool_id: i32 },

    #[route("/ipv6/space/:space_id/pool/:pool_id/assignment/:assignment_id/")]
    Ipv6Assignment { space_id: i32, pool_id: i32, assignment_id: i32 },
}

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");
    launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(Option::<component::account::User>::None));
    use_context_provider(|| Signal::new(component::DrawerState { open: false }));

    rsx! {
        Router::<Route> {}
    }
}

pub fn close_drawer() {
    let mut drawer = use_context::<Signal<component::DrawerState>>();
    drawer.set(component::DrawerState { open: false });
}

#[component]
fn Wrapper() -> Element {
    let nav = navigator();
    use_context_provider(move || nav.clone());

    let drawer_open = use_context::<Signal<component::DrawerState>>();
    let drawer_open = drawer_open().open;

    rsx! {
        link {
            rel: "icon",
            href: "/icon.svg",
        }
        link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200",
        }
        link {
            rel: "stylesheet",
            href: "/main.css",
        }
        div {
            id: "app",
            class: if drawer_open { "app-drawer-open" } else { "app-drawer-closed" },
            div {
                id: "app-top-bar",
                div {
                    id: "app-top-bar-side",
                    button {
                        class: "app-drawer-toggle-button app-button material-symbols-outlined",
                        onclick: move |_| {
                            let mut drawer = use_context::<Signal<component::DrawerState>>();
                            drawer.set(component::DrawerState { open: !drawer_open });
                        },
                        if drawer_open { "left_panel_close" } else { "left_panel_open" }
                    }
                    div {
                        id: "app-top-bar-branding",
                        "MIRAMS"
                    }
                }
                div {
                    id: "app-top-bar-main",
                }
            }
            div {
                id: "app-main",

                // The index route will be rendered here
                Outlet::<Route> {}
            }
            div {
                id: "app-overlay",
                onclick: move |_| {
                    let mut drawer = use_context::<Signal<component::DrawerState>>();
                    drawer.set(component::DrawerState { open: false });
                },
            }
            component::Drawer {
                onlogout: |_| {
                    let mut user = use_context::<Signal<Option<component::account::User>>>();
                    user.set(None);
                    let nav = use_context::<Navigator>();
                    nav.push(Route::Home {});
                },
            }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        div {
            h1 { "MIRAMS" }
            p {
                "MIRAMS: Menhera.org Internet Resources Assignment Management System"
            }
            ul {
                li {
                    Link {
                        to: NavigationTarget::<Route>::External(GITHUB_LINK.to_string()),
                        "GitHub"
                    }
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginResult {
    pub api_token: String,
}

#[component]
fn Login() -> Element {
    let mut error = use_signal(|| None);
    let mut user = use_context::<Signal<Option<component::account::User>>>();

    let log_in = move |attempt: component::account::LoginAttempt| {
        spawn(async move {
            let username = attempt.username.clone();


            let res: Result<LoginResult, _> = fetch::post("/api/v1/login", &attempt, None).await;
            let res = match res {
                Ok(res) => {
                res
                }
                Err(_e) => {
                    error.set(Some(format!("Login failed")));
                    return;
                }
            };

            user.set(Some(component::account::User {
                username,
                api_token: res.api_token,
            }));

            let nav = use_context::<Navigator>();
            nav.replace(Route::Home {});
        });
    };
    rsx! {
        component::account::LoginForm {
            error: error(),
            username: None,
            onlogin: log_in
        }
    }
}

pub fn use_token() -> Option<String> {
    use_context::<Signal<Option<component::account::User>>>().as_ref().map(|u| u.api_token.clone())
}

#[component]
fn AsnSpaceList() -> Element {
    let token = use_token();
    let future = use_resource(move || {
        let token = token.clone();
        async move {
            let api_res = fetch::get::<inet::ApiResponse>("/api/v1/asn/assignment_space", token.as_deref()).await;
            match api_res {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::AsnAssignmentSpaces(spaces)) => {
                            spaces
                        }
                        _ => {
                            vec![]
                        }
                    }
                }
                Err(_) => {
                    vec![]
                }
            }
        }
    });
    match &*future.read_unchecked() {
        Some(spaces) => {
            let table_rows = spaces.iter().map(|space| {
                let assignment = inet::format_asn_range(space.asn_from, space.asn_to);
                component::table::TableRow {
                    assignment,
                    name: space.name.clone(),
                    description: space.description.clone(),
                    visibility: space.space_visibility.to_string(),
                    url: format!("/asn/space/{}/", space.id),
                }
            }).collect::<Vec<_>>();
            let crumbs = vec![component::BreadCrumb {
                name: "Home".to_string(),
                route: Route::Home {},
            }];
            rsx! {
                component::BreadCrumbs { crumbs, title: "ASNs" }
                h1 { "ASN Assignment Spaces" }
                component::table::AssignmentTable { rows: table_rows }
            }
        }
        None => {
            rsx! {
                h1 { "ASN Assignment Spaces" }
                p { "Loading..." }
            }
        }
    }
}

#[component]
fn AsnSpace(space_id: i32) -> Element {
    let token = use_token();
    let future = use_resource(move || {
        let token = token.clone();
        async move {
            let api_res = fetch::get::<inet::ApiResponse>(&format!("/api/v1/asn/assignment_space/{space_id}"), token.as_deref()).await;
            let space = match api_res {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::AsnAssignmentSpace(space)) => {
                            Some(space)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            let pools = fetch::get::<inet::ApiResponse>(&format!("/api/v1/asn/assignment_space/{space_id}/pool"), token.as_deref()).await;
            let pools = match pools {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::AsnAssignmentPools(pools)) => {
                            Some(pools)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            match (space, pools) {
                (Some(space), Some(pools)) => {
                    Some((space, pools))
                }
                _ => {
                    None
                }
            }
        }
    });
    match &*future.read_unchecked() {
        Some(Some((space, pools))) => {
            let table_rows = pools.iter().map(|pool| {
                let assignment = inet::format_asn_range(pool.asn_from, pool.asn_to);
                component::table::TableRow {
                    assignment,
                    name: pool.name.clone(),
                    description: pool.description.clone(),
                    visibility: pool.pool_visibility.to_string(),
                    url: format!("/asn/space/{}/pool/{}/", space_id, pool.id),
                }
            }).collect::<Vec<_>>();
            let assignment = inet::format_asn_range(space.asn_from, space.asn_to);
            let name = space.name.clone();
            let description = space.description.clone();
            let crumbs = vec![component::BreadCrumb {
                name: "Home".to_string(),
                route: Route::Home {},
            }, component::BreadCrumb {
                name: "ASNs".to_string(),
                route: Route::AsnSpaceList {},
            }];
            rsx! {
                component::BreadCrumbs { crumbs, title: "{assignment}" }
                h1 { "Assignment Space: {assignment}" }
                h2 { "{name}" }
                p { "{description}" }
                component::table::AssignmentTable { rows: table_rows }
            }
        }
        _ => {
            rsx! {
                h1 { "Assignment Space" }
                p { "Loading..." }
            }
        }
    }
}

#[component]
fn AsnPool(space_id: i32, pool_id: i32) -> Element {
    let token = use_token();
    let future = use_resource(move || {
        let token = token.clone();
        async move {
            let space = fetch::get::<inet::ApiResponse>(&format!("/api/v1/asn/assignment_space/{space_id}"), token.as_deref()).await;
            let space = match space {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::AsnAssignmentSpace(space)) => {
                            Some(space)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            let api_res = fetch::get::<inet::ApiResponse>(&format!("/api/v1/asn/assignment_space/{space_id}/pool/{pool_id}"), token.as_deref()).await;
            let pool = match api_res {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::AsnAssignmentPool(pool)) => {
                            Some(pool)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            let assignments = fetch::get::<inet::ApiResponse>(&format!("/api/v1/asn/assignment_space/{space_id}/pool/{pool_id}/assignment"), token.as_deref()).await;
            let assignments = match assignments {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::AsnAssignments(assignments)) => {
                            Some(assignments)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            match (space, pool, assignments) {
                (Some(space), Some(pool), Some(assignments)) => {
                    Some((space, pool, assignments))
                }
                _ => {
                    None
                }
            }
        }
    });
    match &*future.read_unchecked() {
        Some(Some((space, pool, assignments))) => {
            let table_rows = assignments.iter().map(|assignment_obj| {
                let assignment = inet::format_asn_range(assignment_obj.asn, assignment_obj.asn);
                component::table::TableRow {
                    assignment,
                    name: assignment_obj.name.clone(),
                    description: assignment_obj.description.clone(),
                    visibility: assignment_obj.assignment_visibility.to_string(),
                    url: format!("/asn/space/{}/pool/{}/assignment/{}/", space_id, pool_id, assignment_obj.id),
                }
            }).collect::<Vec<_>>();
            let name = pool.name.clone();
            let description = pool.description.clone();
            let space = inet::format_asn_range(space.asn_from, space.asn_to);
            let pool = inet::format_asn_range(pool.asn_from, pool.asn_to);
            let crumbs = vec![component::BreadCrumb {
                name: "Home".to_string(),
                route: Route::Home {},
            }, component::BreadCrumb {
                name: "ASNs".to_string(),
                route: Route::AsnSpaceList {},
            }, component::BreadCrumb {
                name: space.clone(),
                route: Route::AsnSpace { space_id },
            }];
            rsx! {
                component::BreadCrumbs { crumbs, title: "{pool}" }
                h1 { "Assignment Pool: {pool}" }
                h2 { "{name}" }
                p { "{description}" }
                component::table::AssignmentTable { rows: table_rows }
            }
        }
        _ => {
            rsx! {
                h1 { "Assignment Pool" }
                p { "Loading..." }
            }
        }
    }
}

#[component]
fn AsnAssignment(space_id: i32, pool_id: i32, assignment_id: i32) -> Element {
    let token = use_token();
    let future = use_resource(move || {
        let token = token.clone();
        async move {
            let space = fetch::get::<inet::ApiResponse>(&format!("/api/v1/asn/assignment_space/{space_id}"), token.as_deref()).await;
            let space = match space {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::AsnAssignmentSpace(space)) => {
                            Some(space)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            let pool = fetch::get::<inet::ApiResponse>(&format!("/api/v1/asn/assignment_space/{space_id}/pool/{pool_id}"), token.as_deref()).await;
            let pool = match pool {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::AsnAssignmentPool(pool)) => {
                            Some(pool)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            let api_res = fetch::get::<inet::ApiResponse>(&format!("/api/v1/asn/assignment_space/{space_id}/pool/{pool_id}/assignment/{assignment_id}"), token.as_deref()).await;
            let assignment = match api_res {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::AsnAssignment(assignment)) => {
                            Some(assignment)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            match (space, pool, assignment) {
                (Some(space), Some(pool), Some(assignment)) => {
                    Some((space, pool, assignment))
                }
                _ => {
                    None
                }
            }
        }
    });
    match &*future.read_unchecked() {
        Some(Some((space, pool, assignment))) => {
            let name = assignment.name.clone();
            let description = assignment.description.clone();
            let space = inet::format_asn_range(space.asn_from, space.asn_to);
            let pool = inet::format_asn_range(pool.asn_from, pool.asn_to);
            let assignment = inet::format_asn_range(assignment.asn, assignment.asn);
            let crumbs = vec![component::BreadCrumb {
                name: "Home".to_string(),
                route: Route::Home {},
            }, component::BreadCrumb {
                name: "ASNs".to_string(),
                route: Route::AsnSpaceList {},
            }, component::BreadCrumb {
                name: space.clone(),
                route: Route::AsnSpace { space_id },
            }, component::BreadCrumb {
                name: pool.clone(),
                route: Route::AsnPool { space_id, pool_id },
            }];
            rsx! {
                component::BreadCrumbs { crumbs, title: "{assignment}" }
                h1 { "Assignment: {assignment}" }
                h2 { "{name}" }
                p { "{description}" }
            }
        }
        _ => {
            rsx! {
                h1 { "Assignment" }
                p { "Loading..." }
            }
        }
    }
}

#[component]
fn Ipv4SpaceList() -> Element {
    let token = use_token();
    let future = use_resource(move || {
        let token = token.clone();
        async move {
            let api_res = fetch::get::<inet::ApiResponse>("/api/v1/ipv4/assignment_space", token.as_deref()).await;
            match api_res {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv4AssignmentSpaces(spaces)) => {
                            spaces
                        }
                        _ => {
                            vec![]
                        }
                    }
                }
                Err(_) => {
                    vec![]
                }
            }
        }
    });
    match &*future.read_unchecked() {
        Some(spaces) => {
            let table_rows = spaces.iter().map(|space| {
                let prefix = inet::format_ipv4_prefix(space.ipv4_prefix, space.ipv4_prefix_len);
                component::table::TableRow {
                    assignment: prefix,
                    name: space.name.clone(),
                    description: space.description.clone(),
                    visibility: space.space_visibility.to_string(),
                    url: format!("/ipv4/space/{}/", space.id),
                }
            }).collect::<Vec<_>>();
            let crumbs = vec![component::BreadCrumb {
                name: "Home".to_string(),
                route: Route::Home {},
            }];
            rsx! {
                component::BreadCrumbs { crumbs, title: "IPv4" }
                h1 { "IPv4 Assignment Spaces" }
                component::table::AssignmentTable { rows: table_rows }
            }
        }
        None => {
            rsx! {
                h1 { "IPv4 Assignment Spaces" }
                p { "Loading..." }
            }
        }
    }
}

#[component]
fn Ipv4Space(space_id: i32) -> Element {
    let token = use_token();
    let future = use_resource(move || {
        let token = token.clone();
        async move {
            let api_res = fetch::get::<inet::ApiResponse>(&format!("/api/v1/ipv4/assignment_space/{space_id}"), token.as_deref()).await;
            let space = match api_res {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv4AssignmentSpace(space)) => {
                            Some(space)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            let pools = fetch::get::<inet::ApiResponse>(&format!("/api/v1/ipv4/assignment_space/{space_id}/pool"), token.as_deref()).await;
            let pools = match pools {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv4AssignmentPools(pools)) => {
                            Some(pools)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            match (space, pools) {
                (Some(space), Some(pools)) => {
                    Some((space, pools))
                }
                _ => {
                    None
                }
            }
        }
    });
    match &*future.read_unchecked() {
        Some(Some((space, pools))) => {
            let table_rows = pools.iter().map(|pool| {
                let assignment = inet::format_ipv4_prefix(pool.ipv4_prefix, pool.ipv4_prefix_len);
                component::table::TableRow {
                    assignment,
                    name: pool.name.clone(),
                    description: pool.description.clone(),
                    visibility: pool.pool_visibility.to_string(),
                    url: format!("/ipv4/space/{}/pool/{}/", space_id, pool.id),
                }
            }).collect::<Vec<_>>();
            let assignment = inet::format_ipv4_prefix(space.ipv4_prefix, space.ipv4_prefix_len);
            let name = space.name.clone();
            let description = space.description.clone();
            let crumbs = vec![component::BreadCrumb {
                name: "Home".to_string(),
                route: Route::Home {},
            }, component::BreadCrumb {
                name: "IPv4".to_string(),
                route: Route::Ipv4SpaceList {},
            }];
            rsx! {
                component::BreadCrumbs { crumbs, title: "{assignment}" }
                h1 { "Assignment Space: {assignment}" }
                h2 { "{name}" }
                p { "{description}" }
                component::table::AssignmentTable { rows: table_rows }
            }
        }
        _ => {
            rsx! {
                h1 { "Assignment Space" }
                p { "Loading..." }
            }
        }
    }
}

#[component]
fn Ipv4Pool(space_id: i32, pool_id: i32) -> Element {
    let token = use_token();
    let future = use_resource(move || {
        let token = token.clone();
        async move {
            let space = fetch::get::<inet::ApiResponse>(&format!("/api/v1/ipv4/assignment_space/{space_id}"), token.as_deref()).await;
            let space = match space {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv4AssignmentSpace(space)) => {
                            Some(space)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            let api_res = fetch::get::<inet::ApiResponse>(&format!("/api/v1/ipv4/assignment_space/{space_id}/pool/{pool_id}"), token.as_deref()).await;
            let pool = match api_res {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv4AssignmentPool(pool)) => {
                            Some(pool)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            let assignments = fetch::get::<inet::ApiResponse>(&format!("/api/v1/ipv4/assignment_space/{space_id}/pool/{pool_id}/assignment"), token.as_deref()).await;
            let assignments = match assignments {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv4Assignments(assignments)) => {
                            Some(assignments)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            match (space, pool, assignments) {
                (Some(space), Some(pool), Some(assignments)) => {
                    Some((space, pool, assignments))
                }
                _ => {
                    None
                }
            }
        }
    });
    match &*future.read_unchecked() {
        Some(Some((space, pool, assignments))) => {
            let table_rows = assignments.iter().map(|assignment_obj| {
                let assignment = inet::format_ipv4_prefix(assignment_obj.ipv4_prefix, assignment_obj.ipv4_prefix_len);
                component::table::TableRow {
                    assignment,
                    name: assignment_obj.name.clone(),
                    description: assignment_obj.description.clone(),
                    visibility: assignment_obj.assignment_visibility.to_string(),
                    url: format!("/ipv4/space/{}/pool/{}/assignment/{}/", space_id, pool_id, assignment_obj.id),
                }
            }).collect::<Vec<_>>();
            let name = pool.name.clone();
            let description = pool.description.clone();
            let space = inet::format_ipv4_prefix(space.ipv4_prefix, space.ipv4_prefix_len);
            let pool = inet::format_ipv4_prefix(pool.ipv4_prefix, pool.ipv4_prefix_len);
            let crumbs = vec![component::BreadCrumb {
                name: "Home".to_string(),
                route: Route::Home {},
            }, component::BreadCrumb {
                name: "IPv4".to_string(),
                route: Route::Ipv4SpaceList {},
            }, component::BreadCrumb {
                name: space.clone(),
                route: Route::Ipv4Space { space_id },
            }];
            rsx! {
                component::BreadCrumbs { crumbs, title: "{pool}" }
                h1 { "Assignment Pool: {pool}" }
                h2 { "{name}" }
                p { "{description}" }
                component::table::AssignmentTable { rows: table_rows }
            }
        }
        _ => {
            rsx! {
                h1 { "Assignment Pool" }
                p { "Loading..." }
            }
        }
    }
}

#[component]
fn Ipv4Assignment(space_id: i32, pool_id: i32, assignment_id: i32) -> Element {
    let token = use_token();
    let future = use_resource(move || {
        let token = token.clone();
        async move {
            let space = fetch::get::<inet::ApiResponse>(&format!("/api/v1/ipv4/assignment_space/{space_id}"), token.as_deref()).await;
            let space = match space {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv4AssignmentSpace(space)) => {
                            Some(space)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            let pool = fetch::get::<inet::ApiResponse>(&format!("/api/v1/ipv4/assignment_space/{space_id}/pool/{pool_id}"), token.as_deref()).await;
            let pool = match pool {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv4AssignmentPool(pool)) => {
                            Some(pool)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            let api_res = fetch::get::<inet::ApiResponse>(&format!("/api/v1/ipv4/assignment_space/{space_id}/pool/{pool_id}/assignment/{assignment_id}"), token.as_deref()).await;
            let assignment = match api_res {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv4Assignment(assignment)) => {
                            Some(assignment)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            match (space, pool, assignment) {
                (Some(space), Some(pool), Some(assignment)) => {
                    Some((space, pool, assignment))
                }
                _ => {
                    None
                }
            }
        }
    });
    match &*future.read_unchecked() {
        Some(Some((space, pool, assignment))) => {
            let name = assignment.name.clone();
            let description = assignment.description.clone();
            let space = inet::format_ipv4_prefix(space.ipv4_prefix, space.ipv4_prefix_len);
            let pool = inet::format_ipv4_prefix(pool.ipv4_prefix, pool.ipv4_prefix_len);
            let assignment = inet::format_ipv4_prefix(assignment.ipv4_prefix, assignment.ipv4_prefix_len);
            let crumbs = vec![component::BreadCrumb {
                name: "Home".to_string(),
                route: Route::Home {},
            }, component::BreadCrumb {
                name: "IPv4".to_string(),
                route: Route::Ipv4SpaceList {},
            }, component::BreadCrumb {
                name: space.clone(),
                route: Route::Ipv4Space { space_id },
            }, component::BreadCrumb {
                name: pool.clone(),
                route: Route::Ipv4Pool { space_id, pool_id },
            }];
            rsx! {
                component::BreadCrumbs { crumbs, title: "{assignment}" }
                h1 { "Assignment: {assignment}" }
                h2 { "{name}" }
                p { "{description}" }
            }
        }
        _ => {
            rsx! {
                h1 { "Assignment" }
                p { "Loading..." }
            }
        }
    }
}

#[component]
fn Ipv6SpaceList() -> Element {
    let token = use_token();
    let future = use_resource(move || {
        let token = token.clone();
        async move {
            let api_res = fetch::get::<inet::ApiResponse>("/api/v1/ipv6/assignment_space", token.as_deref()).await;
            match api_res {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv6AssignmentSpaces(spaces)) => {
                            spaces
                        }
                        _ => {
                            vec![]
                        }
                    }
                }
                Err(_) => {
                    vec![]
                }
            }
        }
    });
    match &*future.read_unchecked() {
        Some(spaces) => {
            let table_rows = spaces.iter().map(|space| {
                let prefix = inet::format_ipv6_prefix(space.ipv6_prefix, space.ipv6_prefix_len);
                component::table::TableRow {
                    assignment: prefix,
                    name: space.name.clone(),
                    description: space.description.clone(),
                    visibility: space.space_visibility.to_string(),
                    url: format!("/ipv6/space/{}/", space.id),
                }
            }).collect::<Vec<_>>();
            let crumbs = vec![component::BreadCrumb {
                name: "Home".to_string(),
                route: Route::Home {},
            }];
            rsx! {
                component::BreadCrumbs { crumbs, title: "IPv6" }
                h1 { "IPv6 Assignment Spaces" }
                component::table::AssignmentTable { rows: table_rows }
            }
        }
        None => {
            rsx! {
                h1 { "IPv6 Assignment Spaces" }
                p { "Loading..." }
            }
        }
    }
}

#[component]
fn Ipv6Space(space_id: i32) -> Element {
    let token = use_token();
    let future = use_resource(move || {
        let token = token.clone();
        async move {
            let api_res = fetch::get::<inet::ApiResponse>(&format!("/api/v1/ipv6/assignment_space/{space_id}"), token.as_deref()).await;
            let space = match api_res {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv6AssignmentSpace(space)) => {
                            Some(space)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            let pools = fetch::get::<inet::ApiResponse>(&format!("/api/v1/ipv6/assignment_space/{space_id}/pool"), token.as_deref()).await;
            let pools = match pools {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv6AssignmentPools(pools)) => {
                            Some(pools)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            match (space, pools) {
                (Some(space), Some(pools)) => {
                    Some((space, pools))
                }
                _ => {
                    None
                }
            }
        }
    });
    match &*future.read_unchecked() {
        Some(Some((space, pools))) => {
            let table_rows = pools.iter().map(|pool| {
                let assignment = inet::format_ipv6_prefix(pool.ipv6_prefix, pool.ipv6_prefix_len);
                component::table::TableRow {
                    assignment,
                    name: pool.name.clone(),
                    description: pool.description.clone(),
                    visibility: pool.pool_visibility.to_string(),
                    url: format!("/ipv6/space/{}/pool/{}/", space_id, pool.id),
                }
            }).collect::<Vec<_>>();
            let assignment = inet::format_ipv6_prefix(space.ipv6_prefix, space.ipv6_prefix_len);
            let name = space.name.clone();
            let description = space.description.clone();
            let crumbs = vec![component::BreadCrumb {
                name: "Home".to_string(),
                route: Route::Home {},
            }, component::BreadCrumb {
                name: "IPv6".to_string(),
                route: Route::Ipv6SpaceList {},
            }];
            rsx! {
                component::BreadCrumbs { crumbs, title: "{assignment}" }
                h1 { "Assignment Space: {assignment}" }
                h2 { "{name}" }
                p { "{description}" }
                component::table::AssignmentTable { rows: table_rows }
            }
        }
        _ => {
            rsx! {
                h1 { "Assignment Space" }
                p { "Loading..." }
            }
        }
    }
}

#[component]
fn Ipv6Pool(space_id: i32, pool_id: i32) -> Element {
    let token = use_token();
    let future = use_resource(move || {
        let token = token.clone();
        async move {
            let space = fetch::get::<inet::ApiResponse>(&format!("/api/v1/ipv6/assignment_space/{space_id}"), token.as_deref()).await;
            let space = match space {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv6AssignmentSpace(space)) => {
                            Some(space)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            let api_res = fetch::get::<inet::ApiResponse>(&format!("/api/v1/ipv6/assignment_space/{space_id}/pool/{pool_id}"), token.as_deref()).await;
            let pool = match api_res {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv6AssignmentPool(pool)) => {
                            Some(pool)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            let assignments = fetch::get::<inet::ApiResponse>(&format!("/api/v1/ipv6/assignment_space/{space_id}/pool/{pool_id}/assignment"), token.as_deref()).await;
            let assignments = match assignments {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv6Assignments(assignments)) => {
                            Some(assignments)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            match (space, pool, assignments) {
                (Some(space), Some(pool), Some(assignments)) => {
                    Some((space, pool, assignments))
                }
                _ => {
                    None
                }
            }
        }
    });
    match &*future.read_unchecked() {
        Some(Some((space, pool, assignments))) => {
            let table_rows = assignments.iter().map(|assignment_obj| {
                let assignment = inet::format_ipv6_prefix(assignment_obj.ipv6_prefix, assignment_obj.ipv6_prefix_len);
                component::table::TableRow {
                    assignment,
                    name: assignment_obj.name.clone(),
                    description: assignment_obj.description.clone(),
                    visibility: assignment_obj.assignment_visibility.to_string(),
                    url: format!("/ipv6/space/{}/pool/{}/assignment/{}/", space_id, pool_id, assignment_obj.id),
                }
            }).collect::<Vec<_>>();
            let name = pool.name.clone();
            let description = pool.description.clone();
            let space = inet::format_ipv6_prefix(space.ipv6_prefix, space.ipv6_prefix_len);
            let pool = inet::format_ipv6_prefix(pool.ipv6_prefix, pool.ipv6_prefix_len);
            let crumbs = vec![component::BreadCrumb {
                name: "Home".to_string(),
                route: Route::Home {},
            }, component::BreadCrumb {
                name: "IPv6".to_string(),
                route: Route::Ipv6SpaceList {},
            }, component::BreadCrumb {
                name: space.clone(),
                route: Route::Ipv6Space { space_id },
            }];
            rsx! {
                component::BreadCrumbs { crumbs, title: "{pool}" }
                h1 { "Assignment Pool: {pool}" }
                h2 { "{name}" }
                p { "{description}" }
                component::table::AssignmentTable { rows: table_rows }
            }
        }
        _ => {
            rsx! {
                h1 { "Assignment Pool" }
                p { "Loading..." }
            }
        }
    }
}

#[component]
fn Ipv6Assignment(space_id: i32, pool_id: i32, assignment_id: i32) -> Element {
    let token = use_token();
    let future = use_resource(move || {
        let token = token.clone();
        async move {
            let space = fetch::get::<inet::ApiResponse>(&format!("/api/v1/ipv6/assignment_space/{space_id}"), token.as_deref()).await;
            let space = match space {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv6AssignmentSpace(space)) => {
                            Some(space)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            let pool = fetch::get::<inet::ApiResponse>(&format!("/api/v1/ipv6/assignment_space/{space_id}/pool/{pool_id}"), token.as_deref()).await;
            let pool = match pool {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv6AssignmentPool(pool)) => {
                            Some(pool)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            let api_res = fetch::get::<inet::ApiResponse>(&format!("/api/v1/ipv6/assignment_space/{space_id}/pool/{pool_id}/assignment/{assignment_id}"), token.as_deref()).await;
            let assignment = match api_res {
                Ok(api_res) => {
                    match api_res.result {
                        Some(inet::ApiResponseVariant::Ipv6Assignment(assignment)) => {
                            Some(assignment)
                        }
                        _ => {
                            None
                        }
                    }
                }
                Err(_) => {
                    None
                }
            };
            match (space, pool, assignment) {
                (Some(space), Some(pool), Some(assignment)) => {
                    Some((space, pool, assignment))
                }
                _ => {
                    None
                }
            }
        }
    });
    match &*future.read_unchecked() {
        Some(Some((space, pool, assignment))) => {
            let name = assignment.name.clone();
            let description = assignment.description.clone();
            let space = inet::format_ipv6_prefix(space.ipv6_prefix, space.ipv6_prefix_len);
            let pool = inet::format_ipv6_prefix(pool.ipv6_prefix, pool.ipv6_prefix_len);
            let assignment = inet::format_ipv6_prefix(assignment.ipv6_prefix, assignment.ipv6_prefix_len);
            let crumbs = vec![component::BreadCrumb {
                name: "Home".to_string(),
                route: Route::Home {},
            }, component::BreadCrumb {
                name: "IPv6".to_string(),
                route: Route::Ipv6SpaceList {},
            }, component::BreadCrumb {
                name: space.clone(),
                route: Route::Ipv6Space { space_id },
            }, component::BreadCrumb {
                name: pool.clone(),
                route: Route::Ipv6Pool { space_id, pool_id },
            }];
            rsx! {
                component::BreadCrumbs { crumbs, title: "{assignment}" }
                h1 { "Assignment: {assignment}" }
                h2 { "{name}" }
                p { "{description}" }
            }
        }
        _ => {
            rsx! {
                h1 { "Assignment" }
                p { "Loading..." }
            }
        }
    }
}
