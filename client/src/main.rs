/*
 * We will be creating three threads one for sending, one for receiving and one for displaying the
 * tui.
 *
 * We will use two seperate broadcast channels, one that will be for receiving messages from the
 * server then sending them to the tui thread, another for the sending of data from the user from
 * the tui thread to the sending thread.
 *
 *
 */
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use futures::{SinkExt, StreamExt};
use std::net::{IpAddr, SocketAddr as StdSocketAddr};
use std::{
    fmt::format,
    io::{self, stdin, stdout, Write},
};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpSocket;
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio::time::{self, Duration};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let client_socket =
        TcpSocket::new_v4().expect("Could not get a socket for the client connection");

    let server_ip = "127.0.0.1"
        .parse::<IpAddr>()
        .expect("Could not parse the server IP address");

    let server_socket = StdSocketAddr::new(server_ip, 42069);

    //We will create two seperate channels for sending and receiving from the server.
    let (mut tx0, mut rx0) = channel::<String>(32);
    let (mut tx1, mut rx1) = channel::<String>(32);

    let connection = client_socket
        .connect(server_socket)
        .await
        .expect("Could not establish a connection with the client");
    let (reader, writer) = connection.into_split();

    let stream = FramedRead::new(reader, LinesCodec::new());
    let sink = FramedWrite::new(writer, LinesCodec::new());

    let mut username = String::new();

    print!("Enter a username for the server: ");
    let _ = stdout().flush();
    stdin()
        .read_line(&mut username)
        .expect("Could not read the username");

    username = username.trim().to_string(); // Remove any trailing newlines

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let reading_thread = tokio::spawn(read_from_server(stream, tx1));
    let sending_thread = tokio::spawn(send_to_server(sink, rx0));

    let mut input = String::new();
    let mut display = String::new();
    let mut cursor_position = 0;

    loop {
        // Poll for incoming messages from the server
        match rx1.try_recv() {
            Ok(peer_msg) => {
                display.push_str(peer_msg.as_str());
                display.push('\n');
            }
            Err(_) => {
                // No new messages, just continue
            }
        }

        // Poll for user input events
        if event::poll(Duration::from_millis(10))? {
            // If there is a key event, process it
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => {
                        input.insert(cursor_position, c);
                        cursor_position += 1;
                    }
                    KeyCode::Backspace => {
                        if cursor_position > 0 {
                            cursor_position -= 1;
                            input.remove(cursor_position);
                        }
                    }
                    KeyCode::Left => {
                        if cursor_position > 0 {
                            cursor_position -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if cursor_position < input.len() {
                            cursor_position += 1;
                        }
                    }
                    KeyCode::Enter => {
                        let mut msg = format!("{}: {}", username, input);
                        if msg.len() > 40 {
                            msg.push('\n');
                        }
                        let _ = tx0.send(msg);
                        input.clear();
                        cursor_position = 0;
                    }
                    KeyCode::Esc => {
                        break; // Exit the loop on Esc
                    }
                    _ => {}
                }
            }
        }

        // Redraw the terminal with the updated state
        terminal.draw(|f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                .split(size);

            let input_with_cursor = if cursor_position < input.len() {
                let (left, right) = input.split_at(cursor_position);
                format!("{}|{}", left, right)
            } else {
                format!("{}|", input)
            };

            // Input area
            let input_paragraph = Paragraph::new(input_with_cursor)
                .block(Block::default().borders(Borders::ALL).title("Input Area"))
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(input_paragraph, chunks[1]);
            let display_paragraph = Paragraph::new(display.as_ref())
                .block(Block::default().borders(Borders::ALL).title("Display Area"));
            f.render_widget(display_paragraph, chunks[0]);
        })?;
    }
    let _ = reading_thread.await;
    let _ = sending_thread.await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

//This function reads from the server, then it will send that message through a Sender to
//the tui thread so that it can be displayed.
async fn read_from_server(
    mut stream: FramedRead<OwnedReadHalf, LinesCodec>,
    tx: Sender<String>,
) -> Result<(), io::Error> {
    loop {
        let _ = match stream.next().await {
            Some(msg) => match msg {
                Ok(user_msg) => tx.send(user_msg),
                Err(e) => {
                    println!("Error {:?}", e);
                    break;
                }
            },
            None => break,
        };
    }

    Ok(())
}

//In this function we will attempt to read from the server then we will send this to the server
//using the sink that is connected to the server.
async fn send_to_server(
    mut sink: FramedWrite<OwnedWriteHalf, LinesCodec>,
    mut rx: Receiver<String>,
) -> Result<(), io::Error> {
    loop {
        match rx.recv().await {
            Ok(peer_msg) => {
                let _ = sink.send(peer_msg).await;
            }
            Err(e) => {
                println!("ERROR, cannot read from the channel {:?}", e);
                break;
            }
        }
    }

    Ok(())
}

async fn capture_user_input() -> io::Result<String> {
    let mut input = String::new();
    let mut cursor_position = 0;

    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => {
                    input.insert(cursor_position, c);
                    cursor_position += 1;
                }
                KeyCode::Backspace => {
                    if cursor_position > 0 {
                        cursor_position -= 1;
                        input.remove(cursor_position);
                    }
                }
                KeyCode::Left => {
                    if cursor_position > 0 {
                        cursor_position -= 1;
                    }
                }
                KeyCode::Right => {
                    if cursor_position < input.len() {
                        cursor_position += 1;
                    }
                }
                KeyCode::Enter => {
                    return Ok(input); // Return the input string when Enter is pressed
                }
                KeyCode::Esc => {
                    return Err(io::Error::new(io::ErrorKind::Interrupted, "Esc pressed"));
                }
                _ => {}
            }
        }
    }
}
