use std::env;

use std::net::SocketAddr;

use crate::asyncresp::{MyError, RedisValueCodec};
use crate::{engine::Engine, ops::translate, types::RedisValue};

use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio_codec::Decoder;

fn process(socket: TcpStream, engine: Engine) {
    let (tx, rx) =
    // Frame the socket using the `Http` protocol. This maps the TCP socket
    // to a Stream + Sink of HTTP frames.
        RedisValueCodec::default().framed(socket)
    // This splits a single `Stream + Sink` value into two separate handles
    // that can be used independently (even on different tasks or threads).
        .split();

    // Map all requests into responses and send them back to the client.
    let task = tx
        .send_all(rx.and_then(move |r: RedisValue| match translate(&r) {
            Ok(op) => {
                let res = engine.clone().exec(op);
                Ok(RedisValue::from(res))
            }
            Err(e) => Ok(RedisValue::from(e)),
        }))
        .then(|res| {
            if let Err(e) = res {
                println!("failed to process connection; error = {:?}", e);
            }

            Ok(())
        });

    // Spawn the task that handles the connection.
    tokio::spawn(task);
}

pub fn server(engine: Engine) -> Result<(), MyError> {
    // Parse the address we're going to run this server on
    // and set up our TCP listener to accept connections.
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>().expect("Cannot bind to port!");

    let listener = TcpListener::bind(&addr).expect("it to work");
    println!("Listening on: {}", addr);

    tokio::run({
        listener
            .incoming()
            .map_err(|e| println!("failed to accept socket; error = {:?}", e))
            .for_each(move |socket| {
                process(socket, engine.clone());
                Ok(())
            })
    });
    Ok(())
}
