use crate::{
  client::*,
  errors::{LemmyAppError, LemmyAppErrorType},
  icon::{Icon, IconType::*},
  *,
};
use lemmy_api_common::{
  person::{Login, LoginResponse},
  site::GetSiteResponse,
};
use leptos::{logging::log, prelude::*, task::spawn_local_scoped_with_cancellation};
use leptos_meta::Title;
use leptos_router::{hooks::*, *};
use leptos_use::{UseTimeoutFnReturn, use_timeout_fn};
use web_sys::MouseEvent;

fn validate_login(form: &Login) -> Option<LemmyAppErrorType> {
  if form.username_or_email.len() == 0 {
    return Some(LemmyAppErrorType::EmptyUsername);
  }
  if form.password.len() == 0 {
    return Some(LemmyAppErrorType::EmptyPassword);
  }
  None
}

async fn try_login(form: Login) -> Result<LoginResponse, LemmyAppError> {
  let val = validate_login(&form);
  match val {
    None => {
      use crate::client::*;
      let result = LemmyClient.login(form).await;
      match result {
        Ok(LoginResponse { ref jwt, .. }) => {
          if let Some(_jwt_string) = jwt {
            result
          } else {
            Err(LemmyAppError { error_type: LemmyAppErrorType::MissingToken, content: format!("{:#?}", LemmyAppErrorType::MissingToken) })
          }
        }
        Err(e) => Err(e),
      }
    }
    Some(e) => Err(LemmyAppError { error_type: e.clone(), content: format!("{:#?}", e) }),
  }
}

#[server]
pub async fn login_fn(username_or_email: String, password: String, uri: String) -> Result<(), ServerFnError> {
  let (get_auth_cookie, set_auth_cookie) =
    use_cookie_with_options::<String, FromToStringCodec>("jwt", UseCookieOptions::default().max_age(691200000).path("/").same_site(SameSite::Lax));
  provide_context(ReadAuthCookie(get_auth_cookie));
  provide_context(WriteAuthCookie(set_auth_cookie));
  let (get_instance_cookie, set_instance_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
    "instance",
    UseCookieOptions::default().max_age(691200000).path("/").same_site(SameSite::Lax),
  );
  provide_context(ReadInstanceCookie(get_instance_cookie));
  provide_context(WriteInstanceCookie(set_instance_cookie));
  use leptos_axum::redirect;
  let req = Login { username_or_email: username_or_email.into(), password: password.into(), totp_2fa_token: None };
  let result = try_login(req).await;
  match result {
    Ok(LoginResponse { jwt, .. }) => {
      let WriteAuthCookie(set_auth_cookie) = expect_context::<WriteAuthCookie>();
      set_auth_cookie.set(Some(jwt.unwrap_or_default().into_inner()));
      if uri.len() > 0 {
        redirect(&uri);
      } else {
        redirect("/");
      }
      Ok(())
    }
    Err(e) => {
      redirect(&format!("/l?error={}", serde_json::to_string(&e)?)[..]);
      Ok(())
    }
  }
}

