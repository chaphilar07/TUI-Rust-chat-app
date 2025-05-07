/*
 * We will be creating three threads one for sending, one for receiving and one for displaying the
 * tui.
 *
 * We will use two seperate broadcast channels, one that will be for receiving messages from the
 * server then sending them to the tui thread, another for the sending of data from the user from
 * the tui thread to the sending thread.
 */

//Next we want to be able to scroll the tui that we have created.
use chrono::Utc;
use crossterm::event::{KeyEvent, KeyModifiers};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{SinkExt, StreamExt};
use inquire::{Password, PasswordDisplayMode};
use serde::{Deserialize, Serialize};
use std::io::{self, stdin, stdout, Write};
use std::net::{IpAddr, SocketAddr as StdSocketAddr};
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use textwrap::wrap;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpSocket;
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio::time::Duration;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
static IS_RUNNING: AtomicBool = AtomicBool::new(true);

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    username: String,
    timestamp: String,
    msg: String,
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let client_socket =
        TcpSocket::new_v4().expect("Could not get a socket for the client connection");

    let server_ip = "127.0.0.1"
        .parse::<IpAddr>()
        .expect("Could not parse the server IP address");

    let server_socket = StdSocketAddr::new(server_ip, 42069);

    //We will create two seperate channels for sending and receiving from the server.
    let (tx0, rx0) = channel::<String>(32);
    let (tx1, mut rx1) = channel::<String>(32);

    let connection = client_socket
        .connect(server_socket)
        .await
        .expect("Could not establish a connection with the client");
    println!("Connected to the server!");

    let (reader, writer) = connection.into_split();

    let mut stream = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());

    let mut username = String::new();

    let mut valid_user = false;
    let mut valid_pass = false;

    while !valid_user {
        print!("Enter a username for the server: ");
        let _ = stdout().flush();
        stdin()
            .read_line(&mut username)
            .expect("Could not read the username");

        username = username.trim().to_string(); // Remove any trailing newlines

        let _ = sink.send(username.clone()).await;

        valid_user = match stream.next().await {
            Some(result) => match result {
                Ok(line) => match line.trim() {
                    "100" | "101" => {
                        println!("Valid user name code receieved from the server");
                        true
                    }
                    _ => false,
                },

                Err(err) => {
                    println!("Connection to the server interrupted {} exiting", err);
                    exit(1);
                }
            },
            None => {
                println!("Connection to the server was interrupted");
                exit(1);
            }
        };
    }

    while !valid_pass {
        let password = Password::new("Enter your password")
            .with_display_mode(PasswordDisplayMode::Masked)
            .prompt()
            .expect("Could not unwrap the password");

        match sink.send(password.clone()).await {
            Ok(num) => {
                println!("Password sent successfully ");
                num
            }
            Err(err) => {
                println!("Error {} Could not send the pas word to the server", err);
            }
        }

        valid_pass = match stream.next().await {
            Some(result) => match result {
                Ok(line) => match line.trim() {
                    "100" | "101" => true,
                    "102" => {
                        println!(
                            "Invalid password used for the known user {}, please try again",
                            username
                        );
                        false
                    }
                    _ => false,
                },
                Err(err) => {
                    println!("Error {} Connection with the server interrupted", err);
                    exit(1);
                }
            },

            None => {
                println!("Connection with the server intererupted");
                exit(1);
            }
        };

        println!("{}", valid_pass);
    }
    let reading_thread = tokio::spawn(read_from_server(stream, tx1));
    let sending_thread = tokio::spawn(send_to_server(sink, rx0));

    //If we get to this point we are now able to enter the server!

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut input = String::new();
    let mut display_lines: Vec<String> = Vec::new();

    let mut scroll_offset: u16 = 0;
    let mut cursor_position = 0;
    let mut input_scroll: u16 = 0;

    //Beginning of the event loop
    loop {
        let size = terminal.size().expect("Cannot unwrap the terminal!!!");
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
            .split(size);

        //        let input_height = chunks[1].height.saturating_sub(2);
        let input_width = chunks[1].width.saturating_sub(2);

        let pane_width = chunks[0].width.saturating_sub(2);
        let pane_height = chunks[0].height.saturating_sub(2);

        if event::poll(Duration::from_millis(10))? {
            // If there is a key event, process it
            if let Event::Key(KeyEvent { code, modifiers }) = event::read()? {
                match (code, modifiers) {
                    (KeyCode::Char(c), KeyModifiers::NONE) => {
                        input.insert(cursor_position, c);
                        cursor_position += 1;
                    }
                    (KeyCode::Backspace, KeyModifiers::NONE) => {
                        if cursor_position > 0 {
                            cursor_position -= 1;
                            input.remove(cursor_position);
                        }
                    }

                    (KeyCode::Up, KeyModifiers::NONE) => {
                        scroll_offset = scroll_offset.saturating_add(1)
                    }
                    (KeyCode::Down, KeyModifiers::NONE) => {
                        scroll_offset = scroll_offset.saturating_sub(1)
                    }

                    (KeyCode::Left, KeyModifiers::NONE) => {
                        if cursor_position > 0 {
                            cursor_position -= 1;
                        }
                    }
                    (KeyCode::Right, KeyModifiers::NONE) => {
                        if cursor_position < input.len() {
                            cursor_position += 1;
                        }
                    }
                    (KeyCode::Enter, KeyModifiers::NONE) => {
                        let time = Utc::now();
                        let time = time.format("%Y-%m-%d %H:%M:%S").to_string();
                        let msg = format!("{}  {}: {}", time, username, input);

                        let _ = tx0.send(msg);
                        input.clear();
                        cursor_position = 0;
                    }

                    (KeyCode::Up, KeyModifiers::ALT) => {
                        input_scroll = input_scroll.saturating_add(1)
                    }
                    (KeyCode::Down, KeyModifiers::ALT) => {
                        input_scroll = input_scroll.saturating_sub(1)
                    }

                    (KeyCode::Esc, KeyModifiers::NONE) => {
                        disable_raw_mode()?;
                        execute!(
                            terminal.backend_mut(),
                            LeaveAlternateScreen,
                            DisableMouseCapture
                        )?;
                        IS_RUNNING.store(false, Ordering::Relaxed);
                        terminal.show_cursor()?;

                        break;
                    }
                    _ => {}
                }
            }
        }

        //We need to change this logic so that the client that
        match rx1.try_recv() {
            Ok(peer_msg) => {
                //Now we should be receiving json messages, we need to deseiralize them.
                let desierialized_msg: Result<Message, _> = serde_json::from_str(&peer_msg);

                match desierialized_msg {
                    Ok(message) => {
                        let formatted_msg = format!(
                            "{}:{} - {}",
                            message.timestamp, message.username, message.msg
                        );
                        for wrapped in wrap(&formatted_msg, pane_width as usize) {
                            display_lines.push(wrapped.into_owned());
                        }
                    }
                    Err(err) => {
                        println!("Could not deserialize the object error {}", err);
                    }
                }
                scroll_offset = 0; //Set to the most recent message
            }
            Err(_) => {
                //Here do nothing no message is in the buffer, note that we do not await this!
            }
        }

        // Redraw the terminal with the updated state
        terminal.draw(|f| {
            let input_with_cursor = if cursor_position < input.len() {
                let (left, right) = input.split_at(cursor_position);
                format!("{}|{}", left, right)
            } else {
                format!("{}|", input)
            };

            let mut display_input: Vec<String> = Vec::new();
            for wrapped in wrap(&input_with_cursor, input_width as usize) {
                display_input.push(wrapped.into_owned());
            }
            let display_string = display_input.join("\n");

            let total_rows = display_lines.len() as u16;
            let start_row = total_rows.saturating_sub(pane_height + scroll_offset);
            let end_row = start_row + pane_height.min(total_rows);
            let visible = display_lines[start_row as usize..end_row as usize].join("\n");

            let display_paragraph = Paragraph::new(visible)
                .block(Block::default().borders(Borders::ALL).title("Display Area"));

            let input_paragraph = Paragraph::new(display_string)
                .block(Block::default().borders(Borders::ALL).title("Input Area"))
                .style(Style::default().fg(Color::Yellow));

            f.render_widget(input_paragraph, chunks[1]);
            f.render_widget(display_paragraph, chunks[0]);
        })?;
    } //End of the event loop

    let _ = reading_thread.await;
    let _ = sending_thread.await;

    Ok(())
}

async fn read_from_server(
    mut stream: FramedRead<OwnedReadHalf, LinesCodec>,
    tx: Sender<String>,
) -> Result<(), io::Error> {
    loop {
        if IS_RUNNING.load(Ordering::Relaxed) == false {
            break;
        }

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
        if IS_RUNNING.load(Ordering::Relaxed) == false {
            break;
        }
        match rx.try_recv() {
            Ok(msg) => {
                let _ = sink.send(msg).await;
            }
            _ => {}
        }
    }

    Ok(())
}
