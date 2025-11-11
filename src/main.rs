#![recursion_limit = "512"]
#![allow(warnings)]

#[cfg(feature = "ssr")]
// #[tokio::main]
// async
fn main() {
  let stack_size_bytes = 8 * 1024 * 1024;
  let runtime = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(4)
    .thread_stack_size(stack_size_bytes)
    .enable_all()
    .build()
    .expect("failed to build Tokio runtime with custom stack size");

  runtime.block_on(async {
    use aos::{html_template, App};
    use axum::Router;
    use leptos::config::get_configuration;
    use leptos_axum::{generate_route_list, LeptosRoutes};

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app = Router::new()
      .leptos_routes(&leptos_options, routes, {
        let leptos_options = leptos_options.clone();
        move || html_template(leptos_options.clone())
      })
      .fallback(leptos_axum::file_and_error_handler(html_template))
      .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
  });
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
  use aos::*;
  use leptos::mount::mount_to_body;

  // _ = init_with_level(log::Level::Debug);
  console_error_panic_hook::set_once();
  mount_to_body(App);
}
