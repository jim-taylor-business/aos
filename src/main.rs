#![allow(warnings)]

use aos::*;
use leptos::*;

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
  use actix_files::Files;
  use actix_web::*;
  use awc::Client;
  use leptos_actix::{generate_route_list, LeptosRoutes};
  use ssr_services::*;

  let conf = get_configuration(None).await.unwrap();
  let addr = conf.leptos_options.site_addr;
  let routes = generate_route_list(App);

  HttpServer::new(move || {
    let leptos_options = &conf.leptos_options;
    let site_root = &leptos_options.site_root;
    let routes = &routes;
    let client = web::Data::new(Client::new());

    App::new()
      .route("/serverfn/{tail:.*}", leptos_actix::handle_server_fns())
      .service(Files::new("/pkg", format!("{site_root}/pkg")))
      // .service(Files::new("/assets", site_root))
      .service(favicon)
      .service(large_favicon)
      .service(icons)
      .service(lemmy)
      .service(manifest)
      .service(service_worker)
      .service(font_regular)
      .service(font_italic)
      .leptos_routes(leptos_options.to_owned(), routes.to_owned(), App)
      .app_data(web::Data::new(leptos_options.to_owned()))
      .app_data(client)
  })
  .bind(&addr)?
  .run()
  .await
}

#[cfg(feature = "ssr")]
mod ssr_services {
  use actix_files::Files;
  use actix_web::*;

  #[actix_web::get("lemmy.svg")]
  async fn lemmy(leptos_options: web::Data<leptos::LeptosOptions>) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!("{site_root}/lemmy.svg"))?)
  }

  #[actix_web::get("favicon.ico")]
  async fn favicon(leptos_options: web::Data<leptos::LeptosOptions>) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!("{site_root}/favicon.ico"))?)
  }

  #[actix_web::get("favicon.png")]
  async fn large_favicon(leptos_options: web::Data<leptos::LeptosOptions>) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!("{site_root}/favicon.png"))?)
  }

  #[actix_web::get("icons.svg")]
  async fn icons(leptos_options: web::Data<leptos::LeptosOptions>) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!("{site_root}/icons.svg"))?)
  }

  #[actix_web::get("manifest.json")]
  async fn manifest(leptos_options: web::Data<leptos::LeptosOptions>) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    // let mut f = actix_files::NamedFile::open(format!("{site_root}/manifest.json"))?.set_content_type(mime::Mime::from_str("application/manifest+json").unwrap());
    // Ok(f)
    Ok(actix_files::NamedFile::open(format!("{site_root}/manifest.json"))?)
  }

  #[actix_web::get("service-worker.js")]
  async fn service_worker(leptos_options: web::Data<leptos::LeptosOptions>) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!("{site_root}/service-worker.js"))?)
  }

  #[actix_web::get("AdwaitaSans-Regular.ttf")]
  async fn font_regular(leptos_options: web::Data<leptos::LeptosOptions>) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!("{site_root}/AdwaitaSans-Regular.ttf"))?)
  }

  #[actix_web::get("AdwaitaSans-Italic.ttf")]
  async fn font_italic(leptos_options: web::Data<leptos::LeptosOptions>) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!("{site_root}/AdwaitaSans-Italic.ttf"))?)
  }
}

#[cfg(not(any(feature = "ssr", feature = "csr")))]
pub fn main() {
  // for pure client-side testing
  // see lib.rs for hydration function
  // a client-side main function is required for using `trunk serve`
  // to run: `trunk serve --open --features hydrate`
}

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
pub fn main() {
  // a client-side main function is required for using `trunk serve`
  // to run: `trunk serve --open --features csr`
  console_error_panic_hook::set_once();
  leptos::mount_to_body(App);
}
