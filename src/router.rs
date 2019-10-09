use failure::Error;
use path_tree::PathTree;
use std::net::SocketAddr;
use std::path::Path;

fn make_path(path: String) -> String {
    let mut path = path.replace("//", "/");
    if !path.starts_with("/") {
        path = format!("/{}", path);
    }
    if path.ends_with("/") {
        path.remove(path.len() - 1);
    }
    path
}

pub struct Target {
    addr: SocketAddr,
    path: Option<String>,
}

pub struct Router {
    routes: PathTree<Target>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: PathTree::new(),
        }
    }

    pub fn eval(&self, path: &str) -> Option<String> {
        if let Some(node) = self.routes.find(path) {
            let mut route = String::new();
            println!("Params: {:?}", node.1);
            let target = node.0;
            route = format!("http://{}:{}", node.0.addr.ip(), target.addr.port());
            if let Some(path) = &node.0.path {
                route += &path;
            }
            if node.1.len() > 0 {
                for (_, v) in node.1 {
                    route += &format!("/{}", v);
                }
            }

            println!("Route: {}", route);
            Some(route)
        } else {
            None
        }
    }

    pub fn add_route(&mut self, source: &str, addr: SocketAddr) {
        self.routes.insert(
            &make_path(source.to_owned()),
            Target {
                addr: addr,
                path: None,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::Router;
    use std::net::SocketAddr;

    fn fmt_addr(addr: SocketAddr, path: &str) -> Option<String> {
        Some(format!("http://{}:{}{}", addr.ip(), addr.port(), path))
    }

    #[test]
    fn evaluation() {
        let root = "0.0.0.0:8080".parse().unwrap();
        let home = "0.0.0.0:8000".parse().unwrap();
        let site = "127.0.0.1:5000".parse().unwrap();
        let bulk = "127.0.0.1:3000".parse().unwrap();
        let multi = "127.0.0.1:2000".parse().unwrap();
        let mut router = Router::new();
        router.add_route("/", root);
        router.add_route("/home", home);
        router.add_route("/home/*any", home);
        router.add_route("/site/:name", site);
        router.add_route("/bulk/*any", bulk);
        router.add_route("/multi/:name/res/:res", multi);

        assert_eq!(router.eval("/"), fmt_addr(root, ""));
        assert_eq!(router.eval("/home"), fmt_addr(home, ""));
        assert_eq!(router.eval("/home/stuff"), fmt_addr(home, "/stuff"));
        assert_eq!(router.eval("/foo"), None);
        assert_eq!(router.eval("/site"), None);
        assert_eq!(router.eval("/site/test"), fmt_addr(site, "/test"));
        assert_eq!(router.eval("/site/test/bar/foo"), None);
        assert_eq!(router.eval("/bulk"), None);
        assert_eq!(router.eval("/bulk/test"), fmt_addr(bulk, "/test"));
        assert_eq!(
            router.eval("/bulk/test/bar/foo"),
            fmt_addr(bulk, "/test/bar/foo")
        );
        assert_eq!(
            router.eval("/multi/calvin/res/css"),
            fmt_addr(multi, "/calvin/css")
        );
    }
}
