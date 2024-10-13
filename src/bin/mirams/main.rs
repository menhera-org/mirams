
mod cli;

use cli::Cli;
use cli::Commands;

use mirams::Store;
use mirams::db_sqlite::SqliteConnection;
use mirams::server::Server;

use clap::Parser;
use syslog::{Facility, Formatter3164, BasicLogger};
use rand::prelude::*;

use std::path::PathBuf;


#[derive(Debug, Clone)]
pub(crate) struct GlobalConfig {
    pub db_path: Option<PathBuf>,
    pub command: Commands,
}

impl GlobalConfig {
    pub fn open_sqlite_connection(&self) -> SqliteConnection {
        if let Some(path) = &self.db_path {
            SqliteConnection::open_file(path.to_str().unwrap())
        } else {
            SqliteConnection::open_memory()
        }.unwrap()
    }

    pub fn store(&self) -> Store<SqliteConnection> {
        Store::new(self.open_sqlite_connection())
    }

    pub fn run(&self) {
        match &self.command {
            Commands::Server { listen_addr: _ } => server(self.clone()),
            Commands::UserSetPassword { username: _, password: _ } => user_set_password(self.clone()),
            Commands::UserDelete { username: _ } => user_delete(self.clone()),

            #[allow(unreachable_patterns)]
            _ => unimplemented!(),
        }
    }

    pub fn check_for_actual_db(&self) {
        if self.db_path.is_none() {
            log::warn!("No database path specified. Using in-memory database for command-line operations makes no sense.");
            std::process::exit(0);
        }
    }

    pub fn is_in_memory_db(&self) -> bool {
        self.db_path.is_none()
    }
}

fn main() {
    let args = Cli::parse();
    let log_level = if args.verbose {
        log::Level::Debug
    } else {
        log::Level::Info
    };

    let formatter = Formatter3164 {
        facility: Facility::LOG_DAEMON,
        hostname: None,
        process: "mirams".into(),
        pid: 0,
    };

    let mut loggers: Vec<Box<dyn log::Log>> = Vec::new();
    loggers.push(Box::new(env_logger::Builder::new().filter_level(log::LevelFilter::Debug).build()));

    let logger = syslog::unix(formatter).ok();
    if let Some(logger) = logger {
        loggers.push(Box::new(BasicLogger::new(logger)));
    }

    multi_log::MultiLogger::init(loggers, log_level).unwrap();

    let mut db_path = std::env::var("MIRAMS_DB_PATH").ok().map(PathBuf::from);
    if let Some(path) = &args.db_path {
        db_path = Some(path.clone());
    }

    if let Some(path) = &db_path {
        log::info!("Using database at {:?}", path);
    } else {
        log::info!("Using in-memory database. All data will be lost when the program exits.");
    }

    let global_config = GlobalConfig {
        db_path,
        command: args.command,
    };

    global_config.run();
}

fn server(global_config: GlobalConfig) {
    let mut listen_addr = std::env::var("MIRAMS_LISTEN_ADDR").unwrap_or("127.0.0.1:3001".to_string());

    if let Some(addr) = match &global_config.command {
        Commands::Server { listen_addr } => listen_addr.clone(),
        _ => None,
    } {
        listen_addr = addr;
    }

    let store = global_config.store();

    if global_config.is_in_memory_db() {
        let mut rng = rand::thread_rng();
        let password: [u8; 8] = rng.gen();
        let password = hex::encode(password);
        let user = "admin";
        store.users().set_password(user, &password).unwrap();
        log::info!("Created user 'admin' with password '{}'", password);
    }

    let server = Server::new(store);

    log::info!("Starting server on {}", listen_addr);
    server.serve(listen_addr);

    loop {
        std::thread::park();
    }
}

fn user_set_password(global_config: GlobalConfig) {
    global_config.check_for_actual_db();

    match &global_config.command {
        Commands::UserSetPassword { username, password } => {
            let store = global_config.store();
            store.users().set_password(username, password).unwrap();
        },
        _ => unreachable!(),
    }
}

fn user_delete(global_config: GlobalConfig) {
    global_config.check_for_actual_db();

    match &global_config.command {
        Commands::UserDelete { username } => {
            let store = global_config.store();
            store.users().delete_user(username).unwrap();
        },
        _ => unreachable!(),
    }
}
