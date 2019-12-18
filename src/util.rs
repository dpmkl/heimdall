use hyper::http::uri::{Authority, Scheme};
use hyper::http::Uri;
use native_tls::Identity;

pub fn load_cert(file: &str, pass: &str) -> Identity {
    use std::io::Read;
    let mut file = std::fs::File::open(file).unwrap();
    let mut identity = vec![];
    file.read_to_end(&mut identity).unwrap();
    Identity::from_pkcs12(&identity, &pass).unwrap()
}

pub fn rewrite_uri(uri: Uri) -> Uri {
    let parts = uri.into_parts();
    Uri::builder()
        .scheme(Scheme::HTTPS)
        .authority(
            parts
                .authority
                .unwrap_or(Authority::from_static("localhost")),
        )
        .path_and_query(parts.path_and_query.unwrap())
        .build()
        .unwrap()
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_rewrite_uri() {
        use super::rewrite_uri;
        use hyper::http::Uri;

        let uri = "http://www.rust-lang.org/".parse::<Uri>().unwrap();
        let uri = rewrite_uri(uri);
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(uri.host(), Some("www.rust-lang.org"));
        assert_eq!(uri.path(), "/");
        assert_eq!(uri.query(), None);

        let uri = "http://www.rust-lang.org/install.html?foo=bar&bar=foo"
            .parse::<Uri>()
            .unwrap();
        let uri = rewrite_uri(uri);
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(uri.host(), Some("www.rust-lang.org"));
        assert_eq!(uri.path(), "/install.html");
        assert_eq!(uri.query(), Some("foo=bar&bar=foo"));
    }
}
