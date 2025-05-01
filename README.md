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
