use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::{ws::Message, Filter, Rejection};

mod handler;

type Result<T> = std::result::Result<T, Rejection>;
type Clients = Arc<RwLock<HashMap<String, Client>>>;

#[derive(Debug, Clone)]
pub struct Client {
    pub id: u64,
    pub topics: Vec<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::addr::remote())
        .and(with_clients(clients.clone()))
        .and_then(handler::ws_handler);

    let routes = ws_route.with(warp::cors().allow_any_origin());
    warp::serve(routes).run(([0,0,0,0], 7900)).await;
}
