use futures_util::stream::StreamExt;
use futures_util::future::FutureExt;
use futures::Future;
use hyper::{Request, Response, Body, StatusCode};
use hyper::server::{conn::Http as HyperHttp, Builder};
use hyper::service::{make_service_fn, service_fn};
use native_tls::Identity;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::tcp::{TcpListener, TcpStream};
use tokio_tls::{TlsAcceptor, TlsStream};

#[macro_use]
extern crate lazy_static;

mod proxy;
mod router;

async fn process(req: Request<Body>, peer_ip: IpAddr) -> hyper::Result<Response<Body>> {
    if false {
        let req = proxy::prepare(req, peer_ip, "127.0.0.1");        
        proxy::call(req).await
    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("No route!"))
            .unwrap())        
    }
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8443);
    let der = include_bytes!("../identity.p12");
    let cert = Identity::from_pkcs12(der, "mypass").unwrap();
    let tls = TlsAcceptor::from(native_tls::TlsAcceptor::builder(cert).build().unwrap());
    let tls = Arc::new(tls);

    let listener = match TcpListener::bind(&addr).await {
        Err(err) => {
            println!("Could not bind socket! {}", err);
            return;
        }
        Ok(listener) => listener,
    };
    let incoming = listener.incoming();

    let service = make_service_fn(move |stream: &TlsStream<TcpStream>| {
        let peer = stream.get_ref().peer_addr().unwrap().ip();        
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                process(req, peer)                
            }))                
        }
    });

    let server = Builder::new(
        hyper::server::accept::from_stream(incoming.filter_map(|socket| {
            async {
                match socket {
                    Ok(stream) => match tls.clone().accept(stream).await {
                        Ok(val) => Some(Ok::<_, hyper::Error>(val)),
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

    if let Err(err) = server.await {
        println!("Error running proxy! {}", err);
    }
}
