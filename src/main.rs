use futures_util::stream::StreamExt;
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

mod app;
mod config;
mod proxy;
mod router;
use router::Router;

async fn process(req: Request<Body>, peer_ip: IpAddr, router: Router) -> hyper::Result<Response<Body>> {
    match router.eval(req.uri().path()) {
        Some(path) => {
            let req = proxy::prepare(req, peer_ip, &path);        
            proxy::call(req).await
        }
        None => {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("No route!"))
                .unwrap())        
        }
    }    
}

#[tokio::main]
async fn main() {
    let config = match app::run() {
        None => return, 
        Some(config) => config,
    };    
    let addr = config.listen.clone();
    let der = include_bytes!("../identity.p12");
    let cert = Identity::from_pkcs12(der, "mypass").unwrap();
    let tls = TlsAcceptor::from(native_tls::TlsAcceptor::builder(cert).build().unwrap());
    let tls = Arc::new(tls);
    
    let router = Router::new();

    let listener = match TcpListener::bind(&addr).await {
        Err(err) => {
            println!("Could not bind socket! {}", err);
            return;
        }
        Ok(listener) => listener,
    };
    let incoming = listener.incoming();

    let service = make_service_fn(move |stream: &TlsStream<TcpStream>| {
        let router = router.clone();
        let peer = stream.get_ref().peer_addr().unwrap().ip();        
        async move {        
            Ok::<_, hyper::Error>(service_fn(move |req| {                
                process(req, peer, router.clone())                
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