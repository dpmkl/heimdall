use crate::config::Config;
use log::error;
use rustls::internal::pemfile;
use rustls::ServerConfig;
use std::{fs, io, sync::Arc};

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