#[component]
pub fn LoginForm() -> impl IntoView {
  // let _i18n = use_i18n();
  // log!("LOGIN");

  let query = use_query_map();
  let name = RwSignal::new(String::new());
  let password = RwSignal::new(String::new());
  let login = ServerAction::<LoginFn>::new();
  let username_validation = RwSignal::new("".to_owned());
  let password_validation = RwSignal::new("".to_owned());
  let ssr_error = move || query.with(|params| params.get("error"));
  let ssr_site = expect_context::<Resource<Result<GetSiteResponse, LemmyAppError>>>();
  // let ssr_site_signal = expect_context::<RwSignal<Option<Result<GetSiteResponse, LemmyAppError>>>>();

  if let Some(e) = ssr_error() {
    let le = serde_json::from_str::<LemmyAppError>(&e[..]);

    match le {
      Ok(e) => match e.error_type {
        LemmyAppErrorType::EmptyUsername => username_validation.set("input-error".to_owned()),
        LemmyAppErrorType::EmptyPassword => password_validation.set("input-error".to_owned()),
        _ => {}
      },
      Err(_) => {}
    }
  }

  let on_login_submit = move |e: MouseEvent| {
    e.prevent_default();
    spawn_local_scoped_with_cancellation(async move {
      let req = Login { username_or_email: name.get().into(), password: password.get().into(), totp_2fa_token: None };
      let result = try_login(req.clone()).await;
      match result {
        Ok(LoginResponse { jwt: Some(jwt), .. }) => {
          let WriteAuthCookie(set_auth_cookie) = expect_context::<WriteAuthCookie>();
          set_auth_cookie.set(Some(jwt.clone().into_inner()));
          ssr_site.refetch();
          use_navigate()("/", Default::default());
          // spawn_local_scoped_with_cancellation(async move {
          //   ssr_site.refetch();
          //   spawn_local_scoped_with_cancellation(async move {
          //   });
          // });
          // let UseTimeoutFnReturn { start, stop, is_pending, .. } = use_timeout_fn(
          //   |i: i32| {
          //     ssr_site.refetch();
          //   },
          //   500.0,
          // );
          // start(3);

          // ssr_site_signal.set(Some(LemmyClient.get_site().await));
        }
        Ok(LoginResponse { jwt: None, .. }) => {}
        Err(e) => {
          password_validation.set("".to_owned());
          username_validation.set("".to_owned());
          match e {
            LemmyAppError { error_type: LemmyAppErrorType::EmptyUsername, .. } => {
              username_validation.set("input-error".to_owned());
            }
            LemmyAppError { error_type: LemmyAppErrorType::EmptyPassword, .. } => {
              password_validation.set("input-error".to_owned());
            }
            _ => {}
          }
        }
      }
    });
  };

  view! {
    <div>
      <ActionForm attr:class="space-y-3" action={login}>
        <input type="hidden" name="uri" value={move || query.get().get("uri").unwrap_or("".into())} />
        <TextInput id="username" autocomplete="username" name="username_or_email" input_value={name} label="Username" />
        <TextInput
          id="password"
          name="password"
          autocomplete="current-password"
          validation_class={password_validation.into()}
          input_value={password}
          input_type={InputType::Password}
          label="Password"
        />
        <button class="btn btn-neutral" on:click={on_login_submit} type="submit">
          "Login"
        </button>
      </ActionForm>
    </div>
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputType {
  Text,
  Password,
}

#[component]
pub fn TextInput(
  #[prop(optional)] disabled: MaybeProp<bool>,
  #[prop(optional)] required: MaybeProp<bool>,
  #[prop(into)] id: TextProp,
  #[prop(into)] name: TextProp,
  #[prop(optional, into)] autocomplete: TextProp,
  #[prop(into)] label: TextProp,
  #[prop(into)] input_value: RwSignal<String>,
  #[prop(default = InputType::Text)] input_type: InputType,
  #[prop(optional)] validation_class: Signal<String>,
) -> impl IntoView {
  let show_password = RwSignal::new(false);
  let eye_icon = Signal::derive(move || if show_password.get() { EyeSlash } else { Eye });

  view! {
    <label class="flex relative gap-2 items-center">
      <input
        type={move || { if input_type == InputType::Text || show_password.get() { "text" } else { "password" } }}
        id={id}
        autocomplete={autocomplete}
        class={move || { format!("input input-bordered p-4 grow {}", validation_class.get()) }}
        placeholder={move || label.get()}
        name={move || name.get()}
        disabled={move || disabled.get().unwrap_or(false)}
        required={move || required.get().unwrap_or(false)}
        on:input={move |e| {
          input_value.set(event_target_value(&e));
        }}
      />
      <Show when={move || input_type == InputType::Password}>
        <button
          type="button"
          class="absolute bottom-2 btn btn-ghost btn-sm btn-circle end-1 text-accent"
          on:click={move |_| show_password.update(|p| *p = !*p)}
        >
          <Icon icon={eye_icon} />
        </button>
      </Show>
    </label>
  }
}

#[component]
pub fn Login() -> impl IntoView {
  view! {
    <Title text="Login" />
    <main class="p-3 mx-auto max-w-screen-md">
      <LoginForm />
    </main>
  }
}
