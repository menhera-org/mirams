#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoginAttempt {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Props)]
pub struct LoginFormProps {
    pub error: Option<String>,
    pub username: Option<String>,
    pub onlogin: EventHandler<LoginAttempt>,
}

pub fn LoginForm(props: LoginFormProps) -> Element {
    let mut username = use_signal(move || props.username.unwrap_or_default().to_owned());
    let mut password = use_signal(|| "".to_owned());

    rsx! {
        form {
            class: "login-form",
            h1 { "Login" }
            label {
                class: "login-form-username",
                "Username",
                input {
                    r#type: "text",
                    placeholder: "Username",
                    value: "{username}",
                    oninput: move |event| username.set(event.value()),
                }
            }
            label {
                class: "login-form-password",
                "Password",
                input {
                    r#type: "password",
                    placeholder: "Password",
                    value: "{password}",
                    oninput: move |event| password.set(event.value()),
                }
            }
            div {
                class: "login-form-buttons",
                button {
                    class: "login-form-login-button",
                    r#type: "button",
                    onclick: move |_| {
                        let username = username();
                        let password = password();
                        tracing::info!("Login attempt: {username}");
                        props.onlogin.call(LoginAttempt { username, password });
                    },
                    "Login",
                }
            }
            match &props.error {
                Some(error) => {
                    rsx! {
                        div {
                            class: "login-form-error",
                            "{error}"
                        }
                    }
                }
                None => {
                    rsx! {}
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub username: String,
    pub api_token: String,
}

#[derive(Clone, Debug, PartialEq, Props)]
pub struct AccountBarProps {
    pub user: Option<User>,
    pub onlogout: EventHandler<User>,
    pub onlogin: EventHandler<()>,
}

pub fn AccountBar(props: AccountBarProps) -> Element {
    rsx! {
        div {
            match props.user {
                Some(user) => {
                    rsx! {
                        div {
                            class: "account-bar",
                            div {
                                class: "account-bar-username logged-in",
                                span {
                                    class: "material-symbols-outlined",
                                    "account_circle",
                                }
                                "{&user.username}"
                            }
                            button {
                                class: "link-button",
                                onclick: move |_| {
                                    crate::close_drawer();
                                    props.onlogout.call(user.clone());
                                },
                                "Logout",
                            }
                        }
                    }
                }
                None => {
                    rsx! {
                        div {
                            class: "account-bar",
                            div {
                                class: "account-bar-username logged-out",
                                span {
                                    class: "material-symbols-outlined",
                                    "account_circle",
                                }
                                "Not logged in"
                            }
                            button {
                                class: "link-button",
                                onclick: move |_| {
                                    crate::close_drawer();
                                    props.onlogin.call(());
                                },
                                "Login",
                            }
                        }
                    }
                }
            }
        }
    }
}
