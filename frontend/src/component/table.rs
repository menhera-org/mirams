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
pub fn AssignmentTable(mut rows: Vec<TableRow>) -> Element {
    rsx! {
        table {
            thead {
                tr {
                    th { "Assignment" }
                    th { "Name" }
                    th { "Description" }
                    th { "Visibility" }
                }
            }
            tbody {
                for row in rows {
                    tr {
                        td {
                            Link {
                                to: NavigationTarget::<crate::Route>::from_str(&row.url).unwrap(),
                                "{row.assignment}"
                            }
                        }
                        td { "{row.name}" }
                        td { "{row.description}" }
                        td { "{row.visibility}" }
                    }
                }
            }
        }
    }
}
