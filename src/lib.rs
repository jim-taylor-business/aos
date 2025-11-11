#![recursion_limit = "512"]
#![allow(warnings)]

pub mod client;
pub mod comment;
pub mod comments;
pub mod db;
pub mod errors;
pub mod home;
pub mod icon;
pub mod listing;
pub mod listings;
pub mod login;
pub mod nav;
pub mod post;
pub mod root;
pub mod search;
pub mod toolbar;

use crate::{
  client::{LemmyApi, LemmyClient},
  errors::{LemmyAppError, LemmyAppResult},
  login::Login,
  post::Post,
  search::Search,
};
use codee::string::FromToStringCodec;
use home::Home;
use lemmy_api_common::{
  post::{GetPosts, GetPostsResponse},
  site::GetSiteResponse,
};
use leptos::{html::Div, prelude::*};
use leptos_meta::{provide_meta_context, Link, Meta, MetaTags, Stylesheet, *};
use leptos_router::{
  components::{ParentRoute, Route, Router, Routes},
  StaticSegment, *,
};
use leptos_use::{use_cookie_with_options, use_service_worker_with_options, SameSite, UseCookieOptions, UseServiceWorkerOptions};
#[cfg(not(feature = "ssr"))]
use leptos_use::{use_document_visibility, UseServiceWorkerReturn};
use root::Root;
use std::collections::BTreeMap;

// leptos_i18n::load_locales!();

#[derive(Clone)]
pub struct OnlineSetter(bool);
#[derive(Clone, Debug, PartialEq)]
pub struct NotificationsRefresh(bool);
#[derive(Clone, PartialEq)]
pub struct ResponseLoad(bool);

#[derive(Clone)]
pub struct ReadAuthCookie(Signal<Option<String>>);
#[derive(Clone)]
pub struct WriteAuthCookie(WriteSignal<Option<String>>);
#[derive(Clone)]
pub struct ReadInstanceCookie(Signal<Option<String>>);
#[derive(Clone)]
pub struct WriteInstanceCookie(WriteSignal<Option<String>>);
#[derive(Clone)]
pub struct ReadThemeCookie(Signal<Option<String>>);
#[derive(Clone)]
pub struct WriteThemeCookie(WriteSignal<Option<String>>);

pub fn html_template(options: LeptosOptions) -> impl IntoView {
  view! {
    <!DOCTYPE html>
    <html lang="en">
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <AutoReload options={options.clone()} />
        <HydrationScripts options />
        <MetaTags />
      </head>
      <body>
        <App />
      </body>
    </html>
  }
}

#[component]
fn NotFound() -> impl IntoView {
  #[cfg(feature = "ssr")]
  {
    let resp = expect_context::<leptos_axum::ResponseOptions>();
    resp.set_status(http::StatusCode::NOT_FOUND);
  }
  let ReadThemeCookie(get_theme_cookie) = expect_context::<ReadThemeCookie>();
  view! {
    <div class="flex flex-col min-h-screen" data-theme={move || get_theme_cookie.get()}>
      <div class="py-4 px-8 break-inside-avoid">
        <div class="flex justify-between alert alert-warning alert-soft">
          <span class="text-lg">"Not Found"</span>
        // <span on:click={on_retry_click} class="btn btn-sm">
        // "Retry"
        // </span>
        </div>
      </div>
    </div>
  }
}

