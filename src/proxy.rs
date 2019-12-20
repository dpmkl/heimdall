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

fn is_hbh_header(name: &str) -> bool {
    HBH_HEADERS.iter().any(|h| h == &name)
}

#[cfg(test)]
mod tests {
    use super::{strip_hbh, HBH_HEADERS};
    use hyper::header::{HeaderMap, HeaderValue};

    fn headers_map() -> HeaderMap<HeaderValue> {
        let mut headers = HeaderMap::new();
        headers.insert("CONNECTION", "val: T".parse().unwrap());
        headers.insert("keep-alive", "val: T".parse().unwrap());
        headers.insert("PROXY-Authenticate", "val: T".parse().unwrap());
        headers.insert("Proxy-Authorization", "val: T".parse().unwrap());
        headers.insert("te", "val: T".parse().unwrap());
        headers.insert("TRAILERS", "val: T".parse().unwrap());
        headers.insert("te", "val: T".parse().unwrap());
        headers.insert("Transfer-Encoding", "val: T".parse().unwrap());
        headers.insert("Upgrade", "val: T".parse().unwrap());
        headers
    }

    #[test]
    fn strip_headers() {
        let headers = headers_map();
        assert_eq!(headers.len(), HBH_HEADERS.len());
        assert_eq!(strip_hbh(&headers), HeaderMap::new());

        const HEADER: &str = "myheader";
        let mut headers = headers_map();
        headers.insert(HEADER, HEADER.parse().unwrap());
        assert_eq!(headers.len(), HBH_HEADERS.len() + 1);
        let headers = strip_hbh(&headers);
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[HEADER], HEADER);
    }
}
