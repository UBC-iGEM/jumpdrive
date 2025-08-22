use helpers::generate_response;
use phf::Map;
pub use proc::jumpdrive;
use std::{
        env, error,
        fmt::Display,
        io::{self, Read, Write},
        net::{TcpListener, TcpStream},
        sync::Arc,
        thread,
};
use tungstenite::{HandshakeError, ServerHandshake, WebSocket, accept, handshake::server::NoCallback};

pub mod prelude;

/// The internal representation of a Jumpdrive process.\
/// **Note**: this type should never be constructed directly.
/// Instead, use the [`jumpdrive!`] macro.
#[derive(Debug)]
pub struct Jumpdrive {
        map: Map<&'static str, (&'static [u8], &'static str)>,
        socket: Option<Socket>,
        other_paths: Map<&'static str, CustomEndpoint>,
        error_handler: ErrorHandler,
}
/// An alias for [`std::io::Error`]
pub type IoError = io::Error;

type Socket = (&'static str, fn(&mut WebSocket<TcpStream>));
type CustomEndpoint = fn(&mut TcpStream) -> io::Result<()>;
type ErrorHandler = fn(Error);

/// A specialized Result type for the [`jumpdrive!`] macro.
/// # Returns
/// - Err if binding to a [`TcpListener`] fails on the requested IP and PORT
/// - Should never realistically return otherwise
pub type JumpdriveResult = io::Result<()>;

impl Jumpdrive {
        /// DO NOT manually construct a [`Jumpdrive`] instance; instead, use the [`jumpdrive!`] macro.
        pub fn new(
                map: Map<&'static str, (&'static [u8], &'static str)>,
                socket: Option<Socket>,
                other_paths: Map<&'static str, CustomEndpoint>,
                error_handler: ErrorHandler,
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
                        let conn = match connection {
                                Ok(c) => c,
                                Err(e) => {
                                        (config.error_handler)(Error::ClientConnectionFailure(e));
                                        continue;
                                }
                        };
                        let mut conn_clone = match conn.try_clone() {
                                Ok(clone) => clone,
                                Err(e) => {
                                        (config.error_handler)(Error::ClientConnectionFailure(e));
                                        continue;
                                }
                        };
                        let config = Arc::clone(&config);
                        thread::spawn(move || {
                                let output = || -> Result<(), Error> {
                                        let mut buffer = [0; 1024];
                                        let len = conn_clone.peek(&mut buffer).map_err(Error::RequestReadFailure)?;
                                        if len == 0 {
                                                return Err(Error::EmptyRequestError);
                                        }
                                        let request = String::from_utf8_lossy(&buffer[0..len]);
                                        let line = match request.lines().next() {
                                                Some(l) => l,
                                                None => return Err(Error::MalformedRequestError),
                                        };
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
                                                                conn_clone.set_nonblocking(true).map_err(Error::DeblockFailure)?;
                                                                let mut ws = accept(conn_clone)
                                                                        .map_err(|e| Error::WebsocketHandshakeError(Box::new(e)))?;
                                                                socket_handler(&mut ws);
                                                                return Ok(());
                                                        }
                                                        if let Some((asset, content_type)) = config.map.get(&path[1..]) {
                                                                let response = generate_response(content_type, asset);
                                                                conn_clone.write_all(&response).map_err(Error::ServeFailure)?;
                                                        }
                                                        if let Some(callback) = config.other_paths.get(path) {
                                                                callback(&mut conn_clone).map_err(Error::ServeFailure)?;
                                                        }
                                                        clear_socket(&mut conn_clone);
                                                        Err(Error::ConfusedMonkey(path.to_string()))
                                                }
                                        } else {
                                                Err(Error::MalformedRequestError)
                                        }
                                };
                                if let Err(e) = output() {
                                        (config.error_handler)(e);
                                }
                        });
                }
                Ok(())
        }
}

fn clear_socket(stream: &mut TcpStream) {
        let mut buf = [0; 2048];
        let _ = stream.read(&mut buf);
}

/// A specialized Error type to represent failures during server execution
#[derive(Debug)]
pub enum Error {
        /// Failed to connect to an incoming client
        ClientConnectionFailure(IoError),
        /// Failed to read a client's request
        RequestReadFailure(IoError),
        /// Empty client request
        EmptyRequestError,
        /// Malformed client request
        MalformedRequestError,
        /// Failed to make stream nonblocking
        DeblockFailure(IoError),
        /// Failed Websocket handshake
        WebsocketHandshakeError(Box<HandshakeFailure>),
        /// Failed while serving a custom endpoint
        ServeFailure(IoError),
        /// Couldn't find a matching endpoint 😕
        ConfusedMonkey(String),
}
type HandshakeFailure = HandshakeError<ServerHandshake<TcpStream, NoCallback>>;

/// Displays an execution error message, as well as any propogated failure
impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                        Self::ClientConnectionFailure(e) => write!(f, "Failed to connect to an incoming client: {e}"),
                        Self::RequestReadFailure(e) => write!(f, "Failed to read a client's request: {e}"),
                        Self::EmptyRequestError => write!(f, "Empty client request"),
                        Self::MalformedRequestError => write!(f, "Malformed client request"),
                        Self::DeblockFailure(e) => write!(f, "Failed to make stream nonblocking: {e}"),
                        Self::WebsocketHandshakeError(e) => write!(f, "Failed Websocket handshake: {e}"),
                        Self::ServeFailure(e) => write!(f, "Failed while serving a custom endpoint: {e}"),
                        Self::ConfusedMonkey(p) => write!(f, "Couldn't find a matching endpoint for path {p}"),
                }
        }
}
impl error::Error for Error {}
