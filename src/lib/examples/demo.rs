use jumpdrive::prelude::*;

fn main() -> JumpdriveResult {
        ::jumpdrive::Jumpdrive::new(::phf::phf_map!{
    "scripts/main.js" => (include_bytes!("/home/seb-hyland/Documents/jumpdrive/src/lib/public/scripts/main.js"),"application/javascript"),"index.html" => (include_bytes!("/home/seb-hyland/Documents/jumpdrive/src/lib/public/index.html"),"text/html")
},Some(("/ws",websocket_handler)), ::phf::phf_map!{
    "/csv" => csv_server
},error_handler,).serve()
}

fn websocket_handler(ws: &mut Websocket) {
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
        serve_file(stream, "crates/lib/examples/file.csv", ContentType::Text(ContentTypeText::Plain))
}
fn error_handler(e: Error) {
        println!("MAJOR OOPSIES: {e}");
}
