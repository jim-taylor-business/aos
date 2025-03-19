use super::login_form::LoginForm;
use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn LoginActivity() -> impl IntoView {
  view! {
    <Title text="Login" />
    <main class="p-3 mx-auto max-w-screen-md">
      <LoginForm />
    </main>
  }
  .into_any()
}
