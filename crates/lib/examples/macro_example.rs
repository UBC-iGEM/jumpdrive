use std::{io::Write, net::TcpStream};

use jumpdrive::JumpdriveResult;
use proc::jumpdrive;
use tungstenite::WebSocket;

fn main() -> JumpdriveResult {
    jumpdrive! {
        dir = "../examples_public/",
        ws = ["/ws", websocket_handler],
        routes = {
            "/csv": csv_server
        },
        err = error_handler
    }
}

fn websocket_handler(ws: &mut WebSocket<TcpStream>) {
    loop {
        if let Ok(msg) = ws.read() {
            println!("WS message read: {msg}");
            if msg.is_text() {
                ws.send(msg).unwrap();
            }
        }
    }
}
fn csv_server(stream: &mut TcpStream) {
    let msg = "Hi from CSV!";
    stream
        .write_all(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
                msg.len(),
                msg
            )
            .as_bytes(),
        )
        .unwrap();
}
fn error_handler(e: String) {
    println!("MAJOR OOPSIES: {e}");
}
