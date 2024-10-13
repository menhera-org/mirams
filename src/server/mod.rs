
pub mod api;

use crate::static_files::frontend_files;
use crate::static_files::types_by_ext;
use crate::store::Store;
use crate::store::DbConnection;

use axum::Router;
use axum::http::HeaderMap;
use axum::body::Body;
use axum::routing::get;
use axum::extract::State as StateExtractor;

use http::{Request, Response};

use std::net::ToSocketAddrs;
use std::path::Path;


#[derive(Debug, Clone)]
pub struct Server<T>
where 
    T: DbConnection + Clone + Send + Sync + 'static,
{
    store: Store<T>,
}

impl<T> Server<T>
where 
    T: DbConnection + Clone + Send + Sync + 'static,
{
    pub fn new(store: Store<T>) -> Self {
        Server { store }
    }

    pub fn store(&self) -> &Store<T> {
        &self.store
    }

    /// Start the server on the given address, in a new thread.
    pub fn serve(&self, addr: impl ToSocketAddrs) {
        let server = self.clone();
        let addr = addr.to_socket_addrs().unwrap().next().unwrap();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
            rt.block_on(async move {
                let mut app = build_frontend_router();

                let api_router = api::build_api_v1_router();
                app = app.nest("/api/v1", api_router);

                app = app.layer(axum::middleware::from_fn_with_state(server.clone(), add_state_extension::<Server<T>>));
                let app = app.with_state(server);

                let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
                axum::serve(listener, app).await.unwrap();
            });
        });
    }
}


pub fn build_frontend_router<S>() -> Router<S>
where 
    S: Clone + Send + Sync + 'static,
{
    let files = frontend_files();
    let types = types_by_ext();

    let mut router = Router::new();
    for (path, content) in files {
        if path == "index.html" {
            continue;
        }
        if path.ends_with(".br") {
            continue;
        }

        let content = *content;
        let br_content = files.get(&(path.to_string() + ".br")).map(|content| *content).map(|content| content.to_owned());
        let content_type = *types.get(&Path::new(path).extension().and_then(|ext| ext.to_str()).unwrap_or("")).unwrap_or(&"application/octet-stream");
        let content_type = content_type.to_owned();

        router = router.route(&format!("/{}", path), get(move |headers: HeaderMap| async move {
            if headers.get("Accept-Encoding").and_then(|header| header.to_str().ok()).map(|header| header.contains("br")).unwrap_or(false) {
                if let Some(br_content) = br_content {
                    return Response::builder()
                        .status(200)
                        .header("Content-Type", &content_type)
                        .header("Content-Encoding", "br")
                        .body(Body::from(br_content))
                        .unwrap()
                }
            }
            Response::builder()
                .status(200)
                .header("Content-Type", &content_type)
                .body(Body::from(content))
                .unwrap()
        }));
    }

    let index_content = *files.get("index.html").expect("index.html not found");
    let index_content = index_content.to_owned();

    let index_content_root = index_content.clone();
    router = router.route("/", get(|| async {
        Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(Body::from(index_content_root))
            .unwrap()
    }));

    router = router.fallback(|| async {
        Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(Body::from(index_content))
            .unwrap()
    });

    router
}

async fn add_state_extension<S: Clone + Send + Sync + 'static>(StateExtractor(s): StateExtractor<S>, req: Request<Body>, next: axum::middleware::Next) -> Response<Body> {
    let mut req = req;
    req.extensions_mut().insert(s);
    next.run(req).await
}
