
use std::path::PathBuf;
use clap::{Parser, Subcommand};

/// MIRAMS: Menhera.org Internet Resources Assignment Management System
#[derive(Debug, Parser, Clone)] // requires `derive` feature
#[command(name = "mirams")]
#[command(version, about, long_about = None)]
pub(crate) struct Cli {
    /// Increase verbosity
    #[arg(short, long)]
    pub verbose: bool,

    /// Path to the SQLite database file
    #[arg(short, long)]
    pub db_path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[non_exhaustive]
#[derive(Debug, Subcommand, Clone)]
pub(crate) enum Commands {
    /// Start the MIRAMS server
    #[command(name = "server")]
    Server {
        /// Address to listen on
        #[arg(short, long)]
        listen_addr: Option<String>,
    },

    /// Set a user's password. If the user does not exist, it will be created.
    #[command(name = "user-set-password")]
    UserSetPassword {
        /// Username
        #[arg(short, long)]
        username: String,

        /// New password
        #[arg(short, long)]
        password: String,
    },

    /// Delete a user
    #[command(name = "user-delete")]
    UserDelete {
        /// Username
        #[arg(short, long)]
        username: String,
    },
}
