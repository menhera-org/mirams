#![allow(non_snake_case)]

pub mod component;
pub mod fetch;
pub mod inet;

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};

use serde::{Deserialize, Serialize};

use std::str::FromStr;

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

    #[route("/asn/space/add/")]
    AsnSpaceAdd {},

    #[route("/asn/space/:space_id/")]
    AsnSpace { space_id: i32 },

    #[route("/asn/space/:space_id/pool/add/")]
    AsnPoolAdd { space_id: i32 },

    #[route("/asn/space/:space_id/pool/:pool_id/")]
    AsnPool { space_id: i32, pool_id: i32 },

    #[route("/asn/space/:space_id/pool/:pool_id/assignment/add/")]
    AsnAssignmentAdd { space_id: i32, pool_id: i32 },

    #[route("/asn/space/:space_id/pool/:pool_id/assignment/:assignment_id/")]
    AsnAssignment { space_id: i32, pool_id: i32, assignment_id: i32 },


    // IPv4 assignments

    #[route("/ipv4/")]
    Ipv4SpaceList {},

    #[route("/ipv4/space/add/")]
    Ipv4SpaceAdd {},

    #[route("/ipv4/space/:space_id/")]
    Ipv4Space { space_id: i32 },

    #[route("/ipv4/space/:space_id/pool/add/")]
    Ipv4PoolAdd { space_id: i32 },

    #[route("/ipv4/space/:space_id/pool/:pool_id/")]
    Ipv4Pool { space_id: i32, pool_id: i32 },

    #[route("/ipv4/space/:space_id/pool/:pool_id/assignment/add/")]
    Ipv4AssignmentAdd { space_id: i32, pool_id: i32 },

    #[route("/ipv4/space/:space_id/pool/:pool_id/assignment/:assignment_id/")]
    Ipv4Assignment { space_id: i32, pool_id: i32, assignment_id: i32 },


    // IPv6 assignments

    #[route("/ipv6/")]
    Ipv6SpaceList {},

    #[route("/ipv6/space/add/")]
    Ipv6SpaceAdd {},

    #[route("/ipv6/space/:space_id/")]
    Ipv6Space { space_id: i32 },

    #[route("/ipv6/space/:space_id/pool/add/")]
    Ipv6PoolAdd { space_id: i32 },

    #[route("/ipv6/space/:space_id/pool/:pool_id/")]
    Ipv6Pool { space_id: i32, pool_id: i32 },

    #[route("/ipv6/space/:space_id/pool/:pool_id/assignment/add/")]
    Ipv6AssignmentAdd { space_id: i32, pool_id: i32 },

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
    let user = use_context::<Signal<Option<component::account::User>>>();
    let signed_in = user().is_some();

    let mut class = if drawer_open { "app-drawer-open" } else { "app-drawer-closed" }.to_string();
    if signed_in {
        class.push_str(" app-signed-in");
    } else {
        class.push_str(" app-signed-out");
    }

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
            class: class,
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
            h2 { "Assignment Spaces" }
            ul {
                li {
                    Link {
                        to: Route::AsnSpaceList {},
                        "ASN Assignment Spaces"
                    }
                }
                li {
                    Link {
                        to: Route::Ipv4SpaceList {},
                        "IPv4 Assignment Spaces"
                    }
                }
                li {
                    Link {
                        to: Route::Ipv6SpaceList {},
                        "IPv6 Assignment Spaces"
                    }
                }
            }
            h2 { "About MIRAMS" }
            p {
                "MIRAMS is a system for managing the assignment of Internet resources, such as ASNs, IPv4 and IPv6 address space."
            }
            p {
                "MIRAMS is Free Software, and is available on GitHub."
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
                component::AddButtonToolbar {
                    add_button_text: "Add ASN Assignment Space",
                    add_button_route: Route::AsnSpaceAdd {},
                }
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
    let mut delete_popup_shown = use_signal(|| false);
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
                component::MetadataForm {
                    name: name.clone(),
                    description: description.clone(),
                    onsubmit: move |metadata| {
                        let token = use_token();
                        let nav = use_context::<Navigator>();
                        spawn(async move {
                            let _ = fetch::put::<inet::ApiResponse, component::MetadataUpdateRequest>(&format!("/api/v1/asn/assignment_space/{space_id}"), metadata, token.as_deref()).await;
                            nav.replace(Route::AsnSpace { space_id });
                        });
                    }
                }
                component::AddButtonToolbar {
                    add_button_text: "Add ASN Assignment Pool",
                    add_button_route: Route::AsnPoolAdd { space_id },
                }
                component::table::AssignmentTable { rows: table_rows }
                div {
                    class: "delete-toolbar",
                    if delete_popup_shown() {
                        div {
                            class: "delete-popup",
                            p { "Are you sure you want to delete this assignment space?" }
                            button {
                                class: "delete-button",
                                onclick: move |_| {
                                    let token = use_token();
                                    spawn(async move {
                                        let _ = fetch::delete::<inet::ApiResponse>(&format!("/api/v1/asn/assignment_space/{space_id}"), token.as_deref()).await;
                                        let nav = use_context::<Navigator>();
                                        nav.replace(Route::AsnSpaceList {});
                                    });
                                },
                                "Yes"
                            }
                            button {
                                class: "cancel-button",
                                onclick: move |_| {
                                    delete_popup_shown.set(false);
                                },
                                "No"
                            }
                        }
                    } else {
                        button {
                            class: "delete-button",
                            onclick: move |_| {
                                delete_popup_shown.set(true);
                            },
                            "Delete Assignment Space"
                        }
                    }
                }
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
    let mut delete_popup_shown = use_signal(|| false);
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
                component::MetadataForm {
                    name: name.clone(),
                    description: description.clone(),
                    onsubmit: move |metadata| {
                        let token = use_token();
                        let nav = use_context::<Navigator>();
                        spawn(async move {
                            let _ = fetch::put::<inet::ApiResponse, component::MetadataUpdateRequest>(&format!("/api/v1/asn/assignment_space/{space_id}/pool/{pool_id}"), metadata, token.as_deref()).await;
                            nav.replace(Route::AsnPool { space_id, pool_id });
                        });
                    }
                }
                component::AddButtonToolbar {
                    add_button_text: "Add ASN Assignment",
                    add_button_route: Route::AsnAssignmentAdd { space_id, pool_id },
                }
                component::table::AssignmentTable { rows: table_rows }
                div {
                    class: "delete-toolbar",
                    if delete_popup_shown() {
                        div {
                            class: "delete-popup",
                            p { "Are you sure you want to delete this assignment pool?" }
                            button {
                                class: "delete-button",
                                onclick: move |_| {
                                    let token = use_token();
                                    spawn(async move {
                                        let _ = fetch::delete::<inet::ApiResponse>(&format!("/api/v1/asn/assignment_space/{space_id}/pool/{pool_id}"), token.as_deref()).await;
                                        let nav = use_context::<Navigator>();
                                        nav.replace(Route::AsnSpace { space_id });
                                    });
                                },
                                "Yes"
                            }
                            button {
                                class: "cancel-button",
                                onclick: move |_| {
                                    delete_popup_shown.set(false);
                                },
                                "No"
                            }
                        }
                    } else {
                        button {
                            class: "delete-button",
                            onclick: move |_| {
                                delete_popup_shown.set(true);
                            },
                            "Delete Assignment Pool"
                        }
                    }
                }
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
    let mut delete_popup_shown = use_signal(|| false);
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
                component::MetadataForm {
                    name: name.clone(),
                    description: description.clone(),
                    onsubmit: move |metadata| {
                        let token = use_token();
                        let nav = use_context::<Navigator>();
                        spawn(async move {
                            let _ = fetch::put::<inet::ApiResponse, component::MetadataUpdateRequest>(&format!("/api/v1/asn/assignment_space/{space_id}/pool/{pool_id}/assignment/{assignment_id}"), metadata, token.as_deref()).await;
                            nav.replace(Route::AsnAssignment { space_id, pool_id, assignment_id });
                        });
                    }
                }
                div {
                    class: "delete-toolbar",
                    if delete_popup_shown() {
                        div {
                            class: "delete-popup",
                            p { "Are you sure you want to delete this assignment?" }
                            button {
                                class: "delete-button",
                                onclick: move |_| {
                                    let token = use_token();
                                    spawn(async move {
                                        let _ = fetch::delete::<inet::ApiResponse>(&format!("/api/v1/asn/assignment_space/{space_id}/pool/{pool_id}/assignment/{assignment_id}"), token.as_deref()).await;
                                        let nav = use_context::<Navigator>();
                                        nav.replace(Route::AsnPool { space_id, pool_id });
                                    });
                                },
                                "Yes"
                            }
                            button {
                                class: "cancel-button",
                                onclick: move |_| {
                                    delete_popup_shown.set(false);
                                },
                                "No"
                            }
                        }
                    } else {
                        button {
                            class: "delete-button",
                            onclick: move |_| {
                                delete_popup_shown.set(true);
                            },
                            "Delete Assignment"
                        }
                    }
                }
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
                component::AddButtonToolbar {
                    add_button_text: "Add IPv4 Assignment Space",
                    add_button_route: Route::Ipv4SpaceAdd {},
                }
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
    let mut delete_popup_shown = use_signal(|| false);
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
                component::MetadataForm {
                    name: name.clone(),
                    description: description.clone(),
                    onsubmit: move |metadata| {
                        let token = use_token();
                        let nav = use_context::<Navigator>();
                        spawn(async move {
                            let _ = fetch::put::<inet::ApiResponse, component::MetadataUpdateRequest>(&format!("/api/v1/ipv4/assignment_space/{space_id}"), metadata, token.as_deref()).await;
                            nav.replace(Route::Ipv4Space { space_id });
                        });
                    }
                }
                component::AddButtonToolbar {
                    add_button_text: "Add IPv4 Assignment Pool",
                    add_button_route: Route::Ipv4PoolAdd { space_id },
                }
                component::table::AssignmentTable { rows: table_rows }
                div {
                    class: "delete-toolbar",
                    if delete_popup_shown() {
                        div {
                            class: "delete-popup",
                            p { "Are you sure you want to delete this assignment space?" }
                            button {
                                class: "delete-button",
                                onclick: move |_| {
                                    let token = use_token();
                                    spawn(async move {
                                        let _ = fetch::delete::<inet::ApiResponse>(&format!("/api/v1/ipv4/assignment_space/{space_id}"), token.as_deref()).await;
                                        let nav = use_context::<Navigator>();
                                        nav.replace(Route::Ipv4SpaceList {});
                                    });
                                },
                                "Yes"
                            }
                            button {
                                class: "cancel-button",
                                onclick: move |_| {
                                    delete_popup_shown.set(false);
                                },
                                "No"
                            }
                        }
                    } else {
                        button {
                            class: "delete-button",
                            onclick: move |_| {
                                delete_popup_shown.set(true);
                            },
                            "Delete Assignment Space"
                        }
                    }
                }
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
    let mut delete_popup_shown = use_signal(|| false);
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
                component::MetadataForm {
                    name: name.clone(),
                    description: description.clone(),
                    onsubmit: move |metadata| {
                        let token = use_token();
                        let nav = use_context::<Navigator>();
                        spawn(async move {
                            let _ = fetch::put::<inet::ApiResponse, component::MetadataUpdateRequest>(&format!("/api/v1/ipv4/assignment_space/{space_id}/pool/{pool_id}"), metadata, token.as_deref()).await;
                            nav.replace(Route::Ipv4Pool { space_id, pool_id });
                        });
                    }
                }
                component::AddButtonToolbar {
                    add_button_text: "Add IPv4 Assignment",
                    add_button_route: Route::Ipv4AssignmentAdd { space_id, pool_id },
                }
                component::table::AssignmentTable { rows: table_rows }
                div {
                    class: "delete-toolbar",
                    if delete_popup_shown() {
                        div {
                            class: "delete-popup",
                            p { "Are you sure you want to delete this assignment pool?" }
                            button {
                                class: "delete-button",
                                onclick: move |_| {
                                    let token = use_token();
                                    spawn(async move {
                                        let _ = fetch::delete::<inet::ApiResponse>(&format!("/api/v1/ipv4/assignment_space/{space_id}/pool/{pool_id}"), token.as_deref()).await;
                                        let nav = use_context::<Navigator>();
                                        nav.replace(Route::Ipv4Space { space_id });
                                    });
                                },
                                "Yes"
                            }
                            button {
                                class: "cancel-button",
                                onclick: move |_| {
                                    delete_popup_shown.set(false);
                                },
                                "No"
                            }
                        }
                    } else {
                        button {
                            class: "delete-button",
                            onclick: move |_| {
                                delete_popup_shown.set(true);
                            },
                            "Delete Assignment Pool"
                        }
                    }
                }
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
    let mut delete_popup_shown = use_signal(|| false);
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
                component::MetadataForm {
                    name: name.clone(),
                    description: description.clone(),
                    onsubmit: move |metadata| {
                        let token = use_token();
                        let nav = use_context::<Navigator>();
                        spawn(async move {
                            let _ = fetch::put::<inet::ApiResponse, component::MetadataUpdateRequest>(&format!("/api/v1/ipv4/assignment_space/{space_id}/pool/{pool_id}/assignment/{assignment_id}"), metadata, token.as_deref()).await;
                            nav.replace(Route::Ipv4Assignment { space_id, pool_id, assignment_id });
                        });
                    }
                }
                div {
                    class: "delete-toolbar",
                    if delete_popup_shown() {
                        div {
                            class: "delete-popup",
                            p { "Are you sure you want to delete this assignment?" }
                            button {
                                class: "delete-button",
                                onclick: move |_| {
                                    let token = use_token();
                                    spawn(async move {
                                        let _ = fetch::delete::<inet::ApiResponse>(&format!("/api/v1/ipv4/assignment_space/{space_id}/pool/{pool_id}/assignment/{assignment_id}"), token.as_deref()).await;
                                        let nav = use_context::<Navigator>();
                                        nav.replace(Route::Ipv4Pool { space_id, pool_id });
                                    });
                                },
                                "Yes"
                            }
                            button {
                                class: "cancel-button",
                                onclick: move |_| {
                                    delete_popup_shown.set(false);
                                },
                                "No"
                            }
                        }
                    } else {
                        button {
                            class: "delete-button",
                            onclick: move |_| {
                                delete_popup_shown.set(true);
                            },
                            "Delete Assignment"
                        }
                    }
                }
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
                component::AddButtonToolbar {
                    add_button_text: "Add IPv6 Assignment Space",
                    add_button_route: Route::Ipv6SpaceAdd {},
                }
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
    let mut delete_popup_shown = use_signal(|| false);
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
                component::MetadataForm {
                    name: name.clone(),
                    description: description.clone(),
                    onsubmit: move |metadata| {
                        let token = use_token();
                        let nav = use_context::<Navigator>();
                        spawn(async move {
                            let _ = fetch::put::<inet::ApiResponse, component::MetadataUpdateRequest>(&format!("/api/v1/ipv6/assignment_space/{space_id}"), metadata, token.as_deref()).await;
                            nav.replace(Route::Ipv6Space { space_id });
                        });
                    }
                }
                component::AddButtonToolbar {
                    add_button_text: "Add IPv6 Assignment Pool",
                    add_button_route: Route::Ipv6PoolAdd { space_id },
                }
                component::table::AssignmentTable { rows: table_rows }
                div {
                    class: "delete-toolbar",
                    if delete_popup_shown() {
                        div {
                            class: "delete-popup",
                            p { "Are you sure you want to delete this assignment space?" }
                            button {
                                class: "delete-button",
                                onclick: move |_| {
                                    let token = use_token();
                                    spawn(async move {
                                        let _ = fetch::delete::<inet::ApiResponse>(&format!("/api/v1/ipv6/assignment_space/{space_id}"), token.as_deref()).await;
                                        let nav = use_context::<Navigator>();
                                        nav.replace(Route::Ipv6SpaceList {});
                                    });
                                },
                                "Yes"
                            }
                            button {
                                class: "cancel-button",
                                onclick: move |_| {
                                    delete_popup_shown.set(false);
                                },
                                "No"
                            }
                        }
                    } else {
                        button {
                            class: "delete-button",
                            onclick: move |_| {
                                delete_popup_shown.set(true);
                            },
                            "Delete Assignment Space"
                        }
                    }
                }
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
    let mut delete_popup_shown = use_signal(|| false);
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
                component::MetadataForm {
                    name: name.clone(),
                    description: description.clone(),
                    onsubmit: move |metadata| {
                        let token = use_token();
                        let nav = use_context::<Navigator>();
                        spawn(async move {
                            let _ = fetch::put::<inet::ApiResponse, component::MetadataUpdateRequest>(&format!("/api/v1/ipv6/assignment_space/{space_id}/pool/{pool_id}"), metadata, token.as_deref()).await;
                            nav.replace(Route::Ipv6Pool { space_id, pool_id });
                        });
                    }
                }
                component::AddButtonToolbar {
                    add_button_text: "Add IPv6 Assignment",
                    add_button_route: Route::Ipv6AssignmentAdd { space_id, pool_id },
                }
                component::table::AssignmentTable { rows: table_rows }
                div {
                    class: "delete-toolbar",
                    if delete_popup_shown() {
                        div {
                            class: "delete-popup",
                            p { "Are you sure you want to delete this assignment pool?" }
                            button {
                                class: "delete-button",
                                onclick: move |_| {
                                    let token = use_token();
                                    spawn(async move {
                                        let _ = fetch::delete::<inet::ApiResponse>(&format!("/api/v1/ipv6/assignment_space/{space_id}/pool/{pool_id}"), token.as_deref()).await;
                                        let nav = use_context::<Navigator>();
                                        nav.replace(Route::Ipv6Space { space_id });
                                    });
                                },
                                "Yes"
                            }
                            button {
                                class: "cancel-button",
                                onclick: move |_| {
                                    delete_popup_shown.set(false);
                                },
                                "No"
                            }
                        }
                    } else {
                        button {
                            class: "delete-button",
                            onclick: move |_| {
                                delete_popup_shown.set(true);
                            },
                            "Delete Assignment Pool"
                        }
                    }
                }
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
    let mut delete_popup_shown = use_signal(|| false);
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
                component::MetadataForm {
                    name: name.clone(),
                    description: description.clone(),
                    onsubmit: move |metadata| {
                        let token = use_token();
                        let nav = use_context::<Navigator>();
                        spawn(async move {
                            let _ = fetch::put::<inet::ApiResponse, component::MetadataUpdateRequest>(&format!("/api/v1/ipv6/assignment_space/{space_id}/pool/{pool_id}/assignment/{assignment_id}"), metadata, token.as_deref()).await;
                            nav.replace(Route::Ipv6Assignment { space_id, pool_id, assignment_id });
                        });
                    }
                }
                div {
                    class: "delete-toolbar",
                    if delete_popup_shown() {
                        div {
                            class: "delete-popup",
                            p { "Are you sure you want to delete this assignment?" }
                            button {
                                class: "delete-button",
                                onclick: move |_| {
                                    let token = use_token();
                                    spawn(async move {
                                        let _ = fetch::delete::<inet::ApiResponse>(&format!("/api/v1/ipv6/assignment_space/{space_id}/pool/{pool_id}/assignment/{assignment_id}"), token.as_deref()).await;
                                        let nav = use_context::<Navigator>();
                                        nav.replace(Route::Ipv6Pool { space_id, pool_id });
                                    });
                                },
                                "Yes"
                            }
                            button {
                                class: "cancel-button",
                                onclick: move |_| {
                                    delete_popup_shown.set(false);
                                },
                                "No"
                            }
                        }
                    } else {
                        button {
                            class: "delete-button",
                            onclick: move |_| {
                                delete_popup_shown.set(true);
                            },
                            "Delete Assignment"
                        }
                    }
                }
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
fn AsnSpaceAdd() -> Element {
    let token = use_token();
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut asn_from = use_signal(|| String::new());
    let mut asn_to = use_signal(|| String::new());
    let mut visibility = use_signal(|| String::from("Public"));
    let mut error = use_signal(|| None);

    let add_space = move |_| {
        let token = token.clone();
        let name = name().trim().to_owned();
        let description = description().trim().to_owned();
        let mut asn_from = asn_from().trim().to_owned();
        let mut asn_to = asn_to().trim().to_owned();
        let visibility = visibility().clone();

        if name.is_empty() || asn_from.is_empty() || asn_to.is_empty() || visibility.is_empty() {
            error.set(Some("All fields are required".to_string()));
            return;
        }

        if asn_from.starts_with("AS") {
            asn_from = asn_from[2..].to_string();
        }

        if asn_to.starts_with("AS") {
            asn_to = asn_to[2..].to_string();
        }

        let asn_from: u32 = match asn_from.parse() {
            Ok(val) => val,
            Err(_) => {
                error.set(Some("Invalid ASN from value".to_string()));
                return;
            }
        };

        let asn_to: u32 = match asn_to.parse() {
            Ok(val) => val,
            Err(_) => {
                error.set(Some("Invalid ASN to value".to_string()));
                return;
            }
        };

        if asn_from > asn_to {
            error.set(Some("ASN from must be less than ASN to".to_string()));
            return;
        }

        let visibility = match visibility.as_str() {
            "Public" | "Private" => visibility,
            _ => {
                error.set(Some("Invalid visibility value".to_string()));
                return;
            }
        };

        let new_space = inet::AssignmentSpaceAsn {
            id: 0, // This will be set by the server
            name,
            description,
            asn_from,
            asn_to,
            space_visibility: inet::ObjectVisibility::from_str(&visibility).unwrap(),
        };

        spawn(async move {
            let res: Result<inet::ApiResponse, _> = fetch::post("/api/v1/asn/assignment_space", &new_space, token.as_deref()).await;
            match res {
                Ok(inet::ApiResponse { error: None, result: _ }) => {
                    let nav = use_context::<Navigator>();
                    nav.push(Route::AsnSpaceList {});
                }
                Ok(inet::ApiResponse { error: Some(err), result: _ }) => {
                    error.set(Some(err));
                }
                Err(_) => {
                    error.set(Some("Failed to add ASN space".to_string()));
                }
            }
        });
    };

    rsx! {
        div {
            h1 { "Add ASN Assignment Space" }
            if let Some(err) = error() {
                p { style: "color: red;", "{err}" }
            }
            component::TextInput {
                placeholder: "Name",
                value: "{name}",
                oninput: move |e: Event<FormData>| name.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "Description",
                value: "{description}",
                oninput: move |e: Event<FormData>| description.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "ASN From",
                value: "{asn_from}",
                oninput: move |e: Event<FormData>| asn_from.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "ASN To",
                value: "{asn_to}",
                oninput: move |e: Event<FormData>| asn_to.set(e.value().clone()),
            }
            label {
                class: "select-label",
                "Visibility"
                select {
                    value: "{visibility}",
                    oninput: move |e| visibility.set(e.value().clone()),
                    option { "Public" }
                    option { "Private" }
                }
            }
            button {
                onclick: add_space,
                "Add ASN Space"
            }
        }
    }
}

#[component]
fn AsnPoolAdd(space_id: i32) -> Element {
    let token = use_token();
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut asn_from = use_signal(|| String::new());
    let mut asn_to = use_signal(|| String::new());
    let mut visibility = use_signal(|| String::from("Public"));
    let mut error = use_signal(|| None);

    let add_pool = move |_| {
        let space_id = space_id;
        let token = token.clone();
        let name = name().trim().to_owned();
        let description = description().trim().to_owned();
        let mut asn_from = asn_from().trim().to_owned();
        let mut asn_to = asn_to().trim().to_owned();
        let visibility = visibility().clone();

        if name.is_empty() || asn_from.is_empty() || asn_to.is_empty() || visibility.is_empty() {
            error.set(Some("All fields are required".to_string()));
            return;
        }

        if asn_from.starts_with("AS") {
            asn_from = asn_from[2..].to_string();
        }

        if asn_to.starts_with("AS") {
            asn_to = asn_to[2..].to_string();
        }

        let asn_from: u32 = match asn_from.parse() {
            Ok(val) => val,
            Err(_) => {
                error.set(Some("Invalid ASN from value".to_string()));
                return;
            }
        };

        let asn_to: u32 = match asn_to.parse() {
            Ok(val) => val,
            Err(_) => {
                error.set(Some("Invalid ASN to value".to_string()));
                return;
            }
        };

        if asn_from > asn_to {
            error.set(Some("ASN from must be less than ASN to".to_string()));
            return;
        }

        let visibility = match visibility.as_str() {
            "Public" | "Private" => visibility,
            _ => {
                error.set(Some("Invalid visibility value".to_string()));
                return;
            }
        };

        let new_pool = inet::AssignmentPoolAsn {
            id: 0, // This will be set by the server
            name,
            description,
            asn_from,
            asn_to,
            pool_visibility: inet::ObjectVisibility::from_str(&visibility).unwrap(),
            assignment_space_id: space_id,
        };

        spawn(async move {
            let res: Result<inet::ApiResponse, _> = fetch::post(&format!("/api/v1/asn/assignment_space/{space_id}/pool", space_id=space_id), &new_pool, token.as_deref()).await;
            match res {
                Ok(inet::ApiResponse { error: None, result: _ }) => {
                    let nav = use_context::<Navigator>();
                    nav.push(Route::AsnSpace { space_id });
                }
                Ok(inet::ApiResponse { error: Some(err), result: _ }) => {
                    error.set(Some(err));
                }
                Err(_) => {
                    error.set(Some("Failed to add ASN pool".to_string()));
                }
            }
        });
    };

    rsx! {
        div {
            h1 { "Add ASN Assignment Pool" }
            if let Some(err) = error() {
                p { style: "color: red;", "{err}" }
            }
            component::TextInput {
                placeholder: "Name",
                value: "{name}",
                oninput: move |e: Event<FormData>| name.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "Description",
                value: "{description}",
                oninput: move |e: Event<FormData>| description.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "ASN From",
                value: "{asn_from}",
                oninput: move |e: Event<FormData>| asn_from.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "ASN To",
                value: "{asn_to}",
                oninput: move |e: Event<FormData>| asn_to.set(e.value().clone()),
            }
            label {
                class: "select-label",
                "Visibility"
                select {
                    value: "{visibility}",
                    oninput: move |e| visibility.set(e.value().clone()),
                    option { "Public" }
                    option { "Private" }
                }
            }
            button {
                onclick: add_pool,
                "Add ASN Pool"
            }
        }
    }
}

#[component]
fn AsnAssignmentAdd(space_id: i32, pool_id: i32) -> Element {
    let token = use_token();
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut asn = use_signal(|| String::new());
    let mut visibility = use_signal(|| String::from("Public"));
    let mut error = use_signal(|| None);

    let add_assignment = move |_| {
        let space_id = space_id;
        let pool_id = pool_id;
        let token = token.clone();
        let name = name().trim().to_owned();
        let description = description().trim().to_owned();
        let mut asn = asn().trim().to_owned();
        let visibility = visibility().clone();

        if name.is_empty() || asn.is_empty() || visibility.is_empty() {
            error.set(Some("All fields are required".to_string()));
            return;
        }

        if asn.starts_with("AS") {
            asn = asn[2..].to_string();
        }

        let asn: u32 = match asn.parse() {
            Ok(val) => val,
            Err(_) => {
                error.set(Some("Invalid ASN value".to_string()));
                return;
            }
        };

        let visibility = match visibility.as_str() {
            "Public" | "Private" => visibility,
            _ => {
                error.set(Some("Invalid visibility value".to_string()));
                return;
            }
        };

        let new_assignment = inet::AssignmentAsn {
            id: 0, // This will be set by the server
            name,
            description,
            asn,
            assignment_visibility: inet::ObjectVisibility::from_str(&visibility).unwrap(),
            assignment_pool_id: pool_id,
        };

        spawn(async move {
            let res: Result<inet::ApiResponse, _> = fetch::post(&format!("/api/v1/asn/assignment_space/{space_id}/pool/{pool_id}/assignment", space_id=space_id, pool_id=pool_id), &new_assignment, token.as_deref()).await;
            match res {
                Ok(inet::ApiResponse { error: None, result: _ }) => {
                    let nav = use_context::<Navigator>();
                    nav.push(Route::AsnPool { space_id, pool_id });
                }
                Ok(inet::ApiResponse { error: Some(err), result: _ }) => {
                    error.set(Some(err));
                }
                Err(_) => {
                    error.set(Some("Failed to add ASN assignment".to_string()));
                }
            }
        });
    };

    rsx! {
        div {
            h1 { "Add ASN Assignment" }
            if let Some(err) = error() {
                p { style: "color: red;", "{err}" }
            }
            component::TextInput {
                placeholder: "Name",
                value: "{name}",
                oninput: move |e: Event<FormData>| name.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "Description",
                value: "{description}",
                oninput: move |e: Event<FormData>| description.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "ASN",
                value: "{asn}",
                oninput: move |e: Event<FormData>| asn.set(e.value().clone()),
            }
            label {
                class: "select-label",
                "Visibility"
                select {
                    value: "{visibility}",
                    oninput: move |e| visibility.set(e.value().clone()),
                    option { "Public" }
                    option { "Private" }
                }
            }
            button {
                onclick: add_assignment,
                "Add ASN Assignment"
            }
        }
    }
}

#[component]
fn Ipv4SpaceAdd() -> Element {
    let token = use_token();
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut ipv4_prefix = use_signal(|| String::new());
    let mut ipv4_prefix_len = use_signal(|| String::new());
    let mut visibility = use_signal(|| String::from("Public"));
    let mut error = use_signal(|| None);

    let add_space = move |_| {
        let token = token.clone();
        let name = name().trim().to_owned();
        let description = description().trim().to_owned();
        let ipv4_prefix = ipv4_prefix().trim().to_owned();
        let ipv4_prefix_len = ipv4_prefix_len().trim().to_owned();

        if name.is_empty() || ipv4_prefix.is_empty() || ipv4_prefix_len.is_empty() {
            error.set(Some("All fields are required".to_string()));
            return;
        }

        let ipv4_prefix_len = match ipv4_prefix_len.parse::<i32>() {
            Ok(val) => val,
            Err(_) => {
                error.set(Some("Invalid IPv4 prefix length value".to_string()));
                return;
            }
        };

        let prefix_obj = inet::Ipv4Prefix::new(&ipv4_prefix, ipv4_prefix_len);
        let (ipv4_prefix, ipv4_prefix_len) = match prefix_obj {
            Ok(prefix_obj) => (prefix_obj.prefix_octets(), prefix_obj.prefix_len()),
            Err(s) => {
                error.set(Some(s.clone()));
                return;
            }
        };

        let visibility = visibility().clone();

        let visibility = match visibility.as_str() {
            "Public" | "Private" => visibility,
            _ => {
                error.set(Some("Invalid visibility value".to_string()));
                return;
            }
        };

        let new_space = inet::AssignmentSpaceIpv4 {
            id: 0, // This will be set by the server
            name,
            description,
            ipv4_prefix,
            ipv4_prefix_len,
            space_visibility: inet::ObjectVisibility::from_str(&visibility).unwrap(),
        };

        spawn(async move {
            let res: Result<inet::ApiResponse, _> = fetch::post("/api/v1/ipv4/assignment_space", &new_space, token.as_deref()).await;
            match res {
                Ok(inet::ApiResponse { error: None, result: _ }) => {
                    let nav = use_context::<Navigator>();
                    nav.push(Route::Ipv4SpaceList {});
                }
                Ok(inet::ApiResponse { error: Some(err), result: _ }) => {
                    error.set(Some(err));
                }
                Err(_) => {
                    error.set(Some("Failed to add IPv4 space".to_string()));
                }
            }
        });
    };

    rsx! {
        div {
            h1 { "Add IPv4 Assignment Space" }
            if let Some(err) = error() {
                p { style: "color: red;", "{err}" }
            }
            component::TextInput {
                placeholder: "Name",
                value: "{name}",
                oninput: move |e: Event<FormData>| name.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "Description",
                value: "{description}",
                oninput: move |e: Event<FormData>| description.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "IPv4 Prefix",
                value: "{ipv4_prefix}",
                oninput: move |e: Event<FormData>| ipv4_prefix.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "IPv4 Prefix Length",
                value: "{ipv4_prefix_len}",
                oninput: move |e: Event<FormData>| ipv4_prefix_len.set(e.value().clone()),
            }
            label {
                class: "select-label",
                "Visibility"
                select {
                    value: "{visibility}",
                    oninput: move |e| visibility.set(e.value().clone()),
                    option { "Public" }
                    option { "Private" }
                }
            }
            button {
                onclick: add_space,
                "Add IPv4 Space"
            }
        }
    }
}

#[component]
fn Ipv4PoolAdd(space_id: i32) -> Element {
    let token = use_token();
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut ipv4_prefix = use_signal(|| String::new());
    let mut ipv4_prefix_len = use_signal(|| String::new());
    let mut visibility = use_signal(|| String::from("Public"));
    let mut error = use_signal(|| None);

    let add_pool = move |_| {
        let space_id = space_id;
        let token = token.clone();
        let name = name().trim().to_owned();
        let description = description().trim().to_owned();
        let ipv4_prefix = ipv4_prefix().trim().to_owned();
        let ipv4_prefix_len = ipv4_prefix_len().trim().to_owned();
        let visibility = visibility().clone();

        if name.is_empty() || ipv4_prefix.is_empty() || ipv4_prefix_len.is_empty() || visibility.is_empty() {
            error.set(Some("All fields are required".to_string()));
            return;
        }

        let ipv4_prefix_len = match ipv4_prefix_len.parse::<i32>() {
            Ok(val) => val,
            Err(_) => {
                error.set(Some("Invalid IPv4 prefix length value".to_string()));
                return;
            }
        };

        let prefix_obj = inet::Ipv4Prefix::new(&ipv4_prefix, ipv4_prefix_len);
        let (ipv4_prefix, ipv4_prefix_len) = match prefix_obj {
            Ok(prefix_obj) => (prefix_obj.prefix_octets(), prefix_obj.prefix_len()),
            Err(s) => {
                error.set(Some(s.clone()));
                return;
            }
        };

        let visibility = match visibility.as_str() {
            "Public" | "Private" => visibility,
            _ => {
                error.set(Some("Invalid visibility value".to_string()));
                return;
            }
        };

        let new_pool = inet::AssignmentPoolIpv4 {
            id: 0, // This will be set by the server
            name,
            description,
            ipv4_prefix,
            ipv4_prefix_len,
            pool_visibility: inet::ObjectVisibility::from_str(&visibility).unwrap(),
            assignment_space_id: space_id,
        };

        spawn(async move {
            let res: Result<inet::ApiResponse, _> = fetch::post(&format!("/api/v1/ipv4/assignment_space/{space_id}/pool", space_id=space_id), &new_pool, token.as_deref()).await;
            match res {
                Ok(inet::ApiResponse { error: None, result: _ }) => {
                    let nav = use_context::<Navigator>();
                    nav.push(Route::Ipv4Space { space_id });
                }
                Ok(inet::ApiResponse { error: Some(err), result: _ }) => {
                    error.set(Some(err));
                }
                Err(_) => {
                    error.set(Some("Failed to add IPv4 pool".to_string()));
                }
            }
        });
    };

    rsx! {
        div {
            h1 { "Add IPv4 Assignment Pool" }
            if let Some(err) = error() {
                p { style: "color: red;", "{err}" }
            }
            component::TextInput {
                placeholder: "Name",
                value: "{name}",
                oninput: move |e: Event<FormData>| name.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "Description",
                value: "{description}",
                oninput: move |e: Event<FormData>| description.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "IPv4 Prefix",
                value: "{ipv4_prefix}",
                oninput: move |e: Event<FormData>| ipv4_prefix.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "IPv4 Prefix Length",
                value: "{ipv4_prefix_len}",
                oninput: move |e: Event<FormData>| ipv4_prefix_len.set(e.value().clone()),
            }
            label {
                class: "select-label",
                "Visibility"
                select {
                    value: "{visibility}",
                    oninput: move |e| visibility.set(e.value().clone()),
                    option { "Public" }
                    option { "Private" }
                }
            }
            button {
                onclick: add_pool,
                "Add IPv4 Pool"
            }
        }
    }
}

#[component]
fn Ipv4AssignmentAdd(space_id: i32, pool_id: i32) -> Element {
    let token = use_token();
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut ipv4_prefix = use_signal(|| String::new());
    let mut ipv4_prefix_len = use_signal(|| String::new());
    let mut visibility = use_signal(|| String::from("Public"));
    let mut error = use_signal(|| None);

    let add_assignment = move |_| {
        let space_id = space_id;
        let pool_id = pool_id;
        let token = token.clone();
        let name = name().trim().to_owned();
        let description = description().trim().to_owned();
        let ipv4_prefix = ipv4_prefix().trim().to_owned();
        let visibility = visibility().clone();

        if name.is_empty() || ipv4_prefix.is_empty() || visibility.is_empty() {
            error.set(Some("All fields are required".to_string()));
            return;
        }

        let ipv4_prefix_len = match ipv4_prefix_len().parse::<i32>() {
            Ok(val) => val,
            Err(_) => {
                error.set(Some("Invalid IPv4 prefix length value".to_string()));
                return;
            }
        };

        let prefix_obj = inet::Ipv4Prefix::new(&ipv4_prefix, ipv4_prefix_len);
        let (ipv4_prefix, _) = match prefix_obj {
            Ok(prefix_obj) => (prefix_obj.prefix_octets(), prefix_obj.prefix_len()),
            Err(s) => {
                error.set(Some(s.clone()));
                return;
            }
        };

        let visibility = match visibility.as_str() {
            "Public" | "Private" => visibility,
            _ => {
                error.set(Some("Invalid visibility value".to_string()));
                return;
            }
        };

        let new_assignment = inet::AssignmentIpv4 {
            id: 0, // This will be set by the server
            name,
            description,
            ipv4_prefix,
            ipv4_prefix_len,
            assignment_visibility: inet::ObjectVisibility::from_str(&visibility).unwrap(),
            assignment_pool_id: pool_id,
        };

        spawn(async move {
            let res: Result<inet::ApiResponse, _> = fetch::post(&format!("/api/v1/ipv4/assignment_space/{space_id}/pool/{pool_id}/assignment", space_id=space_id, pool_id=pool_id), &new_assignment, token.as_deref()).await;
            match res {
                Ok(inet::ApiResponse { error: None, result: _ }) => {
                    let nav = use_context::<Navigator>();
                    nav.push(Route::Ipv4Pool { space_id, pool_id });
                }
                Ok(inet::ApiResponse { error: Some(err), result: _ }) => {
                    error.set(Some(err));
                }
                Err(_) => {
                    error.set(Some("Failed to add IPv4 assignment".to_string()));
                }
            }
        });
    };

    rsx! {
        div {
            h1 { "Add IPv4 Assignment" }
            if let Some(err) = error() {
                p { style: "color: red;", "{err}" }
            }
            component::TextInput {
                placeholder: "Name",
                value: "{name}",
                oninput: move |e: Event<FormData>| name.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "Description",
                value: "{description}",
                oninput: move |e: Event<FormData>| description.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "IPv4 Prefix",
                value: "{ipv4_prefix}",
                oninput: move |e: Event<FormData>| ipv4_prefix.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "IPv4 Prefix Length",
                value: "{ipv4_prefix_len}",
                oninput: move |e: Event<FormData>| ipv4_prefix_len.set(e.value().clone()),
            }
            label {
                class: "select-label",
                "Visibility"
                select {
                    value: "{visibility}",
                    oninput: move |e| visibility.set(e.value().clone()),
                    option { "Public" }
                    option { "Private" }
                }
            }
            button {
                onclick: add_assignment,
                "Add IPv4 Assignment"
            }
        }
    }
}

#[component]
fn Ipv6SpaceAdd() -> Element {
    let token = use_token();
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut ipv6_prefix = use_signal(|| String::new());
    let mut ipv6_prefix_len = use_signal(|| String::new());
    let mut visibility = use_signal(|| String::from("Public"));
    let mut error = use_signal(|| None);

    let add_space = move |_| {
        let token = token.clone();
        let name = name().trim().to_owned();
        let description = description().trim().to_owned();
        let ipv6_prefix = ipv6_prefix().trim().to_owned();
        let ipv6_prefix_len = ipv6_prefix_len().trim().to_owned();

        if name.is_empty() || ipv6_prefix.is_empty() || ipv6_prefix_len.is_empty() {
            error.set(Some("All fields are required".to_string()));
            return;
        }

        let ipv6_prefix_len = match ipv6_prefix_len.parse::<i32>() {
            Ok(val) => val,
            Err(_) => {
                error.set(Some("Invalid IPv6 prefix length value".to_string()));
                return;
            }
        };

        let prefix_obj = inet::Ipv6Prefix::new(&ipv6_prefix, ipv6_prefix_len);
        let (ipv6_prefix, ipv6_prefix_len) = match prefix_obj {
            Ok(prefix_obj) => (prefix_obj.prefix_octets(), prefix_obj.prefix_len()),
            Err(s) => {
                error.set(Some(s.clone()));
                return;
            }
        };

        let visibility = visibility().clone();

        let visibility = match visibility.as_str() {
            "Public" | "Private" => visibility,
            _ => {
                error.set(Some("Invalid visibility value".to_string()));
                return;
            }
        };

        let new_space = inet::AssignmentSpaceIpv6 {
            id: 0, // This will be set by the server
            name,
            description,
            ipv6_prefix,
            ipv6_prefix_len,
            space_visibility: inet::ObjectVisibility::from_str(&visibility).unwrap(),
        };

        spawn(async move {
            let res: Result<inet::ApiResponse, _> = fetch::post("/api/v1/ipv6/assignment_space", &new_space, token.as_deref()).await;
            match res {
                Ok(inet::ApiResponse { error: None, result: _ }) => {
                    let nav = use_context::<Navigator>();
                    nav.push(Route::Ipv6SpaceList {});
                }
                Ok(inet::ApiResponse { error: Some(err), result: _ }) => {
                    error.set(Some(err));
                }
                Err(_) => {
                    error.set(Some("Failed to add IPv6 space".to_string()));
                }
            }
        });
    };

    rsx! {
        div {
            h1 { "Add IPv6 Assignment Space" }
            if let Some(err) = error() {
                p { style: "color: red;", "{err}" }
            }
            component::TextInput {
                placeholder: "Name",
                value: "{name}",
                oninput: move |e: Event<FormData>| name.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "Description",
                value: "{description}",
                oninput: move |e: Event<FormData>| description.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "IPv6 Prefix",
                value: "{ipv6_prefix}",
                oninput: move |e: Event<FormData>| ipv6_prefix.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "IPv6 Prefix Length",
                value: "{ipv6_prefix_len}",
                oninput: move |e: Event<FormData>| ipv6_prefix_len.set(e.value().clone()),
            }
            label {
                class: "select-label",
                "Visibility"
                select {
                    value: "{visibility}",
                    oninput: move |e| visibility.set(e.value().clone()),
                    option { "Public" }
                    option { "Private" }
                }
            }
            button {
                onclick: add_space,
                "Add IPv6 Space"
            }
        }
    }
}

#[component]
fn Ipv6PoolAdd(space_id: i32) -> Element {
    let token = use_token();
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut ipv6_prefix = use_signal(|| String::new());
    let mut ipv6_prefix_len = use_signal(|| String::new());
    let mut visibility = use_signal(|| String::from("Public"));
    let mut error = use_signal(|| None);

    let add_pool = move |_| {
        let space_id = space_id;
        let token = token.clone();
        let name = name().trim().to_owned();
        let description = description().trim().to_owned();
        let ipv6_prefix = ipv6_prefix().trim().to_owned();
        let ipv6_prefix_len = ipv6_prefix_len().trim().to_owned();
        let visibility = visibility().clone();

        if name.is_empty() || ipv6_prefix.is_empty() || ipv6_prefix_len.is_empty() || visibility.is_empty() {
            error.set(Some("All fields are required".to_string()));
            return;
        }

        let ipv6_prefix_len = match ipv6_prefix_len.parse::<i32>() {
            Ok(val) => val,
            Err(_) => {
                error.set(Some("Invalid IPv6 prefix length value".to_string()));
                return;
            }
        };

        let prefix_obj = inet::Ipv6Prefix::new(&ipv6_prefix, ipv6_prefix_len);
        let (ipv6_prefix, ipv6_prefix_len) = match prefix_obj {
            Ok(prefix_obj) => (prefix_obj.prefix_octets(), prefix_obj.prefix_len()),
            Err(s) => {
                error.set(Some(s.clone()));
                return;
            }
        };

        let visibility = match visibility.as_str() {
            "Public" | "Private" => visibility,
            _ => {
                error.set(Some("Invalid visibility value".to_string()));
                return;
            }
        };

        let new_pool = inet::AssignmentPoolIpv6 {
            id: 0, // This will be set by the server
            name,
            description,
            ipv6_prefix,
            ipv6_prefix_len,
            pool_visibility: inet::ObjectVisibility::from_str(&visibility).unwrap(),
            assignment_space_id: space_id,
        };

        spawn(async move {
            let res: Result<inet::ApiResponse, _> = fetch::post(&format!("/api/v1/ipv6/assignment_space/{space_id}/pool", space_id=space_id), &new_pool, token.as_deref()).await;
            match res {
                Ok(inet::ApiResponse { error: None, result: _ }) => {
                    let nav = use_context::<Navigator>();
                    nav.push(Route::Ipv6Space { space_id });
                }
                Ok(inet::ApiResponse { error: Some(err), result: _ }) => {
                    error.set(Some(err));
                }
                Err(_) => {
                    error.set(Some("Failed to add IPv6 pool".to_string()));
                }
            }
        });
    };

    rsx! {
        div {
            h1 { "Add IPv6 Assignment Pool" }
            if let Some(err) = error() {
                p { style: "color: red;", "{err}" }
            }
            component::TextInput {
                placeholder: "Name",
                value: "{name}",
                oninput: move |e: Event<FormData>| name.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "Description",
                value: "{description}",
                oninput: move |e: Event<FormData>| description.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "IPv6 Prefix",
                value: "{ipv6_prefix}",
                oninput: move |e: Event<FormData>| ipv6_prefix.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "IPv6 Prefix Length",
                value: "{ipv6_prefix_len}",
                oninput: move |e: Event<FormData>| ipv6_prefix_len.set(e.value().clone()),
            }
            label {
                class: "select-label",
                "Visibility"
                select {
                    value: "{visibility}",
                    oninput: move |e| visibility.set(e.value().clone()),
                    option { "Public" }
                    option { "Private" }
                }
            }
            button {
                onclick: add_pool,
                "Add IPv6 Pool"
            }
        }
    }
}

#[component]
fn Ipv6AssignmentAdd(space_id: i32, pool_id: i32) -> Element {
    let token = use_token();
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut ipv6_prefix = use_signal(|| String::new());
    let mut ipv6_prefix_len = use_signal(|| String::new());
    let mut visibility = use_signal(|| String::from("Public"));
    let mut error = use_signal(|| None);

    let add_assignment = move |_| {
        let space_id = space_id;
        let pool_id = pool_id;
        let token = token.clone();
        let name = name().trim().to_owned();
        let description = description().trim().to_owned();
        let ipv6_prefix = ipv6_prefix().trim().to_owned();
        let visibility = visibility().clone();

        if name.is_empty() || ipv6_prefix.is_empty() || visibility.is_empty() {
            error.set(Some("All fields are required".to_string()));
            return;
        }

        let ipv6_prefix_len = match ipv6_prefix_len().parse::<i32>() {
            Ok(val) => val,
            Err(_) => {
                error.set(Some("Invalid IPv6 prefix length value".to_string()));
                return;
            }
        };

        let prefix_obj = inet::Ipv6Prefix::new(&ipv6_prefix, ipv6_prefix_len);
        let (ipv6_prefix, _) = match prefix_obj {
            Ok(prefix_obj) => (prefix_obj.prefix_octets(), prefix_obj.prefix_len()),
            Err(s) => {
                error.set(Some(s.clone()));
                return;
            }
        };

        let visibility = match visibility.as_str() {
            "Public" | "Private" => visibility,
            _ => {
                error.set(Some("Invalid visibility value".to_string()));
                return;
            }
        };

        let new_assignment = inet::AssignmentIpv6 {
            id: 0, // This will be set by the server
            name,
            description,
            ipv6_prefix,
            ipv6_prefix_len,
            assignment_visibility: inet::ObjectVisibility::from_str(&visibility).unwrap(),
            assignment_pool_id: pool_id,
        };

        spawn(async move {
            let res: Result<inet::ApiResponse, _> = fetch::post(&format!("/api/v1/ipv6/assignment_space/{space_id}/pool/{pool_id}/assignment", space_id=space_id, pool_id=pool_id), &new_assignment, token.as_deref()).await;
            match res {
                Ok(inet::ApiResponse { error: None, result: _ }) => {
                    let nav = use_context::<Navigator>();
                    nav.push(Route::Ipv6Pool { space_id, pool_id });
                }
                Ok(inet::ApiResponse { error: Some(err), result: _ }) => {
                    error.set(Some(err));
                }
                Err(_) => {
                    error.set(Some("Failed to add IPv6 assignment".to_string()));
                }
            }
        });
    };

    rsx! {
        div {
            h1 { "Add IPv6 Assignment" }
            if let Some(err) = error() {
                p { style: "color: red;", "{err}" }
            }
            component::TextInput {
                placeholder: "Name",
                value: "{name}",
                oninput: move |e: Event<FormData>| name.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "Description",
                value: "{description}",
                oninput: move |e: Event<FormData>| description.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "IPv6 Prefix",
                value: "{ipv6_prefix}",
                oninput: move |e: Event<FormData>| ipv6_prefix.set(e.value().clone()),
            }
            component::TextInput {
                placeholder: "IPv6 Prefix Length",
                value: "{ipv6_prefix_len}",
                oninput: move |e: Event<FormData>| ipv6_prefix_len.set(e.value().clone()),
            }
            label {
                class: "select-label",
                "Visibility"
                select {
                    value: "{visibility}",
                    oninput: move |e| visibility.set(e.value().clone()),
                    option { "Public" }
                    option { "Private" }
                }
            }
            button {
                onclick: add_assignment,
                "Add IPv6 Assignment"
            }
        }
    }
}
