use bytes::BytesMut;
use httparse::{parse_headers, Request, Response, Status as HttpStatus};
use native_tls;
use native_tls::Identity;
use pretty_hex::*;
use std::fs::File;
use std::io::{self, BufReader};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use structopt::StructOpt;
//use futures::StreamExt;
use futures_util::StreamExt;
// use tokio::prelude::*;
// use tokio_io::BufMut;
use tokio_io::{AsyncReadExt, AsyncWriteExt, BufMut};
use tokio_net::tcp::TcpListener;
use tokio_tls::TlsAcceptor;

#[macro_use]
extern crate lazy_static;

mod router;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8443);
    let der = include_bytes!("../identity.p12");
    let cert = Identity::from_pkcs12(der, "mypass")?;
    let tls = tokio_tls::TlsAcceptor::from(native_tls::TlsAcceptor::builder(cert).build()?);

    let mut listener = match TcpListener::bind(&addr).await {
        Err(err) => panic!("Could not bind socket! {}", err),
        Ok(listener) => listener,
    };

    loop {
        let tls = tls.clone();
        println!("loop");
        let tcp = match listener.accept().await {
            Ok((tcp, peer_addr)) => tcp,
            Err(err) => {
                println!("Failed TCP handeshake! {}", err);
                continue;
            }
        };
        tokio::spawn(async move {
            let mut stream = match tls.accept(tcp).await {
                Ok(stream) => stream,
                Err(err) => {
                    println!("Failed TLS handshake! {}", err);
                    return;
                }
            };
            let mut buff = [0; 4096];
            match stream.read(&mut buff).await {
                Ok(size) => {
                    let mut headers = [httparse::EMPTY_HEADER; 64];
                    let mut req = Request::new(&mut headers);
                    if let Err(err) = req.parse(&buff[..size]) {
                        println!("Failed HTTP request! {}", err);
                        return;
                    }
                    println!("{:?} {:?} {:?}", req.method, req.path, req.version);
                    println!("{:?}", &headers[..]);
                }
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        });
    }
}
