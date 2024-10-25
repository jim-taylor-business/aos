use crate::ui::components::login::login_form::LoginForm;
use leptos::*;
use leptos_meta::Title;

#[component]
pub fn LoginActivity() -> impl IntoView {
  // let title = expect_context::<RwSignal<Option<TitleSetter>>>();

  // title.set(Some(TitleSetter("Login".into())));

  view! {
    <Title text="Login" />
    <main class="p-3 mx-auto max-w-screen-md">
      <LoginForm />
    </main>
  }
}
