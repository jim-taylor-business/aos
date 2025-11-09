use crate::{errors::LemmyAppError, ReadThemeCookie};
use lemmy_api_common::site::GetSiteResponse;
use leptos::{prelude::*, server::codee::string::FromToStringCodec};
use leptos_router::components::Outlet;
use leptos_use::*;

#[component]
pub fn Root() -> impl IntoView {
  let ReadThemeCookie(get_theme_cookie) = expect_context::<ReadThemeCookie>();
  let ssr_site = expect_context::<Resource<Result<GetSiteResponse, LemmyAppError>>>();
  let ssr_site_signal = expect_context::<RwSignal<Option<Result<GetSiteResponse, LemmyAppError>>>>();

  view! {
    <Transition fallback={|| {}}>
      {move || {
        ssr_site
          .get()
          .map(|s| {
            ssr_site_signal.set(Some(s));
            view! {
              <div class="flex flex-col min-h-screen" data-theme={move || get_theme_cookie.get()}>
                <Outlet />
              </div>
            }
          })
      }}
    </Transition>
  }
}
