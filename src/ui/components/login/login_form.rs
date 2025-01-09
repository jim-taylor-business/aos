use crate::{
  errors::{AosAppError, AosAppErrorType},
  // i18n::*,
  ui::components::common::text_input::{InputType, TextInput},
  UriSetter,
};
use codee::string::FromToStringCodec;
use components::Form;
use hooks::use_query_map;
use lemmy_api_common::person::{Login, LoginResponse};
use leptos::{logging, prelude::*, task::spawn};
use leptos_meta::*;
use leptos_router::*;
use leptos_use::{use_cookie_with_options, SameSite, UseCookieOptions};
use web_sys::{Event, MouseEvent, SubmitEvent};

fn validate_login(form: &Login) -> Option<AosAppErrorType> {
  if form.username_or_email.len() == 0 {
    return Some(AosAppErrorType::EmptyUsername);
  }
  if form.password.len() == 0 {
    return Some(AosAppErrorType::EmptyPassword);
  }
  None
}

async fn try_login(form: Login) -> Result<LoginResponse, AosAppError> {
  let val = validate_login(&form);
  leptos::logging::log!("B3");
  match val {
    None => {
      use crate::lemmy_client::*;
      leptos::logging::log!("B4");

      let result = LemmyClient.login(form).await;
      match result {
        Ok(LoginResponse { ref jwt, .. }) => {
          if let Some(_jwt_string) = jwt {
            result
          } else {
            Err(AosAppError {
              context: "Login error".into(),
              error_type: AosAppErrorType::MissingToken,
              description: format!("{:#?}", AosAppErrorType::MissingToken),
            })
          }
        }
        Err(e) => Err(e),
      }
    }
    Some(e) => Err(AosAppError {
      context: "Login Validation error".into(),
      error_type: e.clone(),
      description: format!("{:#?}", e),
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
  // let _i18n = use_i18n();

  let query = use_query_map();

  let error = expect_context::<RwSignal<Vec<Option<(AosAppError, Option<RwSignal<bool>>)>>>>();
  let authenticated = expect_context::<RwSignal<Option<bool>>>();
  let uri = expect_context::<RwSignal<UriSetter>>();

  let name = RwSignal::new(String::new());
  let password = RwSignal::new(String::new());

  let login = ServerAction::<LoginFn>::new();

  let username_validation = RwSignal::new("".to_string());
  let password_validation = RwSignal::new("".to_string());

  let ssr_error = move || query.with(|params| params.get("error").clone());

  if let Some(e) = ssr_error() {
    let le = serde_json::from_str::<AosAppError>(&e[..]);

    match le {
      Ok(e) => match e.error_type {
        AosAppErrorType::EmptyUsername => username_validation.set("input-error".to_string()),
        AosAppErrorType::EmptyPassword => password_validation.set("input-error".to_string()),
        _ => {}
      },
      Err(_) => {}
    }
  }

  // async fn get_user(user: String) -> Result<String, ServerFnError> {
  //   Ok(format!("this user is {user}"))
  // }

  // let on_submit = move |ev: SubmitEvent| {
  let on_submit = move |ev: MouseEvent| {
    ev.prevent_default();
    logging::log!("poooo");

    spawn(async move {
      // let user_res = get_user("user".into()).await.unwrap_or_default();
      // signal.set(user_res);
      let req = Login {
        username_or_email: name.get().into(),
        password: password.get().into(),
        totp_2fa_token: None,
      };
      leptos::logging::log!("B1");
      let result = try_login(req.clone()).await;
      leptos::logging::log!("B20");
      match result {
        Ok(LoginResponse { jwt: Some(jwt), .. }) => {
          leptos::logging::log!("B21");
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
          window().history().map(|h| h.back());
        }
        Ok(LoginResponse { jwt: None, .. }) => {
          leptos::logging::log!("B22");
          error.update(|es| {
            es.push(Some((
              AosAppError {
                context: "Login Reponse error".into(),
                error_type: AosAppErrorType::MissingToken,
                description: String::default(),
              },
              None,
            )))
          });
        }
        Err(e) => {
          leptos::logging::log!("B23 {:#?}", e.clone());
          error.update(|es| es.push(Some((e.clone(), None))));
          password_validation.set("".to_string());
          username_validation.set("".to_string());
          match e {
            AosAppError {
              error_type: AosAppErrorType::EmptyUsername,
              ..
            } => {
              username_validation.set("input-error".to_string());
            }
            AosAppError {
              error_type: AosAppErrorType::EmptyPassword,
              ..
            } => {
              password_validation.set("input-error".to_string());
            }
            _ => {}
          }
        }
      }
    });

    // Resource::new(
    //   move || (name.get(), password.get()),
    //   move |(name, password)| async move {
    //     let req = Login {
    //       username_or_email: name.into(),
    //       password: password.into(),
    //       totp_2fa_token: None,
    //     };
    //     let result = try_login(req.clone()).await;
    //     match result {
    //       Ok(LoginResponse { jwt: Some(jwt), .. }) => {
    //         let (_, set_auth_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
    //           "jwt",
    //           UseCookieOptions::default()
    //             .max_age(604800000)
    //             .path("/")
    //             .secure(true)
    //             .same_site(SameSite::Lax),
    //         );
    //         set_auth_cookie.set(Some(jwt.clone().into_inner()));
    //         authenticated.set(Some(true));
    //         // leptos_router::use_navigate()("/", Default::default());
    //         // leptos_router::use_navigate()(&query.get().get("uri").cloned().unwrap_or("/".into()), Default::default());

    //         // leptos_router::use_navigate()(&uri.get().0, Default::default());
    //         window().history().map(|h| h.back());
    //       }
    //       Ok(LoginResponse { jwt: None, .. }) => {
    //         error.update(|es| {
    //           es.push(Some((
    //             AosAppError {
    //               context: "Login Reponse error".into(),
    //               error_type: AosAppErrorType::MissingToken,
    //               description: String::default(),
    //             },
    //             None,
    //           )))
    //         });
    //       }
    //       Err(e) => {
    //         error.update(|es| es.push(Some((e.clone(), None))));
    //         password_validation.set("".to_string());
    //         username_validation.set("".to_string());
    //         match e {
    //           AosAppError {
    //             error_type: AosAppErrorType::EmptyUsername,
    //             ..
    //           } => {
    //             username_validation.set("input-error".to_string());
    //           }
    //           AosAppError {
    //             error_type: AosAppErrorType::EmptyPassword,
    //             ..
    //           } => {
    //             password_validation.set("input-error".to_string());
    //           }
    //           _ => {}
    //         }
    //       }
    //     }
    //   },
    // );
  };

  view! {
    // <form action="" onsubmit={on_submit}>
    // <Form attr:class="space-y-3" action="" attr:onsubmit={on_submit}>
    //   <input type="hidden" name="uri" /*value={move || query.get().get("uri").cloned().unwrap_or("".into())}*/ />
    //   <TextInput id="username" name="username_or_email" /*on_input={move |s| update!(| name | * name = s)}*/ label="Username" />
    //   <TextInput
    //     id="password"
    //     name="password"
    //     validation_class={password_validation.into()}
    //     // on_input={move |s| update!(| password | * password = s)}
    //     input_type={InputType::Password}
    //     label="Password"
    //   />
    //   <button class="btn btn-neutral" type="submit">
    //     "Login"
    //   </button>

    <ActionForm attr:class="space-y-3" action={login}> // on:submit={on_submit}>
      <input type="hidden" name="uri" /*value={move || query.get().get("uri").cloned().unwrap_or("".into())}*/ />
      <TextInput id="username" name="username_or_email" on_input={move |e: Event| name.set(event_target_value(&e))} label="Username" />
      <TextInput
        id="password"
        name="password"
        validation_class={password_validation.into()}
        on_input={move |e| password.set(event_target_value(&e))}
        // on_input={move |s| update!(| password | * password = s)}
        input_type={InputType::Password}
        label="Password"
      />
      <button class="btn btn-neutral" type="submit" on:click={on_submit}> // on:click=|ev: web_sys::MouseEvent| { ev.prevent_default(); /*ev.set_cancel_bubble(true);*/ }>
      // <button class="btn btn-neutral" type="button" on:>
        "Login"
      </button>
    </ActionForm>
    // </Form>
    // </form>
  }
}
