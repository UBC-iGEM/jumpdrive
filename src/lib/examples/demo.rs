use jumpdrive::prelude::*;

fn main() -> JumpdriveResult {
        jumpdrive! {
                dir = "public/",
                ws = "/ws": websocket_handler,
                routes = {
                        "/csv": csv_server
                },
                err = error_handler
        }
}

fn websocket_handler(mut ws: Websocket) {
        loop {
                if let Ok(msg) = ws.read() {
                        println!("WS message read: {msg}");
                        if msg.is_text() {
                                ws.send(msg).unwrap();
                        }
                }
        }
}
fn csv_server(stream: &mut Stream) -> IoResult {
        serve_file(stream, "src/lib/examples/file.csv", ContentType::Text(ContentTypeText::Plain))
}
fn error_handler(e: Error) {
        println!("OOPS: {e}");
}
