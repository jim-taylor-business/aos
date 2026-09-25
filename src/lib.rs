#![recursion_limit = "512"]
#![allow(warnings)]

pub mod client;
pub mod comment;
pub mod comments;
pub mod community;
pub mod db;
pub mod default;
pub mod errors;
pub mod hero;
pub mod icon;
pub mod listing;
pub mod listings;
pub mod login;
pub mod nav;
pub mod overview;
pub mod post;
pub mod search;
pub mod toolbar;
pub mod user;

use crate::{
  client::{LemmyApi, LemmyClient},
  errors::{LemmyAppError, LemmyAppResult},
  login::Login,
  post::Post,
  search::Search,
  user::User,
};
use codee::string::FromToStringCodec;
use community::Community;
use default::Default;
use lemmy_api_common::{
  comment::{GetComments, GetCommentsResponse},
  post::{GetPost, GetPostResponse, GetPosts, GetPostsResponse},
  site::GetSiteResponse,
};
use leptos::prelude::*;
use leptos::logging::log;
use leptos_meta::{Link, MetaTags, Stylesheet, provide_meta_context, *};
use leptos_router::{
  StaticSegment, components::{Outlet, ParentRoute, Route, Router, Routes}, *,
};
#[cfg(not(feature = "ssr"))]
use leptos_use::use_document_visibility;
use leptos_use::{SameSite, UseCookieOptions, use_cookie_with_options};
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

#[derive(Clone, Debug, Default)]
pub struct PassedUrl(pub Option<String>);

pub fn html_template(options: LeptosOptions) -> impl IntoView {
  view! {
    <!DOCTYPE html>
    <html lang="en">
      <head>
        <Link rel="preload" href="/AdwaitaSans-Italic.ttf" as_="font" crossorigin="anonymous" />
        <Link rel="preload" href="/AdwaitaSans-Regular.ttf" as_="font" crossorigin="anonymous" />
        <Link rel="preload" href="/icons.svg" as_="image" />
        <MetaTags />
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        // <AutoReload options={options.clone()} />
        <HydrationScripts options />
      </head>
      <body>
        <App />
      </body>
    </html>
  }
}

#[component]
pub fn Root() -> impl IntoView {
  let ReadThemeCookie(get_theme_cookie) = expect_context::<ReadThemeCookie>();
  view! {
    <div class="flex flex-col min-h-screen" data-theme={move || get_theme_cookie.get()}>
      <Outlet />
    </div>
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

  let on_scroll_element = NodeRef::<leptos::html::Div>::new();

  let location = hooks::use_location();
  let search = location.search.get();
  let url = format!("{}{}", location.pathname.get(), if search.len() > 0 { format!("?{}", search) } else { "".into() });

  view! {
    <div class="flex flex-col min-h-screen" data-theme={move || get_theme_cookie.get()}>
      <main class="flex flex-col">
        <nav::TopNav scroll_element={on_scroll_element.into()} lost_path={RwSignal::new(url)} />
        <div class="flex flex-grow">
          <div
            node_ref={on_scroll_element}
            class="min-w-full sm:overflow-x-auto sm:overflow-y-hidden sm:absolute sm:px-4 gap-4{} sm:h-[calc(100%-4rem)] sm:columns-[23rem]"
          >
            <div class="flex justify-between alert alert-warning alert-soft">
              <span class="text-lg"> "Link not found, try searching instead" </span>
            </div>
          </div>
        </div>
      </main>
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

  // #[cfg(not(feature = "ssr"))]
  // let UseServiceWorkerReturn { .. } =
  //   use_service_worker_with_options(UseServiceWorkerOptions::default().script_url("/service-worker.js").skip_waiting_message("skipWaiting"));

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

  let listing_browser_cache: RwSignal<BTreeMap<(usize, GetPosts, Option<String>), (i64, LemmyAppResult<GetPostsResponse>)>> =
    RwSignal::new(BTreeMap::new());
  provide_context(listing_browser_cache);
  let post_browser_cache: RwSignal<BTreeMap<(GetPost, Option<String>), (i64, LemmyAppResult<GetPostResponse>)>> = RwSignal::new(BTreeMap::new());
  provide_context(post_browser_cache);
  let comments_browser_cache: RwSignal<BTreeMap<(GetComments, Option<String>), (i64, LemmyAppResult<GetCommentsResponse>)>> =
    RwSignal::new(BTreeMap::new());
  provide_context(comments_browser_cache);

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
    set_instance_cookie.set(Some("lemmy.world".to_owned()));
  }

  let (get_theme_cookie, set_theme_cookie) =
    use_cookie_with_options::<String, FromToStringCodec>("theme", UseCookieOptions::default().max_age(691200000).path("/").same_site(SameSite::Lax));
  provide_context(ReadThemeCookie(get_theme_cookie));
  provide_context(WriteThemeCookie(set_theme_cookie));
  #[cfg(feature = "ssr")]
  if let Some(t) = get_theme_cookie.get() {
    set_theme_cookie.set(Some(t));
  }

  let ssr_site = Resource::new(
    move || (),
    move |()| async move {
      let result: Result<GetSiteResponse, LemmyAppError> = { LemmyClient.get_site().await };
      match result {
        Ok(o) => Ok(o),
        Err(e) => Err(e),
      }
    },
  );

  provide_context(ssr_site);
  provide_context(RwSignal::new(PassedUrl(Some("face".to_string()))));

  view! {
    <Transition fallback={|| {}}>
      {move || {
        ssr_site
          .get()
          .map(|s| {
            match s {
              Ok(site) => {
                if let Some(d) = site.site_view.site.description {
                  view! {
                    <Title
                      formatter={move |text: String| {
                        if text.len() > 0 {
                          format!("{} - AOS for {} - {}", text, site.site_view.site.name, d.clone())
                        } else {
                          format!("AOS for {} - {}", site.site_view.site.name, d.clone())
                        }
                      }}
                      text=""
                    />
                  }
                } else {
                  view! {
                    <Title
                      formatter={move |text: String| {
                        if text.len() > 0 {
                          format!("{} - AOS for {}", text, site.site_view.site.name)
                        } else {
                          format!("AOS for {}", site.site_view.site.name)
                        }
                      }}
                      text=""
                    />
                  }
                }
              }
              _ => {
                view! { <Title formatter={|text: String| if text.len() > 0 { format!("{} - AOS", text) } else { format!("AOS") }} text="" /> }
              }
            }
          })
      }}
    </Transition>
    <Stylesheet id="leptos" href="/pkg/aos.css" />
    <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico" />
    <Link rel="manifest" href="/manifest.json" />
    <Router>
      <Routes fallback={NotFound}>
        <ParentRoute path={StaticSegment("")} view={Root} ssr={SsrMode::Async}>
          <Route path={StaticSegment("")} view={Default} />
          <Route path={StaticSegment("l")} view={Login} />
          <Route path={(StaticSegment("p"), ParamSegment("id"))} view={Post} />
          <Route path={(StaticSegment("c"), ParamSegment("name"))} view={Community} />
          <Route path={(StaticSegment("u"), ParamSegment("name"))} view={User} />
          <Route path={StaticSegment("s")} view={Search} />
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
