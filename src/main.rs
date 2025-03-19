#![allow(warnings)]
// #![recursion_limit = "10000"]

#[cfg(feature = "ssr")]
mod ssr_imports {
  pub use actix_files::Files;
  pub use actix_web::*;
  pub use aos::App;

  #[get("/pkg/aos.css")]
  pub async fn css() -> impl Responder {
    actix_files::NamedFile::open_async("./target/site/pkg/aos.css").await
  }
  #[get("/pkg/aos.js")]
  pub async fn js() -> impl Responder {
    actix_files::NamedFile::open_async("./target/site/pkg/aos.js").await
  }
  #[get("/pkg/aos.wasm")]
  pub async fn wasm() -> impl Responder {
    actix_files::NamedFile::open_async("./target/site/pkg/aos.wasm").await
  }
  #[get("/favicon.ico")]
  pub async fn favicon() -> impl Responder {
    actix_files::NamedFile::open_async("./target/site/favicon.ico").await
  }
  #[get("/favicon.png")]
  pub async fn favicon_large() -> impl Responder {
    actix_files::NamedFile::open_async("./target/site/favicon.png").await
  }
  #[get("/manifest.json")]
  pub async fn manifest() -> impl Responder {
    actix_files::NamedFile::open_async("./target/site/manifest.json").await
  }
  #[get("/service-worker.js")]
  pub async fn worker() -> impl Responder {
    actix_files::NamedFile::open_async("./target/site/service-worker.js").await
  }
  #[get("/icons.svg")]
  pub async fn icons() -> impl Responder {
    actix_files::NamedFile::open_async("./target/site/icons.svg").await
  }
  #[get("/lemmy.svg")]
  pub async fn lemmy() -> impl Responder {
    actix_files::NamedFile::open_async("./target/site/lemmy.svg").await
  }
}

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
  use actix_files::Files;
  use actix_web::*;
  // use leptos::prelude::*;
  use leptos_actix::{generate_route_list, handle_server_fns, LeptosRoutes};
  use leptos_meta::{Link, MetaTags, Stylesheet};
  // use awc::Client;
  use leptos::prelude::*;
  // use ssr_services::*;
  use ssr_imports::*;

  let conf = get_configuration(None).unwrap();
  let addr = conf.leptos_options.site_addr;

  HttpServer::new(move || {
    let routes = generate_route_list(aos::App);
    let leptos_options = conf.leptos_options.clone();
    let site_root = conf.leptos_options.site_root.clone();

    App::new()
      .service(favicon)
      .service(favicon_large)
      .service(manifest)
      .service(worker)
      .service(icons)
      .service(lemmy)
      .service(js)
      .service(wasm)
      .service(css)
      .route("/serverfn/{tail:.*}", handle_server_fns())
      // .leptos_routes(routes, move || aos::App)
      .leptos_routes(routes, {
        // let leptos_options = leptos_options.clone();
        move || {
          use leptos::prelude::*;
          view! {
              <!DOCTYPE html>
              <html>
                  <head>
                      <meta charset="utf-8"/>
                      <meta name="viewport" content="width=device-width, initial-scale=1"/>
                      <Stylesheet id="leptos" href="/pkg/aos.css" />
                      <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico" />
                      <Link rel="manifest" href="/manifest.json" />
                      <AutoReload options=leptos_options.clone() />
                      <HydrationScripts options=leptos_options.clone() />
                      <MetaTags />
                  </head>
                  <body>
                      <aos::App/>
                  </body>
              </html>
          }
          .into_any()
        }
      })
    // .service(Files::new("/", site_root.as_ref()))
    //.wrap(middleware::Compress::default())
  })
  .bind(&addr)?
  .run()
  .await
}

// // CSR-only setup
// #[cfg(not(feature = "ssr"))]
// fn main() {
//   use aos::App;
//   console_error_panic_hook::set_once();
//   leptos::mount::mount_to_body(App)
// }

// // use aos::*;

