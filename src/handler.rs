use crate::{Client, Clients, Result};
use std::collections::HashMap;
use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::sync::{RwLockWriteGuard};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{http::StatusCode, ws::Message, Reply};
use futures::{FutureExt, StreamExt};

#[derive(Deserialize, Debug)]
pub struct WebSocketCmd {
    method: String,
    params: Vec<String>,
    id: u64
}

pub async fn client_conn(socket: warp::ws::WebSocket, id: String, clients: Clients, mut c: Client) {

    let (wtx, mut wrx) = socket.split();
    let (tx, rx) = mpsc::unbounded_channel();
    let stream = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(stream.forward(wtx).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending msg: {}", e)
        }
    }));

    c.sender = Some(tx);
    clients.write().await.insert(id.clone(), c);

    println!("{} connected", id.clone());

    while let Some(result) = wrx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error received ws msg {}: {}", id.clone(), e);
                break;
            }
        };
    }
    clients.write().await.remove(&id);
    println!("{} disconnected", id);
}

pub async fn ws_handler(ws: warp::ws::Ws, addr: Option<SocketAddr>, clients: Clients) -> Result<impl Reply> {
    let id = addr.unwrap().to_string();
    let c = clients.read().await.get(&id).cloned().unwrap_or_else(|| {
        Client{
            id: 0,
            topics: vec![],
            sender: None,
        }
    });
    Ok(ws.on_upgrade(move |socket| client_conn(socket, id, clients, c)))
}

/*pub async fn publish_handler(body: String, clients: Clients) -> Result<impl Reply> {
    clients.read().await.iter().for_each(|(_, client)| {
        if let Some(sender) = &client.sender {
            sender.send(Ok(Message::text("hello")));
        }
    });
    Ok(StatusCode::OK)
}

pub async fn register_handler(body: WebSocketCmd, clients: Clients) -> Result<impl Reply> {
    let mut map = clients.write().await;
    map.insert(
        body.id,
        Client{
            id: body.id,
            topics: vec![],
            sender: None,
        },
    );
    Ok(StatusCode::OK)
}*/
