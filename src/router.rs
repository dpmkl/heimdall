use crate::acl::{parse_allowed_methods, AllowedMethods};
use crate::config::Config;
use hyper::{Body, Request};
use path_tree::PathTree;
use std::net::SocketAddr;

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
    Success(String),
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
            if node.0.allowed_methods == AllowedMethods::Any
                || node.0.allowed_methods.contains(req.method())
            {
                let target = node.0;
                let route = if let Some(path) = &node.0.path {
                    format!("http://{}:{}{}", node.0.addr.ip(), target.addr.port(), path)
                } else {
                    format!("http://{}:{}", node.0.addr.ip(), target.addr.port())
                };
                if node.1.is_empty() {
                    RouterResult::Success(route)
                } else {
                    let ext: Vec<String> =
                        node.1.into_iter().map(|(_, v)| format!("/{}", v)).collect();
                    println!("{:?}", ext);
                    RouterResult::Success(format!("{}{}", route, ext.join("/")))
                }
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
    pub fn add_route(&mut self, source: &str, addr: SocketAddr, allowed_methods: AllowedMethods) {
        self.routes.insert(
            &make_path(source.to_owned()),
            Target {
                addr,
                path: None,
                allowed_methods,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::AllowedMethods;
    use super::{Router, RouterResult};
    use hyper::{Body, Method, Request};

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
        let site = "127.0.0.1:5000".parse().unwrap();
        let bulk = "127.0.0.1:3000".parse().unwrap();
        let multi = "127.0.0.1:2000".parse().unwrap();
        let mut router = Router::new();
        router.add_route("/", root, AllowedMethods::Any);
        router.add_route("/home", home, AllowedMethods::Only(vec![Method::GET]));
        router.add_route("/home/*any", home, AllowedMethods::Any);
        router.add_route("/site/:name", site, AllowedMethods::Any);
        router.add_route("/bulk/*any", bulk, AllowedMethods::Any);
        router.add_route("/multi/:name/res/:res", multi, AllowedMethods::Any);

        assert_eq!(
            router.eval(&build_req("/home", Method::GET)),
            RouterResult::Success("http://0.0.0.0:8000".to_owned())
        );
        assert_eq!(
            router.eval(&build_req("/home/qwerty", Method::GET)),
            RouterResult::Success("http://0.0.0.0:8000/qwerty".to_owned())
        );
        assert_eq!(
            router.eval(&build_req("/home/asdf/qwerty", Method::GET)),
            RouterResult::Success("http://0.0.0.0:8000/asdf/qwerty".to_owned())
        );
        assert_eq!(
            router.eval(&build_req("/home", Method::POST)),
            RouterResult::NotAllowedMethod
        );
        assert_eq!(
            router.eval(&build_req("/notdefined", Method::GET)),
            RouterResult::NotDefined
        );
    }
}
