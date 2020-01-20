use hyper::http::uri::{Authority, Scheme};
use hyper::http::Uri;
use hyper::{Body, Method, Request};
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;

// Upgrade the uri scheme from HTTP to HTTPS
pub fn rewrite_uri_scheme(uri: Uri) -> Uri {
    let parts = uri.into_parts();
    Uri::builder()
        .scheme(Scheme::HTTPS)
        .authority(
            parts
                .authority
                .unwrap_or_else(|| Authority::from_static("localhost")),
        )
        .path_and_query(parts.path_and_query.unwrap())
        .build()
        .unwrap()
}

// This checks if the incoming request is done by an ACME bot
// Checks if the path has 4 elements based on the example challenge request from documentation
// First '.well-known', second 'acme-challenge', the third being the token
// Example challenge:
//   http://domain/.well-known/acme-challenge/HGr8U1IeTW4kY_Z6UIyaakzOkyQgPr_7ArlLgtZE8SX
// Returns file name of the token as String
pub fn is_acme_challenge(request: &Request<Body>) -> Option<String> {
    const ACME_FIRST: &str = ".well-known";
    const ACME_SECOND: &str = "acme-challenge";
    let path = Path::new(request.uri().path());
    let components: Vec<_> = path.components().map(|comp| comp.as_os_str()).collect();
    if request.method() != Method::GET {
        None
    } else if components.len() == 4 {
        if components[0] == OsStr::new("/")
            && components[1] == OsStr::new(ACME_FIRST)
            && components[2] == OsStr::new(ACME_SECOND)
        {
            Some(components[3].to_str().unwrap().to_owned())
        } else {
            None
        }
    } else {
        None
    }
}

pub fn get_token(webroot: &str, token: &str) -> Option<Vec<u8>> {
    let path = Path::new(webroot).join(token);
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return None,
    };
    let mut token = vec![];
    if file.read_to_end(&mut token).is_ok() {
        Some(token)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{is_acme_challenge, rewrite_uri_scheme};
    use hyper::http::Uri;
    use hyper::{Body, Method, Request};
    use std::str::FromStr;

    fn build_req(uri: &str, method: hyper::Method) -> Request<Body> {
        Request::builder()
            .uri(uri)
            .method(method)
            .body(Body::from("asdf"))
            .unwrap()
    }

    #[test]
    fn check_acme_challenge() {
        assert_eq!(
            is_acme_challenge(&build_req("http://foo.bar/.well-known/acme-challenge/HGr8U1IeTW4kY_Z6UIyaakzOkyQgPr_7ArlLgtZE8SX", Method::GET)),
            Some("HGr8U1IeTW4kY_Z6UIyaakzOkyQgPr_7ArlLgtZE8SX".to_owned())
        );
        assert_eq!(
            is_acme_challenge(&build_req("http://foo.bar/.well-known/acme-challenge/HGr8U1IeTW4kY_Z6UIyaakzOkyQgPr_7ArlLgtZE8SX", Method::PUT)),
            None,
        );
        assert_eq!(
            is_acme_challenge(&build_req("http://foo.bar/", Method::GET)),
            None
        );
        assert_eq!(
            is_acme_challenge(&build_req("http://foo.bar/../../etc/passwd", Method::GET)),
            None
        );
        assert_eq!(
            is_acme_challenge(&build_req("http://foo.bar/..../etc/passwd", Method::GET)),
            None
        );
        assert_eq!(
            is_acme_challenge(&build_req("http://foo.bar/../../*", Method::GET)),
            None
        );
    }

    #[test]
    fn check_rewrite_uri() {
        assert_eq!(
            rewrite_uri_scheme(Uri::from_str("http://www.foo.bar").unwrap()),
            Uri::from_str("https://www.foo.bar").unwrap()
        );
        assert_eq!(
            rewrite_uri_scheme(Uri::from_str("https://www.foo.bar").unwrap()),
            Uri::from_str("https://www.foo.bar").unwrap()
        );
        assert_eq!(
            rewrite_uri_scheme(Uri::from_str("http://www.foo.bar/?foo=bar").unwrap()),
            Uri::from_str("https://www.foo.bar/?foo=bar").unwrap()
        );
        assert_eq!(
            rewrite_uri_scheme(Uri::from_str("http://www.foo.bar/qwerty?foo=bar").unwrap()),
            Uri::from_str("https://www.foo.bar/qwerty?foo=bar").unwrap()
        );
        assert_eq!(
            rewrite_uri_scheme(Uri::from_str("http://localhost").unwrap()),
            Uri::from_str("https://localhost").unwrap()
        );

        let uri = "http://www.rust-lang.org/".parse::<Uri>().unwrap();
        let uri = rewrite_uri_scheme(uri);
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(uri.host(), Some("www.rust-lang.org"));
        assert_eq!(uri.path(), "/");
        assert_eq!(uri.query(), None);

        let uri = "http://www.rust-lang.org/install.html?foo=bar&bar=foo"
            .parse::<Uri>()
            .unwrap();
        let uri = rewrite_uri_scheme(uri);
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(uri.host(), Some("www.rust-lang.org"));
        assert_eq!(uri.path(), "/install.html");
        assert_eq!(uri.query(), Some("foo=bar&bar=foo"));
    }
}
