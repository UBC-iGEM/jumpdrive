use phf::Map;
use std::{
    env,
    error::Error,
    io::{self, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};
use tungstenite::{WebSocket, accept};

pub use proc::jumpdrive;

/// The internal representation of a Jumpdrive process.\
/// **Note**: this type should never be constructed directly.
/// Instead, use the `jumpdrive!` macro.
#[derive(Debug)]
pub struct Jumpdrive {
    map: Map<&'static str, (&'static [u8], &'static str)>,
    socket: Option<Socket>,
    other_paths: Map<&'static str, fn(&mut TcpStream)>,
    error_handler: fn(String),
}
type Socket = (&'static str, fn(&mut WebSocket<TcpStream>));

/// A specialized Result type for the `jumpdrive!` macro.
/// # Returns
/// - Err if binding to a TcpListener fails on the requested IP and PORT
/// - Should never realistically return otherwise
pub type JumpdriveResult = io::Result<()>;

impl Jumpdrive {
    pub fn new(
        map: Map<&'static str, (&'static [u8], &'static str)>,
        socket: Option<Socket>,
        other_paths: Map<&'static str, fn(&mut TcpStream)>,
        error_handler: fn(String),
    ) -> Self {
        Self {
            map,
            socket,
            other_paths,
            error_handler,
        }
    }

    pub fn serve(self) -> JumpdriveResult {
        let config = Arc::new(self);
        let ip = env::var("IP").unwrap_or("127.0.0.1".to_string());
        let port = env::var("PORT").unwrap_or("9999".to_string());
        let addr = format!("{ip}:{port}");
        let listener = TcpListener::bind(addr)?;

        for connection in listener.incoming() {
            if let Ok(conn) = connection
                && let Ok(mut conn_clone) = conn.try_clone()
            {
                let config = Arc::clone(&config);
                thread::spawn(move || {
                    let output = || -> Result<(), Box<dyn Error>> {
                        let mut buffer = [0; 1024];
                        let len = conn_clone.peek(&mut buffer)?;
                        if len == 0 {
                            return Err("Request is empty!".into());
                        }
                        let request = String::from_utf8_lossy(&buffer[0..len]);
                        if let Some(line) = request.lines().next() {
                            let components: Vec<_> = line.split_whitespace().collect();
                            if let [method, mut path, protocol] = components[0..3]
                                && method == "GET"
                                && protocol == "HTTP/1.1"
                            {
                                {
                                    if path == "/" {
                                        path = "/index.html";
                                    }
                                    if let Some((socket_path, socket_handler)) = config.socket
                                        && path == socket_path
                                    {
                                        let mut ws = accept(conn_clone)?;
                                        socket_handler(&mut ws);
                                        return Ok(());
                                    }
                                    if let Some((asset, content_type)) = config.map.get(&path[1..])
                                    {
                                        let response = generate_response(content_type, asset);
                                        conn_clone.write_all(&response)?;
                                        return Ok(());
                                    }

                                    if let Some(callback) = config.other_paths.get(path) {
                                        callback(&mut conn_clone);
                                    }
                                }
                            } else {
                                return Err("Malformed HTTP request header!".into());
                            }
                        }
                        Ok(())
                    };
                    if let Err(e) = output() {
                        (config.error_handler)(e.to_string());
                    }
                });
            } else {
                (config.error_handler)("Failed to connect to a client!".to_string());
            }
        }
        Ok(())
    }
}

fn generate_response(ty: &'static str, content: &'static [u8]) -> Vec<u8> {
    let len = content.len();
    let mut response =
        format!("HTTP/1.1 200 OK\r\nContent-Length: {len}\r\nContent-Type: {ty}\r\n\r\n")
            .into_bytes();
    response.extend_from_slice(content);
    response
}
