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

#[derive(Debug)]
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
                    RouterResult::Success(format!("{}/{}", route, ext.join("/")))
                }
            } else {
                RouterResult::NotAllowedMethod
            }
        } else {
            RouterResult::NotDefined
        }
    }

    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            routes: PathTree::new(),
        }
    }

    #[allow(dead_code)]
    pub fn add_route(&mut self, source: &str, addr: SocketAddr) {
        self.routes.insert(
            &make_path(source.to_owned()),
            Target {
                addr,
                path: None,
                allowed_methods: AllowedMethods::Any,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::Router;
    use std::net::SocketAddr;
    // #[test]
    // fn evaluation() {
    //     let root = "0.0.0.0:8080".parse().unwrap();
    //     let home = "0.0.0.0:8000".parse().unwrap();
    //     let site = "127.0.0.1:5000".parse().unwrap();
    //     let bulk = "127.0.0.1:3000".parse().unwrap();
    //     let multi = "127.0.0.1:2000".parse().unwrap();
    //     let mut router = Router::new();
    //     router.add_route("/", root);
    //     router.add_route("/home", home);
    //     router.add_route("/home/*any", home);
    //     router.add_route("/site/:name", site);
    //     router.add_route("/bulk/*any", bulk);
    //     router.add_route("/multi/:name/res/:res", multi);

    //     assert_eq!(router.eval("/"), fmt_addr(root, ""));
    //     assert_eq!(router.eval("/home"), fmt_addr(home, ""));
    //     assert_eq!(router.eval("/home/stuff"), fmt_addr(home, "/stuff"));
    //     assert_eq!(router.eval("/foo"), None);
    //     assert_eq!(router.eval("/site"), None);
    //     assert_eq!(router.eval("/site/test"), fmt_addr(site, "/test"));
    //     assert_eq!(router.eval("/site/test/bar/foo"), None);
    //     assert_eq!(router.eval("/bulk"), None);
    //     assert_eq!(router.eval("/bulk/test"), fmt_addr(bulk, "/test"));
    //     assert_eq!(
    //         router.eval("/bulk/test/bar/foo"),
    //         fmt_addr(bulk, "/test/bar/foo")
    //     );
    //     assert_eq!(
    //         router.eval("/multi/calvin/res/css"),
    //         fmt_addr(multi, "/calvin/css")
    //     );
    // }
}
