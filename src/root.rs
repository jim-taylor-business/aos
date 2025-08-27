use lemmy_api_common::site::GetSiteResponse;
use leptos::*;
use leptos_router::Outlet;

use crate::errors::LemmyAppError;

#[component]
pub fn Root(ssr_site: Resource<Option<String>, Result<GetSiteResponse, LemmyAppError>>) -> impl IntoView {
  view! {
    <Transition fallback={|| {}}>
      {move || {
        ssr_site
          .get()
          .map(|_| {
            view! {
              <Outlet />
            }
          })
      }}
    </Transition>
  }
}
