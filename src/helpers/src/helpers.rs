use crate::content_types::ContentType;
use std::{
        fs,
        io::{self, Write},
        net::TcpStream,
        path::Path,
};
pub mod content_types;

/// Generates a response with a valid HTTP/1.1 header
pub fn generate_response<I: ToString>(mime_type: I, content: &[u8]) -> Vec<u8> {
        fn inner(ty: String, content: &[u8]) -> Vec<u8> {
                let len = content.len();
                let mut response = format!("HTTP/1.1 200 OK\r\nContent-Length: {len}\r\nContent-Type: {ty}\r\n\r\n").into_bytes();
                response.extend_from_slice(content);
                response
        }
        inner(mime_type.to_string(), content)
}

/// Serves a file at runtime
pub fn serve_file<P: AsRef<Path>>(stream: &mut TcpStream, path: P, ty: ContentType) -> Result<(), io::Error> {
        fn inner(stream: &mut TcpStream, path: &Path, ty: ContentType) -> Result<(), io::Error> {
                let file_contents = fs::read(path)?;
                let response = generate_response(ty, &file_contents);
                stream.write_all(&response)?;
                Ok(())
        }
        inner(stream, path.as_ref(), ty)
}
