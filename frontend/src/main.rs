#![allow(non_snake_case)]
pub mod component;
pub mod fetch;
pub mod inet;
mod session;

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use inet::{ApiResponse, Assignment};
use serde::{Deserialize, Serialize};

pub const GITHUB_LINK: &str = "https://github.com/menhera-org/mirams";

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(Wrapper)]
    #[route("/")]
    Home {},
    #[route("/login/")]
    Login {},
    #[route("/asn/")]
    AsnAssignments {},
    #[route("/ipv4/")]
    Ipv4Assignments {},
    #[route("/ipv6/")]
    Ipv6Assignments {},
    #[route("/assignments/:resource_type/add/")]
    RootAssignmentAdd { resource_type: String },
    #[route("/assignment/:id/")]
    AssignmentPage { id: i32 },
    #[route("/assignment/:parent_id/add/")]
    ChildAssignmentAdd { parent_id: i32 },
    #[route("/:..segments")]
    PageNotFound { segments: Vec<String> },
}

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");
    launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(session::load_user()));
    use_context_provider(|| Signal::new(component::DrawerState { open: false }));

    rsx! {
        Router::<Route> {}
    }
}

pub fn close_drawer() {
    let mut drawer = consume_context::<Signal<component::DrawerState>>();
    drawer.set(component::DrawerState { open: false });
}