#[component]
pub fn App() -> impl IntoView {
  provide_meta_context();
  let online = RwSignal::new(OnlineSetter(true));
  provide_context(online);
  let notifications_refresh = RwSignal::new(NotificationsRefresh(true));
  provide_context(notifications_refresh);

  #[cfg(not(feature = "ssr"))]
  let UseServiceWorkerReturn { .. } = use_service_worker_with_options(
    UseServiceWorkerOptions::default()
      .script_url("/service-worker.js")
      .skip_waiting_message("skipWaiting"),
  );
  #[cfg(not(feature = "ssr"))]
  let visibility = use_document_visibility();
  #[cfg(not(feature = "ssr"))]
  provide_context(visibility);

  let on_online = move |b: bool| {
    move |_| {
      online.set(OnlineSetter(b));
    }
  };
  let _offline_handle = window_event_listener_untyped("offline", on_online(false));
  let _online_handle = window_event_listener_untyped("online", on_online(true));

  let response_cache: RwSignal<BTreeMap<(usize, GetPosts, Option<bool>), (i64, LemmyAppResult<GetPostsResponse>)>> = RwSignal::new(BTreeMap::new());
  provide_context(response_cache);
  // let search_cache: RwSignal<BTreeMap<(usize, String, ListingType, SortType, String), Option<GetPostsResponse>>> = RwSignal::new(BTreeMap::new());
  // provide_context(response_cache);

  let scroll_element: RwSignal<Option<NodeRef<Div>>> = RwSignal::new(None);
  provide_context(scroll_element);

  let (get_auth_cookie, set_auth_cookie) =
    use_cookie_with_options::<String, FromToStringCodec>("jwt", UseCookieOptions::default().max_age(691200000).path("/").same_site(SameSite::Lax));
  provide_context(ReadAuthCookie(get_auth_cookie));
  provide_context(WriteAuthCookie(set_auth_cookie));
  #[cfg(feature = "ssr")]
  if let Some(t) = get_auth_cookie.get() {
    set_auth_cookie.set(Some(t));
  }

  let (get_instance_cookie, set_instance_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
    "instance",
    UseCookieOptions::default().max_age(691200000).path("/").same_site(SameSite::Lax),
  );
  provide_context(ReadInstanceCookie(get_instance_cookie));
  provide_context(WriteInstanceCookie(set_instance_cookie));
  #[cfg(feature = "ssr")]
  if let Some(t) = get_instance_cookie.get() {
    set_instance_cookie.set(Some(t));
  } else {
    set_instance_cookie.set(Some("lemmy.world".to_string()));
  }

  // #[cfg(feature = "ssr")]
  let (get_theme_cookie, set_theme_cookie) =
    use_cookie_with_options::<String, FromToStringCodec>("theme", UseCookieOptions::default().max_age(691200000).path("/").same_site(SameSite::Lax));
  provide_context(ReadThemeCookie(get_theme_cookie));
  provide_context(WriteThemeCookie(set_theme_cookie));
  #[cfg(feature = "ssr")]
  if let Some(t) = get_theme_cookie.get() {
    set_theme_cookie.set(Some(t));
  }

  let ssr_site_signal: RwSignal<Option<Result<GetSiteResponse, LemmyAppError>>> = RwSignal::new(None);

  let ssr_site = Resource::new(
    move || (),
    move |()| async move {
      let result: Result<GetSiteResponse, LemmyAppError> = { LemmyClient.get_site().await };
      // ssr_site_signal.set(Some(result.clone()));
      match result {
        Ok(o) => Ok(o),
        Err(e) => Err(e),
      }
    },
  );

  provide_context(ssr_site);
  provide_context(ssr_site_signal);

  let formatter = move |text: String| match ssr_site_signal.get() {
    Some(Ok(site)) => {
      if text.len() > 0 {
        if let Some(d) = site.site_view.site.description {
          format!("{} - AOS for {} - {}", text, site.site_view.site.name, d)
        } else {
          format!("{} - AOS for {}", text, site.site_view.site.name)
        }
      } else {
        if let Some(d) = site.site_view.site.description {
          format!("AOS for {} - {}", site.site_view.site.name, d)
        } else {
          format!("AOS for {}", site.site_view.site.name)
        }
      }
    }
    _ => "AOS".to_string(),
  };

  view! {
    <Stylesheet id="leptos" href="/pkg/aos.css" />
    <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico" />
    <Link rel="manifest" href="/manifest.json" />
    <Meta name="description" content={formatter("".into())} />
    <Title formatter />
    <Router>
      <Routes fallback={NotFound}>
        <ParentRoute path={(StaticSegment(""))} view={Root} ssr={SsrMode::Async}>
          <Route path={(StaticSegment(""))} view={Home} />
          <Route path={(StaticSegment("l"))} view={Login} />
          <Route path={(StaticSegment("p"), ParamSegment("id"))} view={Post} />
          <Route path={(StaticSegment("c"), ParamSegment("name"))} view={Home} />
          <Route path={(StaticSegment("s"))} view={Search} />
        </ParentRoute>
      </Routes>
    </Router>
  }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
  console_error_panic_hook::set_once();
  leptos::mount::hydrate_body(App);
}
