mod resp;
use resp::data::RespObj;
mod debug;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use resp::data::Database;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

#[tokio::main]
async fn main() {
    let mut listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    
    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        
        // clone the handle to hashmap
        let db = db.clone();
        
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(stream, db).await;
        });
    }
}

async fn process(mut stream: TcpStream, db: Database) {
    let mut buf = [0; 1024];

    // In a loop, read data from the socket and write the data back.
    loop {
        match stream.read(&mut buf).await {
            Ok(n) if n == 0 => return, // socket closed
            Ok(n) => n, // bytes read
            Err(e) => { // Error/fail
                println!("Failed to read from socket; err = {:?}", e);
                return;
            }
        };
        
        let parsed = match RespObj::from(&buf) {
            Ok(res) => res,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };
        
        let response = match resp::interpreter::interpret(&parsed, &db) {
            Ok(res) => res,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };
        
        if let Err(e) = stream.write(response.as_bytes()).await {
            println!("Failed to write to stream; err = {:?}", e);
            return;
        }
    }
}
