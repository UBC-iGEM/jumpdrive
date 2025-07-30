pub use crate::{Error, IoError, IoResult, JumpdriveResult};
pub use helpers::{
        content_types::{ContentType, ContentTypeApplication, ContentTypeImage, ContentTypeText},
        generate_response, serve_file,
};
pub use proc::jumpdrive;
/// An alias for `std::net::TcpStream`
pub type Stream = std::net::TcpStream;
/// An alias for `tungstenite::WebSocket<std::net::TcpStream>`
pub type Websocket = tungstenite::WebSocket<std::net::TcpStream>;
