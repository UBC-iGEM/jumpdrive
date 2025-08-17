use crate::content_types::ContentType;
use std::{
        fmt::Display,
        fs,
        io::{self, Write},
        net::TcpStream,
        path::Path,
};
pub mod content_types;

/// Generates a response with a valid HTTP/1.1 header
pub fn generate_response<I: Display>(mime_type: I, content: &[u8]) -> Vec<u8> {
        let len = content.len();
        let mut response = format!("HTTP/1.1 200 OK\r\nContent-Length: {len}\r\nContent-Type: {mime_type}\r\n\r\n").into_bytes();
        response.extend_from_slice(content);
        response
}

/// An alias for [`std::net::TcpStream`]
pub type Stream = TcpStream;
/// An alias for `Result<(), E>` where `E` is [`std::io::Error`]
pub type IoResult = io::Result<()>;

/// Serves a file at runtime
pub fn serve_file<P: AsRef<Path>>(stream: &mut Stream, path: P, ty: ContentType) -> IoResult {
        fn inner(stream: &mut Stream, path: &Path, ty: ContentType) -> IoResult {
                let file_contents = fs::read(path)?;
                let response = generate_response(ty, &file_contents);
                stream.write_all(&response)?;
                Ok(())
        }
        inner(stream, path.as_ref(), ty)
}
