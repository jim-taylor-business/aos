use crate::ReadThemeCookie;
use leptos::prelude::*;
use leptos_router::components::Outlet;

#[component]
pub fn Root() -> impl IntoView {
  let ReadThemeCookie(get_theme_cookie) = expect_context::<ReadThemeCookie>();
  view! {
    <div class="flex flex-col min-h-screen" data-theme={move || get_theme_cookie.get()}>
      <Outlet />
    </div>
  }
}
