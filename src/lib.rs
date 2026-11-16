#![recursion_limit = "512"]
#![allow(warnings)]

pub mod client;
pub mod comment;
pub mod db;
pub mod errors;
pub mod hero;
pub mod icon;
include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));
pub mod listing;
pub mod login;
pub mod nav;
pub mod overview;
pub mod post;
pub mod prelude;
pub mod search;
pub mod toolbar;
pub mod user;

use crate::prelude::*;
#[cfg(not(feature = "ssr"))]
use leptos::ev::selectionchange;
#[cfg(not(feature = "ssr"))]
use leptos_use_document::use_document;
#[cfg(not(feature = "ssr"))]
use leptos_use_event_listener::use_event_listener;

pub fn html_template(options: LeptosOptions) -> impl IntoView {
  view! {
    <!DOCTYPE html>
    <html>
      <head>
        <Link rel="preload" href="/AdwaitaSans-Italic.ttf" as_="font" type_="font/ttf" crossorigin="anonymous" />
        <Link rel="preload" href="/AdwaitaSans-Regular.ttf" as_="font" type_="font/ttf" crossorigin="anonymous" />
        <Link rel="prefetch" href="/icons.svg" as_="image" type_="image/svg+xml" />
        <MetaTags />
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
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
  let i18n = use_i18n();
  #[cfg(feature = "ssr")]
  {
    let resp = expect_context::<leptos_axum::ResponseOptions>();
    resp.set_status(http::StatusCode::NOT_FOUND);
  }
  let ReadThemeCookie(get_theme_cookie) = expect_context::<ReadThemeCookie>();
  let on_scroll_element = NodeRef::<Div>::new();
  let location = use_location();
  let search = location.search.get();
  let url = format!("{}{}", location.pathname.get(), if search.len() > 0 { format!("?{}", search) } else { "".into() });

  view! {
    <div class="flex flex-col min-h-screen" data-theme={move || get_theme_cookie.get()}>
      <main class="flex flex-col">
        <TopNav scroll_element={on_scroll_element.into()} lost_path={RwSignal::new(url)} />
        <div class="flex flex-grow">
          <div
            node_ref={on_scroll_element}
            class="min-w-full sm:overflow-x-auto sm:overflow-y-hidden sm:absolute sm:px-4 gap-4{} sm:h-[calc(100%-4rem)] sm:columns-[23rem]"
          >
            <Warning description={t_string!(i18n, link_not_found).to_string()} />
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

  // #[cfg(not(feature = "ssr"))]
  // let visibility = use_document_visibility();
  // #[cfg(not(feature = "ssr"))]
  // provide_context(visibility);

  let on_online = move |b: bool| {
    move |_| {
      online.set(OnlineSetter(b));
    }
  };
  let _offline_handle = window_event_listener_untyped("offline", on_online(false));
  let _online_handle = window_event_listener_untyped("online", on_online(true));

  let listing_browser_cache: RwSignal<BTreeMap<(usize, GetPosts, Option<String>, bool), (i64, LemmyAppResult<GetPostsResponse>)>> =
    RwSignal::new(BTreeMap::new());
  provide_context(listing_browser_cache);
  let post_browser_cache: RwSignal<BTreeMap<(GetPost, Option<String>), (i64, LemmyAppResult<GetPostResponse>)>> = RwSignal::new(BTreeMap::new());
  provide_context(post_browser_cache);
  let comments_browser_cache: RwSignal<BTreeMap<(GetComments, Option<String>), (i64, LemmyAppResult<GetCommentsResponse>)>> =
    RwSignal::new(BTreeMap::new());
  provide_context(comments_browser_cache);
  let search_browser_cache: RwSignal<BTreeMap<(usize, Search, Option<String>), (i64, LemmyAppResult<SearchResponse>)>> =
    RwSignal::new(BTreeMap::new());
  provide_context(search_browser_cache);
  let user_browser_cache: RwSignal<BTreeMap<(usize, GetPersonDetails, Option<String>), (i64, LemmyAppResult<GetPersonDetailsResponse>)>> =
    RwSignal::new(BTreeMap::new());
  provide_context(user_browser_cache);

  let (get_auth_cookie, set_auth_cookie) = provide_auth_cookie();
  #[cfg(feature = "ssr")]
  if let Some(t) = get_auth_cookie.get() {
    set_auth_cookie.set(Some(t));
  }

  let (get_instance_cookie, set_instance_cookie) = provide_instance_cookie();
  #[cfg(feature = "ssr")]
  if let Some(t) = get_instance_cookie.get() {
    set_instance_cookie.set(Some(t));
  } else {
    set_instance_cookie.set(Some("lemmy.world".to_owned()));
  }

  let (get_theme_cookie, set_theme_cookie) = provide_theme_cookie();
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

  let selected_text = RwSignal::new(SelectedText(None));
  provide_context(selected_text);

  #[cfg(not(feature = "ssr"))]
  let _selection_listener = use_event_listener(use_document(), selectionchange, move |e| {
    // let document_selection_converted_to_rust_string =
    let selection = document()
      .get_selection()
      .ok()
      .flatten()
      .map(|s| s.to_string().to_string().into())
      .map(|s: String| if s.len() > 0 { Some(s) } else { None })
      .flatten();
    // log!("Selected text: {:?}", selection);
    // let convert_option_string_to_none_if_len_is_zero = sel.map(|s: String| if s.len() > 0 { Some(s) } else { None }).flatten();
    selected_text.set(SelectedText(selection));
  });

  view! {
    <I18nContextProvider cookie_name="i18n_pref_locale">
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
                            format!("{} - AOS - {} - {}", text, site.site_view.site.name, d.clone())
                          } else {
                            format!("AOS - {} - {}", site.site_view.site.name, d.clone())
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
                            format!("{} - AOS - {}", text, site.site_view.site.name)
                          } else {
                            format!("AOS - {}", site.site_view.site.name)
                          }
                        }}
                        text=""
                      />
                    }
                  }
                }
                _ => {
                  view! { <Title formatter={|text: String| if text.len() > 0 { format!("{} - AOS", text) } else { "AOS".to_owned() }} text="" /> }
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
            <Route path={(StaticSegment("p"), ParamSegment("id"))} view={Post} />
            <Route path={(StaticSegment("c"), ParamSegment("name"))} view={Community} />
            <Route path={(StaticSegment("u"), ParamSegment("name"))} view={User} />
            <Route path={StaticSegment("s")} view={DefaultSearch} />
            <Route path={(StaticSegment("p"), ParamSegment("id"), StaticSegment("s"))} view={PostSearch} />
            <Route path={(StaticSegment("c"), ParamSegment("name"), StaticSegment("s"))} view={CommunitySearch} />
            <Route path={(StaticSegment("u"), ParamSegment("name"), StaticSegment("s"))} view={UserSearch} />
            <Route path={StaticSegment("l")} view={Login} />
          </ParentRoute>
        </Routes>
      </Router>
    </I18nContextProvider>
  }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
  console_error_panic_hook::set_once();
  leptos::mount::hydrate_body(App);
}
