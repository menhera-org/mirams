#![allow(non_snake_case)]

use dioxus::prelude::*;

pub mod account;

use crate::Route;

#[derive(Clone, Debug, PartialEq, Props)]
pub struct DrawerProps {
    pub onlogout: EventHandler<account::User>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawerState {
    /// Whether the drawer is open in mobile view. (in desktop view, the drawer is always open)
    pub open: bool,
}

#[component]
pub fn Drawer(props: DrawerProps) -> Element {
    let user = use_context::<Signal<Option<account::User>>>();
    let drawer_open = use_context::<Signal<DrawerState>>();
    let nav = use_context::<Navigator>();

    let class = if drawer_open().open { "drawer drawer-open" } else { "drawer" };

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
            }
        }
    }
}
