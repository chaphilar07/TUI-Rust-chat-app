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


we extended the code to add the following Features
  - we using usernames/passwords for logging purposes
  - we added message logging in the server using a lightweight DB setup 


Update: Created docker images to run the server and client code on separate machines testing to see if both work the client has already been shown to work

  to run and create the docker images assuming that you have docker

  docker build -t server ./server
  docker build -t client ./client

  docker images (check that the images are there)
  
  docker run server
  docker run client

Note that the images that get created are going to create ARM64 containers, for computers that are running x86_64 architecture you should recompile the project and change the top line of the dockerfiles for the client and the server.
  
