use hyper::header::{HeaderMap, HeaderValue};
use hyper::Client;
use hyper::{Request, Uri};
use lazy_static::lazy_static;
use std::net::IpAddr;
use std::str::FromStr;
use unicase::Ascii;

pub fn call(request: Request<hyper::Body>) -> hyper::client::ResponseFuture {
    Client::new().request(request)
}

pub fn prepare(
    mut request: Request<hyper::Body>,
    source: IpAddr,
    target: &str,
) -> Request<hyper::Body> {
    // Strip Hop-by-Hop headers
    *request.headers_mut() = strip_hbh(request.headers());

    // Redirect to forward uri
    *request.uri_mut() = forward_uri(target, &request);

    // Add forwarding information
    let fwd_header = "x-forwarded-for";
    match request.headers_mut().entry(fwd_header) {
        Ok(header_entry) => match header_entry {
            hyper::header::Entry::Vacant(entry) => {
                entry.insert(format!("{}", source).parse().unwrap());
            }
            hyper::header::Entry::Occupied(mut entry) => {
                entry.insert(
                    format!("{}, {}", entry.get().to_str().unwrap(), source)
                        .parse()
                        .unwrap(),
                );
            }
        },
        Err(err) => panic!("Invalid header name! {}", err),
    }

    request
}

fn forward_uri<B>(forward_url: &str, req: &Request<B>) -> Uri {
    let forward_uri = match req.uri().query() {
        Some(query) => format!("{}{}?{}", forward_url, req.uri().path(), query),
        None => format!("{}{}", forward_url, req.uri().path()),
    };

    Uri::from_str(forward_uri.as_str()).unwrap()
}

fn strip_hbh(headers: &HeaderMap<HeaderValue>) -> HeaderMap<HeaderValue> {
    let mut result = HeaderMap::new();
    for (k, v) in headers.iter() {
        if !is_hbh_header(k.as_str()) {
            result.insert(k.clone(), v.clone());
        }
    }
    result
}

fn is_hbh_header(name: &str) -> bool {
    lazy_static! {
        static ref HBH_HEADERS: Vec<Ascii<&'static str>> = vec![
            Ascii::new("Connection"),
            Ascii::new("Keep-Alive"),
            Ascii::new("Proxy-Authenticate"),
            Ascii::new("Proxy-Authorization"),
            Ascii::new("Te"),
            Ascii::new("Trailers"),
            Ascii::new("Transfer-Encoding"),
            Ascii::new("Upgrade"),
        ];
    }

    HBH_HEADERS.iter().any(|h| h == &name)
}
