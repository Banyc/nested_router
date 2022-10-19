use std::collections::BTreeMap;

use route_recognizer::Router;

pub const SUB_PATH_WILDCARD_NAME: &str = "_sub_path";

#[derive(Debug, PartialEq, Clone)]
pub struct Route {
    pub path: String,
    pub has_sub_routes: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteList {
    pub routes: Vec<Route>,
}

impl RouteList {
    pub fn route(&self, mut rel_path: &str) -> Result<RouteOutput, Error> {
        if rel_path.starts_with('/') {
            return Err(Error::InvalidPath);
        }
        if rel_path.ends_with('/') {
            rel_path = &rel_path[..rel_path.len() - 1];
        }
        let router = {
            let mut router = Router::new();

            for (i, route) in self.routes.iter().enumerate() {
                match route.has_sub_routes {
                    true => router.add(&format!("{}/*{}", &route.path, SUB_PATH_WILDCARD_NAME), i),
                    false => router.add(&route.path, i),
                }
                router.add(&route.path, i);
            }
            router
        };
        let matched = router.recognize(rel_path);
        match matched {
            Ok(matched) => {
                let idx = **matched.handler();
                let route = &self.routes[idx];
                let mut params = BTreeMap::new();
                let mut sub_path = "";

                for (key, value) in matched.params() {
                    if key == SUB_PATH_WILDCARD_NAME {
                        sub_path = value;
                    } else {
                        params.insert(key.to_string(), value.to_string());
                    }
                }

                Ok(RouteOutput {
                    sub_path: sub_path.to_string(),
                    route,
                    params,
                })
            }
            Err(_) => Err(Error::NotFound),
        }
    }
}

#[derive(Debug)]
pub struct RouteOutput<'route_list> {
    pub sub_path: String,
    pub route: &'route_list Route,
    pub params: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok() {
        let (root, sub) = route_list_1();

        {
            let absolute_path = "/123/456";
            let relative_path = &absolute_path[1..];

            let RouteOutput {
                sub_path,
                route,
                params,
            } = root.route(relative_path).unwrap();
            assert_eq!(route.path, ":id");
            assert_eq!(params.get("id"), Some(&"123".to_string()));

            let RouteOutput {
                sub_path,
                route,
                params,
            } = sub.route(&sub_path).unwrap();
            assert_eq!(route.path, ":id");
            assert_eq!(params.get("id"), Some(&"456".to_string()));
            assert_eq!(sub_path, "");
        }

        {
            let absolute_path = "/about";
            let relative_path = &absolute_path[1..];

            let RouteOutput {
                sub_path,
                route,
                params,
            } = root.route(relative_path).unwrap();
            assert_eq!(route.path, "about");
            assert_eq!(sub_path, "");
            assert_eq!(params.len(), 0);
        }

        {
            let absolute_path = "/about/123";
            let relative_path = &absolute_path[1..];

            let RouteOutput {
                sub_path,
                route,
                params,
            } = root.route(relative_path).unwrap();
            assert_eq!(route.path, ":id");
            assert_eq!(params.get("id"), Some(&"about".to_string()));
            assert_eq!(sub_path, "123".to_string());
        }
    }

    #[test]
    fn test_index() {
        let root = route_list_3();

        let absolute_path = "/";
        let relative_path = &absolute_path[1..];

        let RouteOutput {
            sub_path,
            route,
            params,
        } = root.route(relative_path).unwrap();
        assert_eq!(route.path, "");
        assert_eq!(sub_path, "");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_combined_segment() {
        let root = RouteList {
            routes: vec![Route {
                path: "1/2".to_string(),
                has_sub_routes: false,
            }],
        };

        let absolute_path = "/1/2";
        let relative_path = &absolute_path[1..];

        let RouteOutput {
            sub_path,
            route,
            params,
        } = root.route(relative_path).unwrap();
        assert_eq!(route.path, "1/2");
        assert_eq!(params.len(), 0);
        assert_eq!(sub_path, "");
    }

    #[test]
    fn test_not_found() {
        let root = route_list_2();

        {
            let absolute_path = "/about/123";
            let relative_path = &absolute_path[1..];

            assert_eq!(root.route(relative_path).unwrap_err(), Error::NotFound);
        }

        {
            let absolute_path = "/about2";
            let relative_path = &absolute_path[1..];

            assert_eq!(root.route(relative_path).unwrap_err(), Error::NotFound);
        }
    }

    #[test]
    fn test_invalid_path() {
        let root = route_list_2();

        let absolute_path = "/about/123";

        assert_eq!(root.route(absolute_path).unwrap_err(), Error::InvalidPath);
    }

    #[test]
    fn test_sub_index() {
        let root = RouteList {
            routes: vec![
                Route {
                    path: "sub".to_string(),
                    has_sub_routes: true,
                },
                Route {
                    path: "*".to_string(),
                    has_sub_routes: false,
                },
            ],
        };

        {
            let absolute_path = "/sub";
            let relative_path = &absolute_path[1..];

            let RouteOutput {
                sub_path,
                route,
                params,
            } = root.route(relative_path).unwrap();
            assert_eq!(sub_path, "");
            assert_eq!(route.path, "sub");
            assert_eq!(params.len(), 0);
        }

        {
            let absolute_path = "/sub/";
            let relative_path = &absolute_path[1..];

            let RouteOutput {
                sub_path,
                route,
                params,
            } = root.route(relative_path).unwrap();
            assert_eq!(sub_path, "");
            assert_eq!(route.path, "sub");
            assert_eq!(params.len(), 0);
        }
    }

    fn route_list_1() -> (RouteList, RouteList) {
        let sub = RouteList {
            routes: vec![Route {
                path: ":id".to_string(),
                has_sub_routes: false,
            }],
        };
        let root = RouteList {
            routes: vec![
                Route {
                    path: ":id".to_string(),
                    has_sub_routes: true,
                },
                Route {
                    path: "about".to_string(),
                    has_sub_routes: false,
                },
            ],
        };
        (root, sub)
    }

    fn route_list_2() -> RouteList {
        let root = RouteList {
            routes: vec![Route {
                path: "about".to_string(),
                has_sub_routes: false,
            }],
        };
        root
    }

    fn route_list_3() -> RouteList {
        let root = RouteList {
            routes: vec![Route {
                path: "".to_string(),
                has_sub_routes: false,
            }],
        };
        root
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidPath,
    NotFound,
}
