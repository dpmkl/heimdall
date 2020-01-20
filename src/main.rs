use crate::util::{get_token, is_acme_challenge, rewrite_uri_scheme};
use futures_util::stream::StreamExt;
use hyper::server::{conn::Http as HyperHttp, Builder};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use log::{error, info};
use std::net::IpAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{server::TlsStream, TlsAcceptor};

mod acl;
mod app;
mod config;
mod proxy;
mod router;
use router::{Router, RouterResult};
mod tls;
mod util;

async fn handle_proxy(
    req: Request<Body>,
    peer_ip: IpAddr,
    router: Router,
) -> hyper::Result<Response<Body>> {
    match router.eval(&req) {
        RouterResult::Success(uri) => {
            let req = proxy::prepare(req, peer_ip, uri).await;
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

#[allow(clippy::unnecessary_unwrap)]
async fn handle_auxiliary(
    request: Request<Body>,
    http_redirect: bool,
    acme_web_root: Option<String>,
) -> hyper::Result<Response<Body>> {
    let token = is_acme_challenge(&request);
    if token.is_some() && acme_web_root.is_some() {
        if let Some(token) = get_token(&acme_web_root.unwrap(), &token.unwrap()) {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(token))
                .unwrap())
        } else {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Token not found!"))
                .unwrap())
        }
    } else if http_redirect {
        redirect_to_https(request).await
    } else {
        invalid_request().await
    }
}

async fn redirect_to_https(request: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let (parts, _) = request.into_parts();
    Ok(Response::builder()
        .status(StatusCode::MOVED_PERMANENTLY)
        .header("Location", rewrite_uri_scheme(parts.uri).to_string())
        .body(Body::from("Redirect to https"))
        .unwrap())
}

async fn invalid_request() -> Result<Response<Body>, hyper::Error> {
    Ok::<_, hyper::Error>(
        Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("Invalid request!"))
            .unwrap(),
    )
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = match app::run() {
        None => return,
        Some(config) => config,
    };
    let addr = config.listen;
    let router = Router::from_config(config.clone());

    let tls_cfg = match tls::create_config(&config) {
        Some(cfg) => cfg,
        None => {
            error!("No valid TLS config! Exting ...");
            return;
        }
    };

    let mut tcp = match TcpListener::bind(&addr).await {
        Err(err) => {
            error!("Could not bind socket! {}", err);
            return;
        }
        Ok(tcp) => tcp,
    };
    let tls_acceptor = TlsAcceptor::from(tls_cfg);
    let tls_incoming = tcp.incoming();
    let proxy_service = make_service_fn(move |stream: &TlsStream<TcpStream>| {
        let router = router.clone();
        let peer = stream.get_ref().0.peer_addr().unwrap().ip();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handle_proxy(req, peer, router.clone())
            }))
        }
    });
    let tls_server = Builder::new(
        hyper::server::accept::from_stream(tls_incoming.filter_map(|socket| async {
            match socket {
                Ok(stream) => match tls_acceptor.accept(stream).await {
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
        })),
        HyperHttp::new(),
    )
    .serve(proxy_service);

    let https_upgrade = config.redirect_to_https;
    let acme_web_root = config.acme_web_root.clone();

    info!("Starting up ");

    if https_upgrade || acme_web_root.is_some() {
        let util_service = make_service_fn(move |_| {
            let acme_web_root = acme_web_root.clone();
            async move {
                let acme_web_root = acme_web_root.clone();
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    handle_auxiliary(req, https_upgrade, acme_web_root.clone())
                }))
            }
        });

        let addr = "0.0.0.0:80".parse().unwrap();
        let http_server = Server::bind(&addr).serve(util_service);
        let (http, https) = futures::join!(http_server, tls_server);
        if let Err(err) = http {
            error!("Error during http server execution! {}", err);
        }
        if let Err(err) = https {
            error!("Error during https server execution! {}", err);
        }
    } else if let Err(err) = tls_server.await {
        error!("Error during https server execution! {}", err);
    };
}
