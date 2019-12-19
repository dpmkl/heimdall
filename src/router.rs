use crate::acl::{parse_allowed_methods, AllowedMethods};
use crate::config::Config;
use hyper::http::uri::{Authority, Scheme};
use hyper::{Body, Request, Uri};
use path_tree::PathTree;
use std::net::SocketAddr;
use std::str::FromStr;

fn make_path(path: String) -> String {
    let mut path = path.replace("//", "/");
    if !path.starts_with('/') {
        path = format!("/{}", path);
    }
    if path.ends_with('/') {
        path.remove(path.len() - 1);
    }
    path
}

#[derive(Clone)]
pub struct Target {
    addr: SocketAddr,
    path: Option<String>,
    allowed_methods: AllowedMethods,
}

#[derive(Clone)]
pub struct Router {
    routes: PathTree<Target>,
}

#[derive(Debug, PartialEq)]
pub enum RouterResult {
    Success(Uri),
    NotDefined,
    NotAllowedMethod,
}

impl Router {
    pub fn from_config(config: Config) -> Self {
        let mut routes = PathTree::new();
        for route in config.routes {
            routes.insert(
                &make_path(route.source),
                Target {
                    addr: route.target,
                    path: route.target_path,
                    allowed_methods: parse_allowed_methods(route.allowed_methods),
                },
            );
        }
        Self { routes }
    }

    pub fn eval(&self, req: &Request<Body>) -> RouterResult {
        if let Some(node) = self.routes.find(req.uri().path()) {
            let target = node.0;
            if target.allowed_methods == AllowedMethods::Any
                || target.allowed_methods.contains(&req.method())
            {
                let uri = Uri::builder().scheme(Scheme::HTTP);
                let uri = uri.authority(
                    Authority::from_str(&format!("{}:{}", target.addr.ip(), target.addr.port()))
                        .unwrap(),
                );
                let params = node
                    .1
                    .iter()
                    .map(|p| format!("/{}", p.1))
                    .collect::<String>();
                println!("params: {:?}", params);
                let p_and_q = if let Some(p_and_q) = req.uri().path_and_query() {
                    let path = match &target.path {
                        Some(path) => path,
                        None => "",
                    };
                    if let Some(query) = p_and_q.query() {
                        format!("{}{}?{}", path, params, query)
                    } else {
                        format!("{}{}", path, params)
                    }
                } else {
                    println!("No path and query");
                    String::default()
                };
                println!(
                    "{:?} -> {} | {:?}",
                    req.uri().path_and_query(),
                    p_and_q,
                    node.1
                );
                let p_and_q: &str = &p_and_q;
                let uri = uri.path_and_query(p_and_q).build().unwrap();
                RouterResult::Success(uri)
            } else {
                RouterResult::NotAllowedMethod
            }
        } else {
            RouterResult::NotDefined
        }
    }

    #[cfg(test)]
    pub fn new() -> Self {
        Self {
            routes: PathTree::new(),
        }
    }

    #[cfg(test)]
    pub fn add_route(
        &mut self,
        source: &str,
        addr: SocketAddr,
        allowed_methods: AllowedMethods,
        path: Option<String>,
    ) {
        self.routes.insert(
            &make_path(source.to_owned()),
            Target {
                addr,
                path,
                allowed_methods,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{AllowedMethods, Router, RouterResult};
    use hyper::{Body, Method, Request, Uri};

    fn build_req(uri: &str, method: hyper::Method) -> Request<Body> {
        Request::builder()
            .uri(uri)
            .method(method)
            .body(Body::from("asdf"))
            .unwrap()
    }

    #[test]
    fn route_evaluation() {
        let root = "0.0.0.0:8080".parse().unwrap();
        let home = "0.0.0.0:8000".parse().unwrap();
        let path = "0.0.0.0:7000".parse().unwrap();
        let site = "0.0.0.0:5000".parse().unwrap();
        let bulk = "0.0.0.0:3000".parse().unwrap();
        let mut router = Router::new();
        router.add_route("/", root, AllowedMethods::Any, None);
        router.add_route("/home", home, AllowedMethods::Only(vec![Method::GET]), None);
        router.add_route(
            "/specific",
            path,
            AllowedMethods::Any,
            Some("/foobar".to_owned()),
        );
        router.add_route("/bulk/*any", bulk, AllowedMethods::Any, None);
        router.add_route("/site/:name", site, AllowedMethods::Any, None);

        assert_eq!(
            router.eval(&build_req("/", Method::GET)),
            RouterResult::Success(Uri::from_static("http://0.0.0.0:8080"))
        );
        assert_eq!(
            router.eval(&build_req("/home", Method::GET)),
            RouterResult::Success(Uri::from_static("http://0.0.0.0:8000"))
        );
        assert_eq!(
            router.eval(&build_req("/home?asdf=foobar", Method::GET)),
            RouterResult::Success(Uri::from_static("http://0.0.0.0:8000/?asdf=foobar"))
        );
        assert_eq!(
            router.eval(&build_req("/home/asdf", Method::GET)),
            RouterResult::NotDefined
        );
        assert_eq!(
            router.eval(&build_req("/bulk/qwerty", Method::GET)),
            RouterResult::Success(Uri::from_static("http://0.0.0.0:3000/qwerty"))
        );
        assert_eq!(
            router.eval(&build_req("/bulk/asdf/qwerty", Method::GET)),
            RouterResult::Success(Uri::from_static("http://0.0.0.0:3000/asdf/qwerty"))
        );
        assert_eq!(
            router.eval(&build_req("/home", Method::POST)),
            RouterResult::NotAllowedMethod
        );
        assert_eq!(
            router.eval(&build_req("/specific", Method::GET)),
            RouterResult::Success(Uri::from_static("http://0.0.0.0:7000/foobar"))
        );
        assert_eq!(
            router.eval(&build_req("/notdefined", Method::GET)),
            RouterResult::NotDefined
        );
    }
}
