pub use crate::{Error, IoError, JumpdriveResult};
pub use helpers::{
        IoResult, Stream,
        content_types::{ContentType, ContentTypeApplication, ContentTypeImage, ContentTypeText},
        generate_response, serve_file,
};
pub use phf::{self, phf_map};
pub use proc::jumpdrive;
/// An alias for `tungstenite::WebSocket<std::net::TcpStream>`
pub type Websocket = tungstenite::WebSocket<std::net::TcpStream>;
