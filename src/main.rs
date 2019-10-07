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
use tokio_net::tcp::{TcpListener, TcpStream};
use tokio_tls::TlsAcceptor;

// fn load_certs(path: &Path) -> io::Result<Vec<Certificate>> {
//     certs(&mut BufReader::new(File::open(path)?))
//         .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
// }

// fn load_keys(path: &Path) -> io::Result<Vec<PrivateKey>> {
//     rsa_private_keys(&mut BufReader::new(File::open(path)?))
//         .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
// }

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
            Ok((tcp, _)) => tcp,
            Err(err) => {
                continue;
            }
        };
        tokio::spawn( async move {
            let mut stream = match tls.accept(tcp).await {
                Ok(stream) => stream,
                Err(err) => {
                    println!("Failed TLS handshake! {}", err);
                    return;
                }
            };
            let mut buff = [0; 1024];
            match stream.read(&mut buff).await {
                Ok(size) => {
                    println!("Data({}): {:?}", size, buff.to_vec().hex_dump());
                }
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        });
    }


    Ok(())
}
