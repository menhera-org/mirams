#![allow(non_snake_case)]

pub mod account;
pub mod table;

use crate::Route;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Props)]
pub struct DrawerProps {
    pub onlogout: EventHandler<account::User>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawerState {
    /// Whether the drawer is open in mobile view. (in desktop view, the drawer is always open)
    pub open: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionResponse {
    pub version: String,
}

#[component]
pub fn Drawer(props: DrawerProps) -> Element {
    let user = use_context::<Signal<Option<account::User>>>();
    let drawer_open = use_context::<Signal<DrawerState>>();
    let nav = use_context::<Navigator>();

    let class = if drawer_open().open { "drawer drawer-open" } else { "drawer" };

    let version_future = use_resource(|| {
        async {
            let res: Result<VersionResponse, _> = crate::fetch::get("/api/v1/version", None).await;
            res
        }
    });

    let version = match &*version_future.read_unchecked() {
        Some(Ok(res)) => res.version.clone(),
        _ => "unknown".to_string(),
    };

    rsx! {
        div {
            id: "app-drawer",
            class: class,
            div {
                id: "app-drawer-shortcuts",
            }
            div {
                id: "app-drawer-navigation",
                account::AccountBar {
                    user: user(),
                    onlogin: move |_| { nav.replace(Route::Login {}); },
                    onlogout: props.onlogout.clone(),
                }
                Link {
                    class: "link-button",
                    to: Route::Home {},
                    onclick: move |_| { crate::close_drawer(); },
                    "Home",
                }
                Link {
                    class: "link-button",
                    to: Route::AsnSpaceList {},
                    onclick: move |_| { crate::close_drawer(); },
                    "ASN Assignments",
                }
                Link {
                    class: "link-button",
                    to: Route::Ipv4SpaceList {},
                    onclick: move |_| { crate::close_drawer(); },
                    "IPv4 Assignments",
                }
                Link {
                    class: "link-button",
                    to: Route::Ipv6SpaceList {},
                    onclick: move |_| { crate::close_drawer(); },
                    "IPv6 Assignments",
                }
                div {
                    class: "drawer-footer",
                    p { "MIRAMS version: {version}" }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BreadCrumb {
    pub name: String,
    pub route: Route,
}

#[derive(Clone, Debug, PartialEq, Props)]
pub struct BreadCrumbsProps {
    /// parent pages
    pub crumbs: Vec<BreadCrumb>,

    /// Short title for the current page
    pub title: String,
}

#[component]
pub fn BreadCrumbs(props: BreadCrumbsProps) -> Element {
    rsx! {
        div {
            class: "breadcrumbs",
            for crumb in props.crumbs {
                Link {
                    class: "crumb",
                    to: crumb.route.clone(),
                    "{crumb.name}",
                }
                " / "
            }
            span {
                class: "breadcrumbs-current",
                "{props.title}",
            }
        }
    }
}
