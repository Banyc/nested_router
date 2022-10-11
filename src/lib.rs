use std::collections::BTreeMap;

use route_recognizer::Router;

pub const SUB_PATH_WILDCARD_NAME: &str = "_sub_path";

#[derive(Debug, PartialEq)]
pub struct Route {
    pub path: String,
    pub next_routes: Option<RouteList>,
}

impl Clone for Route {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            next_routes: match self.next_routes {
                Some(ref next_routes) => Some(next_routes.clone()),
                None => None,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteList {
    pub routes: Vec<Route>,
}

impl RouteList {
    pub fn route(&self, rel_path: &str) -> Result<RouteOutput, Error> {
        if rel_path.starts_with('/') {
            return Err(Error::InvalidPath);
        }
        let router = {
            let mut router = Router::new();

            for (i, route) in self.routes.iter().enumerate() {
                match route.next_routes {
                    Some(_) => {
                        router.add(&format!("{}/*{}", &route.path, SUB_PATH_WILDCARD_NAME), i)
                    }
                    None => router.add(&route.path, i),
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
                let mut sub_path = None;

                for (key, value) in matched.params() {
                    if key == SUB_PATH_WILDCARD_NAME {
                        sub_path = Some(value);
                    } else {
                        params.insert(key.to_string(), value.to_string());
                    }
                }

                // client code should not occupy the wildcard name `SUB_PATH_WILDCARD_NAME`
                assert_eq!(
                    sub_path.is_some(),
                    route.next_routes.is_some(),
                    "Client code should not occupy the wildcard name `\"{}\"`",
                    SUB_PATH_WILDCARD_NAME
                );

                let sub_path = match sub_path {
                    Some(sub_path) => Some(sub_path.to_string()),
                    None => None,
                };

                Ok(RouteOutput {
                    sub_path,
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
    pub sub_path: Option<String>,
    pub route: &'route_list Route,
    pub params: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok() {
        let root = route_list_1();

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

            let sub = route.next_routes.as_ref().unwrap();

            let RouteOutput {
                sub_path,
                route,
                params,
            } = sub.route(&sub_path.unwrap()).unwrap();
            assert_eq!(route.path, ":id");
            assert_eq!(params.get("id"), Some(&"456".to_string()));
            assert_eq!(sub_path, None);
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
            assert_eq!(sub_path, None);
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
            assert_eq!(sub_path, Some("123".to_string()));
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
        assert_eq!(sub_path, None);
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_combined_segment() {
        let root = route_list_4();

        let absolute_path = "/1/2";
        let relative_path = &absolute_path[1..];

        let RouteOutput {
            sub_path,
            route,
            params,
        } = root.route(relative_path).unwrap();
        assert_eq!(route.path, "1/2");
        assert_eq!(params.len(), 0);
        assert_eq!(sub_path, None);
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
    #[should_panic]
    fn test_bad_client_code() {
        let root = route_list_bad();

        let absolute_path = "/123";
        let relative_path = &absolute_path[1..];

        let _ = root.route(relative_path);
    }

    fn route_list_1() -> RouteList {
        let sub = RouteList {
            routes: vec![Route {
                path: ":id".to_string(),
                next_routes: None,
            }],
        };
        let root = RouteList {
            routes: vec![
                Route {
                    path: ":id".to_string(),
                    next_routes: Some(sub.clone()),
                },
                Route {
                    path: "about".to_string(),
                    next_routes: None,
                },
            ],
        };
        root
    }

    fn route_list_2() -> RouteList {
        let root = RouteList {
            routes: vec![Route {
                path: "about".to_string(),
                next_routes: None,
            }],
        };
        root
    }

    fn route_list_3() -> RouteList {
        let root = RouteList {
            routes: vec![Route {
                path: "".to_string(),
                next_routes: None,
            }],
        };
        root
    }

    fn route_list_4() -> RouteList {
        let root = RouteList {
            routes: vec![Route {
                path: "1/2".to_string(),
                next_routes: None,
            }],
        };
        root
    }

    fn route_list_bad() -> RouteList {
        let root = RouteList {
            routes: vec![Route {
                path: format!(":{}", SUB_PATH_WILDCARD_NAME),
                next_routes: None,
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
