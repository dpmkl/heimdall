use futures_util::stream::StreamExt;
use hyper::server::{conn::Http as HyperHttp, Builder};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Client, Error};
use native_tls;
use native_tls::Identity;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::tcp::TcpListener;
use tokio_tls::TlsAcceptor;


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

    let service = make_service_fn(move |_| {
        async move {
            Ok::<_, Error>(service_fn(move |mut req| {
                let client = Client::new();
                client.request(req)
                // let uri_string = format!("http://{}/{}",
                //                          out_addr_clone,
                //                          req.uri().path_and_query().map(|x| x.as_str()).unwrap_or(""));
                // let uri = uri_string.parse().unwrap();
                // *req.uri_mut() = uri;
                // client.request(req)
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
}
