# Rust TUI Chat Application

This is a simple terminal-based chat application built in Rust using a client-server architecture. The application supports multiple concurrent users communicating over TCP through a custom TUI (text-based user interface).

## 🧠 Features

- Asynchronous networking using [`tokio`](https://crates.io/crates/tokio)
- Terminal UI using [`tui`](https://crates.io/crates/tui) and [`crossterm`](https://crates.io/crates/crossterm)
- Real-time messaging over TCP sockets
- Multi-user support with broadcast channels
- Simple message history displayed upon client connection
- containerized docker images through Dockerfiles for server and client

## 🚀 Running the Project

### Server

The server listens for client connections and handles message distribution.

The client provides a simple TUI for the clients to communicate with each other and the server.


Added logging features for the code that logs usernames/passwords and message logging on the server using SQLite. 

Messages are serialized using JSON format for consistent format to different clients.

## DOCKER ##

The project uses docker for testing we can run the docker containers in local network for testing to do this we must compile the docker images than run them.

We will first have to create a custom docker network, a basic bridge network with a specific subnet, note that right now the clients are set to connect 
to the server with IP 172.30.0.10:42069 change this if you will use a different subnet.

# create the docker network 
    docker network create \
    --driver bridge \
    --subnet 172.30.0.0/16 \
    --gateway 172.30.0.1 \
    chat-server-net

Choose IPs and subnets for how you would like to test the client-server code.
    
# server (already running with a fixed IP)
    docker run -d --name chat-server \
      --network chat-server-net --ip 172.30.0.10 \
      charles9507/rust-chat-server:server

# client (interactive)
    docker run --rm -it --name chat-client \
      --network chat-server-net \
      charles9507/rust-chat-server:client

Note that for the client we NEED the --rm -it flags or else this will close the client prematurely because the session is not interactive.

