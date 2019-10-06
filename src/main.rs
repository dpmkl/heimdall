use std::fs::File;
use std::io::{self, BufReader};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use structopt::StructOpt;
// use futures::StreamExt;
// use tokio::prelude::*;
// use tokio_io::BufMut;
use tokio_io::{AsyncReadExt, AsyncWriteExt};
use tokio_net::tcp::TcpListener;
use tokio_rustls::rustls::internal::pemfile::{certs, rsa_private_keys};
use tokio_rustls::rustls::{Certificate, NoClientAuth, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;

#[derive(StructOpt)]
struct Options {
    #[structopt(short = "c", long = "cert", parse(from_os_str))]
    cert: PathBuf,

    #[structopt(short = "k", long = "key", parse(from_os_str))]
    key: PathBuf,
}

fn load_certs(path: &Path) -> io::Result<Vec<Certificate>> {
    certs(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
}

fn load_keys(path: &Path) -> io::Result<Vec<PrivateKey>> {
    rsa_private_keys(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::from_args();
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8443);
    let certs = load_certs(&options.cert).expect("Could not load cert!");
    let mut keys = load_keys(&options.key).expect("Could not load keys!");
    let mut config = ServerConfig::new(NoClientAuth::new());
    config
        .set_single_cert(certs, keys.remove(0))
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))
        .expect("Could not create TLS config!");
    let acceptor = TlsAcceptor::from(Arc::new(config));
    println!("starting ...");

    let mut listener = TcpListener::bind(&addr)
        .await
        .expect("Could not bind to socket!");
    loop {
        let acceptor = acceptor.clone();
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let acceptor = acceptor.clone();
            let mut stream = acceptor.accept(stream).await;
            let mut stream = match stream {
                Ok(stream) => stream,
                Err(err) => {
                    return;
                }
            };
            let (mut stream, _) = stream.into_inner();
            let mut buf = vec![];
            if let Err(err) = stream.read_to_end(&mut buf).await {
                return;
            }
            println!("data: {:?}", buf);
        });
    }

    Ok(())
}
