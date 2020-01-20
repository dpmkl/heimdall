use crate::config::Config;
use core::task::{Context, Poll};
use futures_util::stream::Stream;
use hyper::server::accept::Accept;
use log::error;
use rustls::internal::pemfile;
use rustls::ServerConfig;
use std::pin::Pin;
use std::{fs, io, sync::Arc};
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

pub struct HyperAcceptor<'a> {
    pub acceptor: Pin<Box<dyn Stream<Item = Result<TlsStream<TcpStream>, io::Error>> + 'a>>,
}

impl Accept for HyperAcceptor<'_> {
    type Conn = TlsStream<TcpStream>;
    type Error = io::Error;

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        Pin::new(&mut self.acceptor).poll_next(cx)
    }
}

pub fn create_config(config: &Config) -> Option<Arc<ServerConfig>> {
    let certs = match load_certs(&config.cert_file) {
        Ok(certs) => certs,
        Err(err) => {
            error!("Could not load certificate! {}", err);
            return None;
        }
    };
    let pkey = match load_private_key(&config.pkey_file) {
        Ok(pkey) => pkey,
        Err(err) => {
            error!("Could not load private key! {}", err);
            return None;
        }
    };
    let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
    if let Err(err) = cfg.set_single_cert(certs, pkey) {
        error!("Could not setup TLS! {}", err);
        return None;
    }
    cfg.set_protocols(&[b"h2".to_vec(), b"http/1.1".to_vec()]);
    Some(Arc::new(cfg))
}

fn error(err: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}

fn load_certs(filename: &str) -> io::Result<Vec<rustls::Certificate>> {
    let certfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(certfile);

    pemfile::certs(&mut reader).map_err(|_| error("failed to load certificate".into()))
}

fn load_private_key(filename: &str) -> io::Result<rustls::PrivateKey> {
    let keyfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(keyfile);

    let keys = pemfile::pkcs8_private_keys(&mut reader)
        .map_err(|_| error("failed to load private key".into()))?;
    if keys.len() != 1 {
        return Err(error("expected a single private key".into()));
    }
    Ok(keys[0].clone())
}
