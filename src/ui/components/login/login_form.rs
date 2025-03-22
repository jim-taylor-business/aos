use crate::{
  errors::{LemmyAppError, LemmyAppErrorType},
  i18n::*,
  ui::components::common::text_input::{InputType, TextInput},
};
use codee::string::FromToStringCodec;
use lemmy_api_common::person::{Login, LoginResponse};
use leptos::*;
use leptos_router::*;
use leptos_use::{use_cookie_with_options, SameSite, UseCookieOptions};
use web_sys::SubmitEvent;

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
      use crate::lemmy_client::*;
      let result = LemmyClient.login(form).await;
      match result {
        Ok(LoginResponse { ref jwt, .. }) => {
          if let Some(_jwt_string) = jwt {
            result
          } else {
            Err(LemmyAppError {
              error_type: LemmyAppErrorType::MissingToken,
              content: format!("{:#?}", LemmyAppErrorType::MissingToken),
            })
          }
        }
        Err(e) => Err(e),
      }
    }
    Some(e) => Err(LemmyAppError {
      error_type: e.clone(),
      content: format!("{:#?}", e),
    }),
  }
}

#[server(LoginFn, "/serverfn")]
pub async fn login(username_or_email: String, password: String, uri: String) -> Result<(), ServerFnError> {
  use leptos_actix::redirect;
  let req = Login {
    username_or_email: username_or_email.into(),
    password: password.into(),
    totp_2fa_token: None,
  };
  let result = try_login(req).await;
  match result {
    Ok(LoginResponse { jwt, .. }) => {
      let (_, set_auth_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
        "jwt",
        UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax),
      );
      set_auth_cookie.set(Some(jwt.unwrap_or_default().into_inner()));
      if uri.len() > 0 {
        redirect(&uri);
      } else {
        redirect("/");
      }
      Ok(())
    }
    Err(e) => {
      redirect(&format!("/login?error={}", serde_json::to_string(&e)?)[..]);
      Ok(())
    }
  }
}

#[component]
pub fn LoginForm() -> impl IntoView {
  let _i18n = use_i18n();

  let query = use_query_map();

  let error = expect_context::<RwSignal<Vec<Option<(LemmyAppError, Option<RwSignal<bool>>)>>>>();
  let authenticated = expect_context::<RwSignal<Option<bool>>>();
  // let uri = expect_context::<RwSignal<UriSetter>>();

  let name = RwSignal::new(String::new());
  let password = RwSignal::new(String::new());

  let login = create_server_action::<LoginFn>();

  let username_validation = RwSignal::new("".to_string());
  let password_validation = RwSignal::new("".to_string());

  let ssr_error = move || query.with(|params| params.get("error").cloned());

  if let Some(e) = ssr_error() {
    let le = serde_json::from_str::<LemmyAppError>(&e[..]);

    match le {
      Ok(e) => match e.error_type {
        LemmyAppErrorType::EmptyUsername => username_validation.set("input-error".to_string()),
        LemmyAppErrorType::EmptyPassword => password_validation.set("input-error".to_string()),
        _ => {}
      },
      Err(_) => {}
    }
  }

  let on_login_submit = move |ev: SubmitEvent| {
    ev.prevent_default();

    create_local_resource(
      move || (name.get(), password.get()),
      move |(name, password)| async move {
        let req = Login {
          username_or_email: name.into(),
          password: password.into(),
          totp_2fa_token: None,
        };
        let result = try_login(req.clone()).await;
        match result {
          Ok(LoginResponse { jwt: Some(jwt), .. }) => {
            let (_, set_auth_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
              "jwt",
              UseCookieOptions::default()
                .max_age(604800000)
                .path("/")
                .secure(true)
                .same_site(SameSite::Lax),
            );
            set_auth_cookie.set(Some(jwt.clone().into_inner()));
            authenticated.set(Some(true));
            // leptos_router::use_navigate()("/", Default::default());
            // leptos_router::use_navigate()(&query.get().get("uri").cloned().unwrap_or("/".into()), Default::default());

            // leptos_router::use_navigate()(&uri.get().0, Default::default());
            let _ = window().history().map(|h| h.back());
          }
          Ok(LoginResponse { jwt: None, .. }) => {
            error.update(|es| {
              es.push(Some((
                LemmyAppError {
                  error_type: LemmyAppErrorType::MissingToken,
                  content: String::default(),
                },
                None,
              )))
            });
          }
          Err(e) => {
            error.update(|es| es.push(Some((e.clone(), None))));
            password_validation.set("".to_string());
            username_validation.set("".to_string());

            match e {
              LemmyAppError {
                error_type: LemmyAppErrorType::EmptyUsername,
                ..
              } => {
                username_validation.set("input-error".to_string());
              }
              LemmyAppError {
                error_type: LemmyAppErrorType::EmptyPassword,
                ..
              } => {
                password_validation.set("input-error".to_string());
              }
              _ => {}
            }
          }
        }
      },
    );
  };

  view! {
    <div>
      <ActionForm class="space-y-3" on:submit={on_login_submit} action={login}>
        <input type="hidden" name="uri" value={move || query.get().get("uri").cloned().unwrap_or("".into())} />
        <TextInput id="username" name="username_or_email" on_input={move |s| update!(| name | * name = s)} label="Username" />
        <TextInput
          id="password"
          name="password"
          validation_class={password_validation.into()}
          on_input={move |s| update!(| password | * password = s)}
          input_type={InputType::Password}
          label="Password"
        />
        <button class="btn btn-neutral" type="submit">
          "Login"
        </button>
      </ActionForm>
    </div>
  }
}