#[component]
fn Wrapper() -> Element {
    let nav = navigator();
    let nav_context = nav.clone();
    use_context_provider(move || nav_context);

    let drawer_open = use_context::<Signal<component::DrawerState>>();
    let drawer_open = drawer_open().open;
    let mut user = use_context::<Signal<Option<component::account::User>>>();
    let signed_in = user().is_some();

    let mut class = if drawer_open {
        "app-drawer-open"
    } else {
        "app-drawer-closed"
    }
    .to_string();
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
                            let mut drawer = consume_context::<Signal<component::DrawerState>>();
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
                    let mut drawer = consume_context::<Signal<component::DrawerState>>();
                    drawer.set(component::DrawerState { open: false });
                },
            }
            component::Drawer {
                onlogout: move |_| {
                    session::save_user(None);
                    user.set(None);
                    nav.push(Route::Home {});
                },
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
    let nav = navigator();

    let log_in = move |attempt: component::account::LoginAttempt| {
        let nav = nav.clone();
        spawn(async move {
            let username = attempt.username.clone();

            let res: Result<LoginResult, _> = fetch::post("/api/v1/login", &attempt, None).await;
            let res = match res {
                Ok(res) => res,
                Err(_e) => {
                    error.set(Some(format!("Login failed")));
                    return;
                }
            };

            let signed_in_user = component::account::User {
                username,
                api_token: res.api_token,
            };
            session::save_user(Some(&signed_in_user));
            user.set(Some(signed_in_user));

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

fn root_route(resource_type: &str) -> Route {
    match resource_type {
        "asn" => Route::AsnAssignments {},
        "ipv4" => Route::Ipv4Assignments {},
        _ => Route::Ipv6Assignments {},
    }
}

fn family_label(resource_type: &str) -> &'static str {
    match resource_type {
        "asn" => "ASN",
        "ipv4" => "IPv4",
        "ipv6" => "IPv6",
        _ => "Unknown",
    }
}

fn token(user: Signal<Option<component::account::User>>) -> Option<String> {
    user().map(|user| user.api_token)
}

fn response(result: Result<ApiResponse, reqwest::Error>) -> Result<ApiResponse, String> {
    let response = result.map_err(|error| format!("Request failed: {error}"))?;
    if let Some(error) = &response.error {
        return Err(error.clone());
    }
    Ok(response)
}

async fn load_assignment(id: i32, token: Option<&str>) -> Result<Assignment, String> {
    response(fetch::get::<ApiResponse>(&format!("/api/v1/assignment/{id}"), token).await)?
        .assignment
        .ok_or_else(|| "Assignment not found".to_string())
}

async fn load_assignments(url: &str, token: Option<&str>) -> Result<Vec<Assignment>, String> {
    response(fetch::get::<ApiResponse>(url, token).await)?
        .assignments
        .ok_or_else(|| "Assignments could not be loaded".to_string())
}

fn table_rows(assignments: &[Assignment]) -> Vec<component::table::TableRow> {
    assignments
        .iter()
        .map(|assignment| component::table::TableRow {
            assignment: assignment.range.label(),
            name: assignment.name.clone(),
            description: assignment.description.clone(),
            visibility: assignment.visibility.to_string(),
            url: format!("/assignment/{}/", assignment.id),
        })
        .collect()
}

#[component]
fn Home() -> Element {
    rsx! {
        h1 { "MIRAMS" }
        p { "Manage Internet resource assignments. Open an assignment to view or add nested assignments." }
        ul {
            li { Link { to: Route::AsnAssignments {}, "ASN Assignments" } }
            li { Link { to: Route::Ipv4Assignments {}, "IPv4 Assignments" } }
            li { Link { to: Route::Ipv6Assignments {}, "IPv6 Assignments" } }
        }
        p { Link { to: GITHUB_LINK, "Source code on GitHub" } }
    }
}

#[component]
fn AsnAssignments() -> Element {
    let resource_type = "asn";
    rsx! { AssignmentList { resource_type } }
}
#[component]
fn Ipv4Assignments() -> Element {
    let resource_type = "ipv4";
    rsx! { AssignmentList { resource_type } }
}
#[component]
fn Ipv6Assignments() -> Element {
    let resource_type = "ipv6";
    rsx! { AssignmentList { resource_type } }
}

#[component]
fn AssignmentList(resource_type: String) -> Element {
    let user = use_context::<Signal<Option<component::account::User>>>();
    let family = resource_type.clone();
    let assignments = use_resource(move || {
        let token = token(user);
        let family = family.clone();
        async move {
            load_assignments(
                &format!("/api/v1/assignment?resource_type={family}"),
                token.as_deref(),
            )
            .await
        }
    });
    let title = format!("{} Assignments", family_label(&resource_type));
    rsx! {
        h1 { "{title}" }
        p { "Top-level assignments may overlap. Open an assignment to manage its children." }
        component::AddButtonToolbar { add_button_text: "Add assignment", add_button_route: Route::RootAssignmentAdd { resource_type } }
        match &*assignments.read_unchecked() {
            Some(Ok(assignments)) => rsx! {
                component::table::AssignmentTable { rows: table_rows(assignments) }
                if assignments.is_empty() { p { "No assignments yet." } }
            },
            Some(Err(error)) => rsx! { p { role: "alert", "{error}" } },
            None => rsx! { p { "Loading assignments…" } },
        }
    }
}

#[component]
fn AssignmentPage(id: i32) -> Element {
    // A keyed fragment gives every assignment its own editor and resources.
    rsx! {
        for node_id in [id] {
            AssignmentDetails { key: "{node_id}", id: node_id }
        }
    }
}

#[component]
fn AssignmentDetails(id: i32) -> Element {
    let user = use_context::<Signal<Option<component::account::User>>>();
    let nav = navigator();
    let mut error = use_signal(|| None::<String>);
    let mut deleting = use_signal(|| false);
    let mut data = use_resource(move || {
        let token = token(user);
        async move {
            let assignment = load_assignment(id, token.as_deref()).await?;
            let ancestors = load_assignments(
                &format!("/api/v1/assignment/{id}/ancestors"),
                token.as_deref(),
            )
            .await?;
            let children = load_assignments(
                &format!("/api/v1/assignment?parent_id={id}"),
                token.as_deref(),
            )
            .await?;
            Ok::<_, String>((assignment, ancestors, children))
        }
    });
    match &*data.read_unchecked() {
        None => rsx! { p { "Loading assignment…" } },
        Some(Err(error)) => rsx! { p { role: "alert", "{error}" } },
        Some(Ok((assignment, ancestors, children))) => {
            let family = assignment.range.resource_type();
            let mut crumbs = vec![component::BreadCrumb {
                name: format!("{} Assignments", family_label(family)),
                route: root_route(family),
            }];
            crumbs.extend(ancestors.iter().map(|ancestor| component::BreadCrumb {
                name: format!("{} — {}", ancestor.range.label(), ancestor.name),
                route: Route::AssignmentPage { id: ancestor.id },
            }));
            let label = assignment.range.label();
            let after_delete = assignment
                .parent_id
                .map(|id| Route::AssignmentPage { id })
                .unwrap_or_else(|| root_route(family));
            rsx! {
                component::BreadCrumbs { crumbs, title: label.clone() }
                h1 { "Assignment: {label}" }
                component::MetadataForm {
                    name: assignment.name.clone(), description: assignment.description.clone(), visibility: assignment.visibility, error: error(),
                    onsubmit: move |metadata| {
                        let token = token(user);
                        error.set(None);
                        spawn(async move {
                            match response(fetch::put::<ApiResponse, _>(&format!("/api/v1/assignment/{id}"), metadata, token.as_deref()).await) {
                                Ok(_) => data.restart(),
                                Err(message) => error.set(Some(message)),
                            }
                        });
                    }
                }
                h2 { "Nested assignments" }
                component::AddButtonToolbar { add_button_text: "Add nested assignment", add_button_route: Route::ChildAssignmentAdd { parent_id: id } }
                component::table::AssignmentTable { rows: table_rows(children) }
                if children.is_empty() { p { "No nested assignments." } }
                if user().is_some() {
                    MoveAssignmentForm {
                        assignment: assignment.clone(),
                        onmoved: move |_| data.restart(),
                    }
                    div {
                        class: "delete-toolbar",
                        if deleting() {
                            div {
                                class: "delete-popup",
                                p { "Delete this assignment and all its nested assignments?" }
                                button {
                                    class: "delete-button",
                                    onclick: move |_| {
                                        let token = token(user);
                                        let nav = nav.clone();
                                        let destination = after_delete.clone();
                                        spawn(async move {
                                            match response(fetch::delete::<ApiResponse>(&format!("/api/v1/assignment/{id}"), token.as_deref()).await) {
                                                Ok(_) => { nav.replace(destination); },
                                                Err(message) => error.set(Some(message)),
                                            }
                                        });
                                    },
                                    "Delete assignment and descendants"
                                }
                                button { class: "cancel-button", onclick: move |_| deleting.set(false), "Cancel" }
                            }
                        } else {
                            button { class: "delete-button", onclick: move |_| deleting.set(true), "Delete assignment" }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Serialize)]
struct MoveRequest {
    parent_id: Option<i32>,
}

#[component]
fn MoveAssignmentForm(assignment: Assignment, onmoved: EventHandler<()>) -> Element {
    let id = assignment.id;
    let user = use_context::<Signal<Option<component::account::User>>>();
    let mut destination = use_signal(|| {
        assignment
            .parent_id
            .map(|id| id.to_string())
            .unwrap_or_default()
    });
    let mut error = use_signal(|| None::<String>);
    let mut saving = use_signal(|| false);
    let mut destinations = use_resource(move || {
        let token = token(user);
        async move {
            load_assignments(
                &format!("/api/v1/assignment/{id}/move-targets"),
                token.as_deref(),
            )
            .await
        }
    });
    rsx! {
        section {
            class: "move-assignment-form",
            h2 { "Move assignment" }
            p { "Nested assignments move with this assignment. Only destinations that can contain its range are shown." }
            match &*destinations.read_unchecked() {
                Some(Ok(targets)) => rsx! {
                    label {
                        class: "select-label",
                        "Destination",
                        select {
                            value: destination(), disabled: saving(), oninput: move |event| destination.set(event.value()),
                            option { value: "", "Top level" }
                            for target in targets {
                                option { value: "{target.id}", "{target.range.label()} — {target.name}" }
                            }
                        }
                    }
                    button {
                        disabled: saving(),
                        onclick: move |_| {
                            let parent_id = if destination().is_empty() { None } else { destination().parse::<i32>().ok() };
                            let token = token(user);
                            error.set(None);
                            saving.set(true);
                            spawn(async move {
                                match response(fetch::post::<ApiResponse, _>(&format!("/api/v1/assignment/{id}/move"), MoveRequest { parent_id }, token.as_deref()).await) {
                                    Ok(_) => { destinations.restart(); onmoved.call(()); },
                                    Err(message) => error.set(Some(message)),
                                }
                                saving.set(false);
                            });
                        },
                        if saving() { "Moving…" } else { "Move assignment" }
                    }
                },
                Some(Err(error)) => rsx! { p { role: "alert", "{error}" } },
                None => rsx! { p { "Loading destinations…" } },
            }
            if let Some(error) = error() { p { role: "alert", "{error}" } }
        }
    }
}

#[component]
fn RootAssignmentAdd(resource_type: String) -> Element {
    rsx! {
        for family in [resource_type] {
            AssignmentForm { key: "root-{family}", resource_type: family, parent: None }
        }
    }
}

#[component]
fn ChildAssignmentAdd(parent_id: i32) -> Element {
    rsx! {
        for id in [parent_id] {
            ChildAssignmentAddInner { key: "{id}", parent_id: id }
        }
    }
}

#[component]
fn ChildAssignmentAddInner(parent_id: i32) -> Element {
    let user = use_context::<Signal<Option<component::account::User>>>();
    let parent = use_resource(move || {
        let token = token(user);
        async move { load_assignment(parent_id, token.as_deref()).await }
    });
    match &*parent.read_unchecked() {
        Some(Ok(parent)) => {
            rsx! { AssignmentForm { resource_type: parent.range.resource_type(), parent: Some(parent.clone()) } }
        }
        Some(Err(error)) => rsx! { p { role: "alert", "{error}" } },
        None => rsx! { p { "Loading parent assignment…" } },
    }
}

#[component]
fn AssignmentForm(resource_type: String, parent: Option<Assignment>) -> Element {
    let user = use_context::<Signal<Option<component::account::User>>>();
    let nav = navigator();
    let parent_id = parent.as_ref().map(|parent| parent.id);
    let defaults = parent
        .as_ref()
        .map(|parent| parent.range.fields())
        .unwrap_or_default();
    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut first = use_signal(|| defaults.0);
    let mut second = use_signal(|| defaults.1);
    let mut visibility = use_signal(|| inet::ObjectVisibility::Public);
    let mut error = use_signal(|| None::<String>);
    let mut saving = use_signal(|| false);
    let first_label = if resource_type == "asn" {
        "Start ASN"
    } else {
        "Network address"
    };
    let second_label = if resource_type == "asn" {
        "End ASN"
    } else {
        "Prefix length"
    };
    let family = family_label(&resource_type);
    if !["asn", "ipv4", "ipv6"].contains(&resource_type.as_str()) {
        return rsx! { p { role: "alert", "Unknown resource type." } };
    }
    if user().is_none() {
        return rsx! { p { "Sign in to create assignments." } Link { to: Route::Login {}, "Sign in" } };
    }
    rsx! {
        h1 { "Add {family} assignment" }
        if let Some(parent) = parent {
            p { "Within " Link { to: Route::AssignmentPage { id: parent.id }, "{parent.range.label()} — {parent.name}" } }
        } else { p { "This is a top-level assignment. Top-level ranges may overlap." } }
        if let Some(error) = error() { p { role: "alert", "{error}" } }
        component::TextInput { placeholder: "Name", value: name(), oninput: move |event: Event<FormData>| name.set(event.value()) }
        component::TextInput { placeholder: "Description", value: description(), oninput: move |event: Event<FormData>| description.set(event.value()) }
        component::TextInput { placeholder: first_label, value: first(), oninput: move |event: Event<FormData>| first.set(event.value()) }
        component::TextInput { placeholder: second_label, value: second(), oninput: move |event: Event<FormData>| second.set(event.value()) }
        label {
            class: "select-label",
            "Visibility",
            select {
                value: visibility().to_string(), oninput: move |event| visibility.set(if event.value() == "Private" { inet::ObjectVisibility::Private } else { inet::ObjectVisibility::Public }),
                option { value: "Public", "Public" }
                option { value: "Private", "Private" }
            }
        }
        button {
            disabled: saving(),
            onclick: move |_| {
                error.set(None);
                let range = match inet::AssignmentRange::from_fields(&resource_type, &first(), &second()) {
                    Ok(range) => range,
                    Err(message) => { error.set(Some(message)); return; }
                };
                if name().trim().is_empty() { error.set(Some("Enter a name for this assignment".into())); return; }
                let assignment = Assignment { id: 0, parent_id, name: name().trim().to_owned(), description: description().trim().to_owned(), visibility: visibility(), range };
                let token = token(user);
                let nav = nav.clone();
                saving.set(true);
                spawn(async move {
                    match response(fetch::post::<ApiResponse, _>("/api/v1/assignment", assignment, token.as_deref()).await) {
                        Ok(response) => match response.assignment {
                            Some(assignment) => { nav.replace(Route::AssignmentPage { id: assignment.id }); },
                            None => error.set(Some("The server did not return the new assignment".into())),
                        },
                        Err(message) => error.set(Some(message)),
                    }
                    saving.set(false);
                });
            },
            if saving() { "Creating…" } else { "Create assignment" }
        }
    }
}

#[component]
fn PageNotFound(segments: Vec<String>) -> Element {
    let path = format!("/{}", segments.join("/"));
    rsx! {
        h1 { "Page not found" }
        p { "No page exists at {path}." }
        Link { to: Route::Home {}, "Browse assignments" }
    }
}
