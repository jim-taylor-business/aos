use crate::{
  errors::LemmyAppError,
  ui::components::common::nav::{BottomNav, TopNav},
};
use codee::string::FromToStringCodec;
use lemmy_api_common::site::GetSiteResponse;
use leptos::*;
use leptos_router::Outlet;
use leptos_use::{use_cookie_with_options, SameSite, UseCookieOptions};

#[component]
pub fn Layout(ssr_site: Resource<(Option<String>, Option<String>), Result<GetSiteResponse, LemmyAppError>>) -> impl IntoView {
  view! {
    <TopNav ssr_site />
    <div class="flex flex-col flex-grow w-full">
      <div class="sm:container sm:mx-auto">
        <div class="flex flex-col flex-grow px-0 w-full lg:px-6">
          <Outlet />
        </div>
      </div>
    </div>
    <BottomNav ssr_site />
  }
}
