use crate::overview::Overview;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn Community() -> impl IntoView {
  let param = use_params_map();
  let ssr_name = Signal::derive(move || param.get().get("name"));
  view! {
    <Overview ssr_name />
  }
}
