# Nested Router

Path segment matching for nested routers.

## How to use

1. Define your routes:

   ```rust
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
   ```

1. Use the two routers:

   ```rust
   let root = route_list_1();

   let absolute_path = "/123/456";
   let relative_path = &absolute_path[1..];

   let RouteOutput {
       sub_path,
       route,
       params,
   } = root.route(relative_path).unwrap();
   assert_eq!(route.path, ":id");
   assert_eq!(params.get("id"), Some(&"123".to_string()));

   // Your business logic here for the first route

   let sub = route.next_routes.as_ref().unwrap();

   let RouteOutput {
       sub_path,
       route,
       params,
   } = sub.route(&sub_path.unwrap()).unwrap();
   assert_eq!(route.path, ":id");
   assert_eq!(params.get("id"), Some(&"456".to_string()));
   assert_eq!(sub_path, None);

   // Your business logic here for the second route
   ```

## Restrictions

- `Route::path` should not capture wildcard with name `"_sub_path"` (`crate::SUB_PATH_WILDCARD_NAME`).
