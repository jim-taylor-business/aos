use crate::ui::components::login::login_form::LoginForm;
use leptos::*;
use leptos_meta::Title;

#[component]
pub fn LoginActivity() -> impl IntoView {
  view! {
    <Title text="Login" />
    <main class="p-3 mx-auto max-w-screen-md">
      <LoginForm />
    </main>
  }
}
