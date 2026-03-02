use crate::{
  ReadThemeCookie,
  client::{LemmyApi, LemmyClient},
  errors::LemmyAppError,
};
use lemmy_api_common::site::GetSiteResponse;
use leptos::{logging::log, prelude::*};
use leptos_router::components::Outlet;

#[component]
pub fn Root() -> impl IntoView {
  let ReadThemeCookie(get_theme_cookie) = expect_context::<ReadThemeCookie>();
  // let ssr_site = expect_context::<Resource<Result<GetSiteResponse, LemmyAppError>>>();
  // let ssr_site_signal = expect_context::<RwSignal<Option<Result<GetSiteResponse, LemmyAppError>>>>();

  // let ssr_site = Resource::new_blocking(
  //   move || (),
  //   move |()| async move {
  //     let result: Result<GetSiteResponse, LemmyAppError> = { LemmyClient.get_site().await };
  //     match result {
  //       Ok(o) => {
  //         ssr_site_signal.set(Some(Ok(o.clone())));
  //         Ok(o)
  //       }
  //       Err(e) => Err(e),
  //     }
  //   },
  // );

  // provide_context(ssr_site);

  view! {
    // <Transition fallback={|| {}}>
    //   {move || {
    //     ssr_site
    //       .get()
    //       .map(|s| {
    //         ssr_site_signal.set(Some(s));
    //         // log!("ROOT");
    //         // view! {
    //         // }
    //       })
    //   }}
    // </Transition>

    // <Transition fallback={|| {}}>
    //   {move || {
    //     ssr_site
    //       .get_untracked()
    //       .map(|s| {
    //         ssr_site_signal.set(Some(s));
    //         view! {
              <div class="flex flex-col min-h-screen" data-theme={move || get_theme_cookie.get()}>
                <Outlet />
              </div>
    //         }
    //       })
    //   }}
    // </Transition>
  }
}
