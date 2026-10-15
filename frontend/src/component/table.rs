//! ASN/IPv4/IPv6 assignment tables

#![allow(non_snake_case)]

use dioxus::prelude::*;

use std::str::FromStr;


/// Assignment table row for ASN/IPv4/IPv6 assignments
#[derive(Debug, Clone, PartialEq)]
pub struct TableRow {
    /// Stringified assignment range
    pub assignment: String,

    /// Assignment name
    pub name: String,

    /// Assignment description
    pub description: String,

    /// Assignment visibility
    pub visibility: String,

    /// (Origin-relative) URL to assignment details
    pub url: String,
}


#[component]
pub fn AssignmentTable(rows: Vec<TableRow>) -> Element {
    rsx! {
        div {
            class: "assignment-table",
            role: "table",
            "aria-label": "Assignments",
            div {
                class: "assignment-head",
                role: "rowgroup",
                div {
                    class: "assignment-row assignment-header",
                    role: "row",
                    div { class: "assignment-cell", role: "columnheader", "Assignment" }
                    div { class: "assignment-cell", role: "columnheader", "Name" }
                    div { class: "assignment-cell", role: "columnheader", "Description" }
                }
            }
            div {
                class: "assignment-body",
                role: "rowgroup",
                for row in rows {
                    div {
                        class: if row.visibility == "Public" { "assignment-row public" } else { "assignment-row private" },
                        role: "row",
                        title: row.visibility,
                        div {
                            class: "assignment-cell",
                            role: "cell",
                            Link {
                                to: NavigationTarget::<crate::Route>::from_str(&row.url).unwrap(),
                                "{row.assignment}"
                            }
                        }
                        div { class: "assignment-cell", role: "cell", "{row.name}" }
                        div { class: "assignment-cell", role: "cell", "{row.description}" }
                    }
                }
            }
        }
    }
}
