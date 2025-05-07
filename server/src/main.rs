use futures::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use sqlx::SqlitePool;
use std::process::exit;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Sender},
};

use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
static DB: Lazy<SqlitePool> = Lazy::new(|| {
    // WAL mode avoids many "database is locked" errors in multi‑client chat
    SqlitePool::connect_lazy("sqlite:Users.db?mode=rwc").expect("pool")
});
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    username: String,
    timestamp: String,
    msg: String,
}
//START OF MAIN
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:42069").await;

    let listener = match listener {
        Ok(real_listener) => {
            println!("Connection has been established, server listnening on socket 42069");
            real_listener
        }
        Err(err) => {
            println!(
                "Error {} cannot bind the listener to the socket exiting",
                err
            );
            exit(1);
        }
    };

    let (tx, _) = broadcast::channel::<String>(32);
    let history: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    loop {
        let (mut tcp, addr) = listener
            .accept()
            .await
            .expect("Could not accept the incoming connection");
        println!("New connection accepted from address {}", addr);

        let ipaddr = addr.to_string();
        let (reader, writer) = tcp.split();

        let mut stream = FramedRead::new(reader, LinesCodec::new());
        let mut sink = FramedWrite::new(writer, LinesCodec::new());

        let username = match stream.next().await {
            Some(Ok(line)) => {
                println!("Username {} has attempted to log on to the server", line);
                line
            }
            _ => {
                println!("connection closed before the username could be received");
                continue;
            }
        };

        let known_user: bool =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM Users WHERE username = ?")
                .bind(&username)
                .fetch_one(&*DB)
                .await
                .expect("Could not unwrap the query for all usernames")
                != 0;

        match known_user {
            true => {
                let _ = sink.send("100").await;
            }
            false => {
                let _ = sink.send("101").await;
            }
        }

        let mut password = match stream.next().await {
            Some(Ok(line)) => {
                println!("Successfully received the password from the user");
                line
            }
            _ => {
                println!("Error connection closed before client sent password.");
                continue;
            }
        };

        let mut passed = if known_user {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM Users 
                              WHERE username =? AND password = ?",
            )
            .bind(&username)
            .bind(&password)
            .fetch_one(&*DB)
            .await
            .expect("Could not unwrap the DB query")
                != 0
        } else {
            let _ = sqlx::query!(
                "INSERT INTO Users (username,password,ipaddr) VALUES (?1,?2,?3)",
                username,
                password,
                ipaddr
            )
            .execute(&*DB)
            .await
            .expect("Could not insert the new tuple into the table");
            true
        };

        while !passed {
            println!("{}", passed);
            let _ = sink.send("102").await;

            //Now we would have to wait for the client to resend a valid password and we will have
            //to check if it is the valid password for the user.

            password = match stream.next().await {
                Some(Ok(line)) => {
                    println!("Successfully received the password from the user now comparing ...");
                    line
                }
                Some(Err(err)) => {
                    println!("Error in the sending {}", err);
                    continue;
                }
                None => {
                    println!("Connection with the client has been interrupted");
                    continue;
                }
            };

            passed = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM Users 
                              WHERE username =? AND password = ?",
            )
            .bind(&username)
            .bind(&password)
            .fetch_one(&*DB)
            .await
            .expect("Could not unwrap the DB query")
                != 0;
        }

        let _ = sink.send("100").await;

        let history_clone = Arc::clone(&history);
        let _ = tokio::spawn(handle_user(tcp, tx.clone(), history_clone));
    }
}
//END OF MAIN
async fn handle_user(
    mut tcp: TcpStream,
    tx: Sender<String>,
    history: Arc<Mutex<Vec<String>>>,
) -> anyhow::Result<()> {
    let (reader, writer) = tcp.split();
    let mut stream = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());
    let mut rx = tx.subscribe();

    //TODO Need to redo how the history gets sent to the client, we want to basically load the so
    //many most recent messages from the server, then we send those to the client, we should load
    //these from the database and then serialize them into json objects
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
                                    let parts: Vec<&str> = msg.split_ascii_whitespace().collect();

                                    let datetime:String = format!("{}:{}", parts[0], parts[1]);
                                    let user_length:usize=parts[2].len();
                                    let mut username:String = String::from(parts[2]); //We remove the last character
                                     username.truncate(user_length-1);
                                    let msg:String = parts[3..].join(" ");

                                    let _ =sqlx::query!("INSERT INTO Messages (username,timestamp,msg) VALUES(?1,?2,?3);", username, datetime, msg).execute(&*DB).await.expect("Could not unwrap the insertion of the message");
                                    let json_msg = Message{  username: username, timestamp: datetime, msg:  msg.clone()};


                                    let wire = serde_json::to_string(&json_msg).unwrap();
                                    println!("Serialized JSON String: {}", wire);

                                    //Now we need to send to the clients the

                                    let mut history_clone = history.lock().await;
                                    history_clone.push(wire.clone());
                                wire
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