// #[cfg(feature = "ssr")]
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//   use actix_files::Files;
//   use actix_web::*;
//   // use awc::Client;
//   use leptos::prelude::*;
//   use leptos_actix::{generate_route_list, LeptosRoutes};
//   use leptos_meta::{Link, Stylesheet};
//   use ssr_services::*;

//   let conf = get_configuration(None).unwrap();
//   let addr = conf.leptos_options.site_addr;

//   HttpServer::new(move || {
//     let leptos_options = &conf.leptos_options;
//     let site_root = &leptos_options.site_root;
//     let routes = generate_route_list(aos::App);

//     // let client = web::Data::new(Client::new());

//     App::new()
//       // .route("/serverfn/{tail:.*}", leptos_actix::handle_server_fns())
//       // .service(Files::new("/assets", site_root.as_ref()))
//       // .service(favicon)
//       // .service(icons)
//       // .service(lemmy)
//       .leptos_routes(routes, {
//         let leptos_options = leptos_options.clone();
//         move || {
//           use leptos::prelude::*;

//           view! {
//               <!DOCTYPE html>
//               <html lang="en">
//                   <head>
//                       // <meta charset="utf-8"/>
//                       // <meta name="viewport" content="width=device-width, initial-scale=1"/>
//                       // <Stylesheet id="leptos" href="/pkg/aos.css" />
//                       // <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico" />
//                       // <AutoReload options=leptos_options.clone() />
//                       <HydrationScripts options=leptos_options.clone()/>
//                       // <MetaTags/>
//                   </head>
//                   <body>
//                       <aos::App/>
//                   </body>
//               </html>
//           }
//         }
//       })
//       // .service(Files::new("/pkg", format!("{}/pkg", site_root.as_ref())))
//       // .leptos_routes(/*leptos_options.to_owned(), */ routes, move || aos::App)
//       // .app_data(web::Data::new(leptos_options.to_owned()))
//       // .app_data(client)
//       // App::new()
//       //   // .service(css)
//       // .service(favicon)
//       // .leptos_routes(routes, {
//       //   let leptos_options = leptos_options.clone();
//       //   move || {
//       //     use leptos::prelude::*;
//       //     view! {
//       //         // <!DOCTYPE html>
//       //         // <html lang="en">
//       //         //     <head>
//       //         // //         <meta charset="utf-8"/>
//       //         // //         <meta name="viewport" content="width=device-width, initial-scale=1"/>
//       //         // //         <AutoReload options=leptos_options.clone() />
//       //         // //         <HydrationScripts options=leptos_options.clone()/>
//       //         //     </head>
//       //         //     <body>
//       //                 <aos::App/>
//       //         //     </body>
//       //         // </html>
//       //     }
//       //   }
//       // })
//       .service(Files::new("/", site_root.as_ref()))
//   })
//   .bind(&addr)?
//   .run()
//   .await
// }

#[cfg(feature = "csr")]
pub fn main() {
  use wasm_bindgen::prelude::wasm_bindgen;
  console_error_panic_hook::set_once();
  leptos::prelude::mount_to_body(aos::App);
}

#[cfg(feature = "ssr")]
mod ssr_services {
  use actix_files::*;
  use actix_web::*;
  // pub use actix_files::Files;
  // pub use actix_web::*;
  // pub use aos::App;

  // use awc::Client;
  use leptos::{logging, prelude::*};
  // use leptos_actix::{generate_route_list, LeptosRoutes};

  #[get("/lemmy.svg")]
  async fn lemmy(leptos_options: web::Data<LeptosOptions>) -> actix_web::Result<NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!("{site_root}/lemmy.svg"))?)
  }

  #[get("/favicon.ico")]
  async fn favicon(leptos_options: web::Data<LeptosOptions>) -> actix_web::Result<NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    logging::log!("{}", site_root);
    Ok(actix_files::NamedFile::open(format!("{site_root}/favicon.ico"))?)
  }

  #[get("/icons.svg")]
  async fn icons(leptos_options: web::Data<LeptosOptions>) -> actix_web::Result<NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!("{site_root}/icons.svg"))?)
  }
}
