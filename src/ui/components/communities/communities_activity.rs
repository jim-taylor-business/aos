use crate::ui::components::common::about::About;
// use leptos::*;
use leptos::prelude::*;

#[component]
pub fn CommunitiesActivity() -> impl IntoView {
  view! {
    <main class="mx-auto">
      <About />
    </main>
  }
}
