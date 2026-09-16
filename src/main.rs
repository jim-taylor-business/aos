#![recursion_limit = "512"]
#![allow(warnings)]

#[cfg(feature = "ssr")]
// #[tokio::main]
// async
fn main() {
  use axum::{body::{Body, to_bytes}, extract::State, response::IntoResponse};
  use http::Request;
  use leptos::config::LeptosOptions;
  use leptos::context::provide_context;
  use leptos_axum::render_app_to_stream_with_context;
  use serde::Deserialize;
  use aos::{PassedUrl, App, html_template};
  use leptos::prelude::*;

  let stack_size_bytes = 8 * 1024 * 1024;
  let runtime = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(4)
    .thread_stack_size(stack_size_bytes)
    .enable_all()
    .build()
    .expect("failed to build Tokio runtime with custom stack size");

  #[derive(Deserialize, Default)]
  struct SourceUrlForm {
      source_url: String,
  }

  async fn destination_post_handler(
      State(leptos_options): State<LeptosOptions>,
      req: Request<Body>,
  ) -> impl IntoResponse {
      let (parts, body) = req.into_parts();
      let bytes = to_bytes(body, usize::MAX).await.unwrap_or_default();
      let form: SourceUrlForm = serde_urlencoded::from_bytes(&bytes).unwrap_or_default();
      let req = Request::from_parts(parts, Body::empty());
      let passed_url = form.source_url;
      let handler = render_app_to_stream_with_context(
          move || provide_context(PassedUrl(Some(passed_url.clone()))),
          move || html_template(leptos_options.clone()),
      );
      handler(/* State(leptos_options),  */req).await.into_response()
  }

  runtime.block_on(async {
    use aos::{App, html_template};
    use axum::Router;
    use axum::routing::post;
    use axum::routing::get;
    use leptos::{config::get_configuration, logging::log};
    use leptos_axum::{LeptosRoutes, generate_route_list};

    if let Ok(conf) = get_configuration(None) {
      let leptos_options = conf.leptos_options;
      let bind_address = leptos_options.site_addr;
      let app_routes = generate_route_list(App);

      let service_router = Router::new()
        .route("/l", post(destination_post_handler))
        .leptos_routes(&leptos_options, app_routes, {
          let leptos_options = leptos_options.clone();
          move || html_template(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(html_template))
        .with_state(leptos_options);

      if let Ok(listener) = tokio::net::TcpListener::bind(&bind_address).await {
        if let Ok(_a) = axum::serve(listener, service_router.into_make_service()).await {
        } else {
          log!("server did not start");
        }
      } else {
        log!("listener did not bind");
      }
    } else {
      log!("configuration not found");
    }
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
