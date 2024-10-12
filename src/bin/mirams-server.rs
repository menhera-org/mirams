
use mirams::Store;
use mirams::db_sqlite::SqliteConnection;
use mirams::server::Server;

fn main() -> ! {
    let db_path = std::env::var("MIRAMS_DB_PATH").ok();
    let listen_addr = std::env::var("MIRAMS_LISTEN_ADDR").unwrap_or("127.0.0.1:3001".to_string());

    let db = if let Some(path) = db_path {
        SqliteConnection::open_file(&path)
    } else {
        SqliteConnection::open_memory()
    }.unwrap();

    let store = Store::new(db);
    let server = Server::new(store);

    println!("Starting server on {}", listen_addr);
    server.serve(listen_addr);

    loop {
        std::thread::park();
    }
}
