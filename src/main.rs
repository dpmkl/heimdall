use crate::util::{load_cert, rewrite_uri};
use futures_util::stream::StreamExt;
use hyper::server::{conn::Http as HyperHttp, Builder};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use log::error;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tls::{TlsAcceptor, TlsStream};

mod acl;
mod app;
mod config;
mod proxy;
mod router;
use router::{Router, RouterResult};
mod util;

async fn process(
    req: Request<Body>,
    peer_ip: IpAddr,
    router: Router,
) -> hyper::Result<Response<Body>> {
    match router.eval(&req) {
        RouterResult::Success(path) => {
            let req = proxy::prepare(req, peer_ip, &path).await;
            proxy::call(req).await
        }
        RouterResult::NotDefined => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("No route defined!"))
            .unwrap()),
        RouterResult::NotAllowedMethod => Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("Invalid http method!"))
            .unwrap()),
    }
}

async fn redirect_http(request: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let (parts, _) = request.into_parts();
    Ok(Response::builder()
        .status(StatusCode::MOVED_PERMANENTLY)
        .header("Location", rewrite_uri(parts.uri).to_string())
        .body(Body::from("Redirect to https"))
        .unwrap())
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = match app::run() {
        None => return,
        Some(config) => config,
    };
    let addr = config.listen;
    let pass = match &config.cert_pass {
        Some(pass) => &pass,
        None => "",
    };
    let cert = load_cert(&config.cert_file, pass);
    let tls = TlsAcceptor::from(native_tls::TlsAcceptor::builder(cert).build().unwrap());
    let tls = Arc::new(tls);

    let router = Router::from_config(config.clone());

    let mut listener = match TcpListener::bind(&addr).await {
        Err(err) => {
            error!("Could not bind socket! {}", err);
            return;
        }
        Ok(listener) => listener,
    };
    let incoming = listener.incoming();

    let service = make_service_fn(move |stream: &TlsStream<TcpStream>| {
        let router = router.clone();
        let peer = stream.get_ref().peer_addr().unwrap().ip();
        async move { Ok::<_, hyper::Error>(service_fn(move |req| process(req, peer, router.clone()))) }
    });

    let server = Builder::new(
        hyper::server::accept::from_stream(incoming.filter_map(|socket| {
            async {
                match socket {
                    Ok(stream) => match tls.clone().accept(stream).await {
                        Ok(val) => Some(Ok::<_, hyper::Error>(val)),
                        Err(err) => {
                            error!("Tls handshake error! {}", err);
                            None
                        }
                    },
                    Err(err) => {
                        error!("Tcp handshake error! {}", err);
                        None
                    }
                }
            }
        })),
        HyperHttp::new(),
    )
    .serve(service);

    if config.redirect_http {
        let addr = ([0, 0, 0, 0], 80).into();
        let redirector = Server::bind(&addr).serve(make_service_fn(|_| {
            async { Ok::<_, hyper::Error>(service_fn(redirect_http)) }
        }));
        let (http, https) = futures::join!(redirector, server);
        if let Err(err) = http {
            error!("Error during http server execution! {}", err);
        }
        if let Err(err) = https {
            error!("Error during https server execution! {}", err);
        }
    } else if let Err(err) = server.await {
        error!("Error during https server execution! {}", err);
    };
}
