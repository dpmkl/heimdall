#[derive(Clone, Debug, PartialEq)]
pub enum AllowedMethods {
    Any,
    Only(Vec<hyper::Method>),
}

impl AllowedMethods {
    pub fn contains(&self, method: &hyper::Method) -> bool {
        match self {
            AllowedMethods::Any => true,
            AllowedMethods::Only(methods) => methods.contains(method),
        }
    }
}

pub fn parse_allowed_methods(allowed_methods: Vec<String>) -> AllowedMethods {
    if allowed_methods.is_empty() {
        AllowedMethods::Any
    } else {
        AllowedMethods::Only(
            allowed_methods
                .into_iter()
                .map(|s| match s.to_lowercase().as_str() {
                    "options" => hyper::Method::OPTIONS,
                    "get" => hyper::Method::GET,
                    "post" => hyper::Method::POST,
                    "put" => hyper::Method::PUT,
                    "delete" => hyper::Method::DELETE,
                    "head" => hyper::Method::HEAD,
                    "trace" => hyper::Method::TRACE,
                    "connect" => hyper::Method::CONNECT,
                    "patch" => hyper::Method::PATCH,
                    _ => panic!("Invalid http method '{}' for route!", s),
                })
                .collect(),
        )
    }
}
