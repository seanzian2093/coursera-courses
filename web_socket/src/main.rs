mod client;
mod server;
fn main() {
    let _ = server::main();
    client::main();
}
