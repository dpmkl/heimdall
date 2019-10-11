use futures_util::stream::StreamExt;
use futures::io::{AsyncReadExt, AsyncWriteExt};
use hyper::server::{
    conn::{AddrStream, Http as HyperHttp},
    Builder,
};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Client, Error};
use native_tls::Identity;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::tcp::{TcpListener, TcpStream};
use tokio_executor::threadpool::{Builder as RuntimeBuilder, ThreadPool};
use tokio_tls::{TlsAcceptor, TlsStream};

// mod router;
// mod util;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8443);
    let der = include_bytes!("../identity.p12");
    let cert = Identity::from_pkcs12(der, "mypass").unwrap();
    let tls = TlsAcceptor::from(native_tls::TlsAcceptor::builder(cert).build().unwrap());
    let tls = Arc::new(tls);

    let mut listener = match TcpListener::bind(&addr).await {
        Err(err) => {
            println!("Could not bind socket! {}", err);
            return;
        }
        Ok(listener) => listener,
    };
    let incoming = listener.incoming();

    let service = make_service_fn(move |stream: &TlsStream<TcpStream>| {
        let peer = stream.get_ref().peer_addr().unwrap();        
        println!("Peer: {:#?}\n", peer);
        async move {
            Ok::<_, Error>(service_fn(move |mut req| {
                let client = Client::new();
                println!("Http: {:?}", req);
                let uri_string = format!(
                    "http://{}/{}",
                    "127.0.0.1:17571",
                    req.uri().path_and_query().map(|x| x.as_str()).unwrap_or("")
                );
                let uri = uri_string.parse().unwrap();
                *req.uri_mut() = uri;
                client.request(req)
            }))
        }
    });

    let server = Builder::new(
        hyper::server::accept::from_stream(incoming.filter_map(|socket| {
            async {
                match socket {
                    Ok(stream) => match tls.clone().accept(stream).await {
                        Ok(val) => {
                            Some(Ok::<_, hyper::Error>(val))
                        }
                        Err(err) => {
                            println!("Tls handshake error! {}", err);
                            None
                        }
                    },
                    Err(err) => {
                        println!("Tcp handshake error! {}", err);
                        None
                    }
                }
            }
        })),
        HyperHttp::new(),
    )
    .serve(service);

    server.await;
}
