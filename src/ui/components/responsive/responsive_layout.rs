use crate::errors::LemmyAppError;
use codee::string::FromToStringCodec;
use lemmy_api_common::site::GetSiteResponse;
use leptos::*;
use leptos_router::Outlet;
use leptos_use::{use_cookie_with_options, SameSite, UseCookieOptions};

#[component]
pub fn ResponsiveLayout(ssr_site: Resource<Option<String>, Result<GetSiteResponse, LemmyAppError>>) -> impl IntoView {
  let (get_theme_cookie, _) =
    use_cookie_with_options::<String, FromToStringCodec>("theme", UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax));

  view! {
    // <Transition fallback={|| {}}>
    //   {move || {
    //     ssr_site
    //       .get()
    //       .map(|_| {
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
