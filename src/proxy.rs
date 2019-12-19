use hyper::header::{HeaderMap, HeaderValue};
use hyper::Client;
use hyper::{Request, Uri};
use lazy_static::lazy_static;
use std::net::IpAddr;
use unicase::Ascii;

pub fn call(request: Request<hyper::Body>) -> hyper::client::ResponseFuture {
    Client::new().request(request)
}

pub async fn prepare(
    mut request: Request<hyper::Body>,
    source: IpAddr,
    target: Uri,
) -> Request<hyper::Body> {
    // Strip Hop-by-Hop headers
    *request.headers_mut() = strip_hbh(request.headers());

    // Redirect to forward uri
    *request.uri_mut() = target;

    // Add forwarding information
    let fwd_header = "x-forwarded-for";
    request
        .headers_mut()
        .entry(fwd_header)
        .or_insert_with(|| format!("{}", source).parse().unwrap());
    request
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
