#![allow(non_snake_case)]

pub mod component;
pub mod fetch;

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

#[component]
fn AsnSpaceList() -> Element {
    rsx! {
        "ASN Space List"
    }
}

#[component]
fn AsnSpace(space_id: i32) -> Element {
    rsx! {
        "ASN Space {space_id}"
    }
}

#[component]
fn AsnPool(space_id: i32, pool_id: i32) -> Element {
    rsx! {
        "ASN Pool {pool_id} in space {space_id}"
    }
}

#[component]
fn AsnAssignment(space_id: i32, pool_id: i32, assignment_id: i32) -> Element {
    rsx! {
        "ASN Assignment {assignment_id} in pool {pool_id} in space {space_id}"
    }
}

#[component]
fn Ipv4SpaceList() -> Element {
    rsx! {
        "IPv4 Space List"
    }
}

#[component]
fn Ipv4Space(space_id: i32) -> Element {
    rsx! {
        "IPv4 Space {space_id}"
    }
}

#[component]
fn Ipv4Pool(space_id: i32, pool_id: i32) -> Element {
    rsx! {
        "IPv4 Pool {pool_id} in space {space_id}"
    }
}

#[component]
fn Ipv4Assignment(space_id: i32, pool_id: i32, assignment_id: i32) -> Element {
    rsx! {
        "IPv4 Assignment {assignment_id} in pool {pool_id} in space {space_id}"
    }
}

#[component]
fn Ipv6SpaceList() -> Element {
    rsx! {
        "IPv6 Space List"
    }
}

#[component]
fn Ipv6Space(space_id: i32) -> Element {
    rsx! {
        "IPv6 Space {space_id}"
    }
}

#[component]
fn Ipv6Pool(space_id: i32, pool_id: i32) -> Element {
    rsx! {
        "IPv6 Pool {pool_id} in space {space_id}"
    }
}

#[component]
fn Ipv6Assignment(space_id: i32, pool_id: i32, assignment_id: i32) -> Element {
    rsx! {
        "IPv6 Assignment {assignment_id} in pool {pool_id} in space {space_id}"
    }
}
