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

#[cfg(test)]
mod tests {
    use super::parse_allowed_methods;
    use super::AllowedMethods;
    use hyper::Method;
    #[test]
    fn valid_acl() {
        let allowed = parse_allowed_methods(vec!["GET".to_owned()]);
        assert_eq!(allowed.contains(&Method::GET), true);
        assert_eq!(allowed.contains(&Method::PATCH), false);

        assert_eq!(parse_allowed_methods(vec![]), AllowedMethods::Any);
        let all_methods = vec![
            Method::OPTIONS,
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::HEAD,
            Method::TRACE,
            Method::CONNECT,
            Method::PATCH,
        ];
        let methods = parse_allowed_methods(vec![
            "Options".to_owned(),
            "GET".to_owned(),
            "POST".to_owned(),
            "pUT".to_owned(),
            "delete".to_owned(),
            "Head".to_owned(),
            "trace".to_owned(),
            "Connect".to_owned(),
            "patch".to_owned(),
        ]);
        assert_eq!(AllowedMethods::Only(all_methods), methods);
    }
}
