use futures::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Sender},
};

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

#[tokio::main]
async fn main() {
    let server = TcpListener::bind("127.0.0.1:42069")
        .await
        .expect("Could not bind the listening socket");

    let (tx, _) = broadcast::channel::<String>(32);

    #[warn(unused_variables)]
    /*TODO make a string that holds the entire chat history of the server
     */
    let history: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    loop {
        let (tcp, client_addr) = server
            .accept()
            .await
            .expect("Could not establish connection to the client");

        println!("New client {:?} connected", client_addr);

        let history_clone = Arc::clone(&history);
        tokio::spawn(handle_user(tcp, tx.clone(), history_clone));
    }
}

async fn handle_user(
    mut tcp: TcpStream,
    tx: Sender<String>,
    history: Arc<Mutex<Vec<String>>>,
) -> anyhow::Result<()> {
    let (reader, writer) = tcp.split();
    let mut stream = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());
    let mut rx = tx.subscribe();

    {
        let history_clone = history.lock().await;
        if !history_clone.is_empty() {
            for msg in history_clone.iter() {
                let _ = sink.send(msg).await;
            }
        }
    }

    loop {
        tokio::select! {

            //Receiving a message from the client
            user_msg = stream.next() => {
                let  user_msg = match user_msg {
                    Some(msg) => {
                        match msg{
                            Ok(msg) => {
                                {
                                    let mut history_clone = history.lock().await;
                                    history_clone.push(msg.clone());
                                }

                                msg
                            },
                            Err(_) => {
                                break},
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
