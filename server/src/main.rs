use futures::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Sender},
};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

#[tokio::main]
async fn main() {
    let server = TcpListener::bind("127.0.0.1:42071")
        .await
        .expect("Could not bind the listening socket");

    let (tx, _) = broadcast::channel::<String>(32);

    #[warn(unused_variables)]
    let history = Arc::new(Mutex::new(String::new()));

    loop {
        let (tcp, _) = server
            .accept()
            .await
            .expect("Could not establish connection to the client");

        tokio::spawn(handle_user(tcp, tx.clone()));
    }
}

async fn handle_user(mut tcp: TcpStream, tx: Sender<String>) -> anyhow::Result<()> {
    let (reader, writer) = tcp.split();
    let mut stream = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());
    let mut rx = tx.subscribe();

    loop {
        tokio::select! {
            user_msg = stream.next() => {
                let  user_msg = match user_msg {
                    Some(msg) => {
                        match msg{
                            Ok(msg) => msg,
                            Err(_) => break,
                        }
                    },
                    None => break,
                };
                let _ = tx.send(user_msg);
            },
            peer_msg = rx.recv() => {
                let peer_msg = match peer_msg {
                    Ok(msg) => msg,
                    Err(_)=> break,

                };
                let _  = sink.send(peer_msg).await;
            }
        }
    }

    Ok(())
}
