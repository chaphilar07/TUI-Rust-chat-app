# Rust TUI Chat Application

This is a simple terminal-based chat application built in Rust using a client-server architecture. The application supports multiple concurrent users communicating over TCP through a custom TUI (text-based user interface).

## 🧠 Features

- Asynchronous networking using [`tokio`](https://crates.io/crates/tokio)
- Terminal UI using [`tui`](https://crates.io/crates/tui) and [`crossterm`](https://crates.io/crates/crossterm)
- Real-time messaging over TCP sockets
- Multi-user support with broadcast channels
- Simple message history displayed upon client connection

## 🚀 Running the Project

### Server

The server listens for client connections and handles message distribution.

```bash
cargo run -p server



The cient will connect to the server and open broadcast channels

cargo run -p client






Added logging features for the code that logs usernames/passwords and message logging on the server using SQLite. 

Next we will add serialization of the messages that get sent using json and serde crate.

we extended the code to add the following Features
  - we using usernames/passwords for logging purposes
  - we added message logging in the server using a lightweight DB setup 
