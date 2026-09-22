use core::num::ParseIntError;
use lemmy_api_common::LemmyErrorType;
use leptos::{
  logging::error,
  prelude::*,
};
use leptos_router::{components::*, location::State, *, hooks::*};
use serde::{Deserialize, Serialize};
use serde_urlencoded::ser;
use web_sys::MouseEvent;

pub type LemmyAppResult<T> = Result<T, LemmyAppError>;

#[derive(Default, strum_macros::Display, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "error", content = "message", rename_all = "snake_case")]
pub enum LemmyAppErrorType {
  #[default]
  Unknown,

  NotFound,
  InternalServerError,
  InternalClientError,
  ParamsError,
  OfflineError,

  ApiError(LemmyErrorType),

  EmptyUsername,
  EmptyPassword,
  MissingToken,

  MissingReason,
}

pub fn message_from_error(error: &LemmyAppError) -> String {
  // let i18n = use_i18n();

  let s = match error.error_type {
    // LemmyAppErrorType::ApiError(LemmyErrorType::IncorrectLogin) => t!(i18n, invalid_login)().into_any().to_s,
    // LemmyAppErrorType::EmptyUsername => t!(i18n, empty_username),
    // LemmyAppErrorType::EmptyPassword => t!(i18n, empty_password),
    // LemmyAppErrorType::MissingReason => t!(i18n, empty_reason),
    // LemmyAppErrorType::InternalServerError => t!(i18n, internal),
    // LemmyAppErrorType::Unknown => t!(i18n, unknown),
    LemmyAppErrorType::OfflineError => "App is offline".to_owned(),
    _ => "An error without description".to_owned(),
  };

  leptos::logging::error!("{}\n{:#?}", s, error);

  s
}

#[component]
pub fn Offline(#[prop(optional, into)] on_retry_click: Option<Callback<MouseEvent>>) -> impl IntoView {
  view! {
    <div class="py-4 px-8 break-inside-avoid">
      <div class="flex justify-between alert alert-warning alert-soft">
        <span class="text-lg">{"Offline"}</span>
        {if let Some(o) = on_retry_click {
          view! {
            <span on:click={ move |e: MouseEvent| { o.run(e); } } class="btn btn-sm">
              "Retry"
            </span>
          }.into_any()
        } else {
          view! {}.into_any()
        }}
      </div>
    </div>
  }
  .into_any()
}

#[component]
pub fn Error(#[prop(optional, into)] get_url: Option<bool>, #[prop(optional, into)] description: Option<String>, error: LemmyAppError, #[prop(optional, into)] on_retry_click: Option<Callback<MouseEvent>>) -> impl IntoView {
  error!("{:#?}", error);
  let description_value = description.unwrap_or("Error".to_owned());
  let location = use_location();
  let search = location.search.get();

  view! {
    <div class="py-4 px-8 break-inside-avoid">
      <div class="flex alert alert-error alert-soft">
        <details class="w-full min-w-0">
          <summary class="flex justify-between list-none">
            <span class="text-lg"> { description_value } </span>
            {if let Some(o) = on_retry_click {
              {if let Some(true) = get_url {
                view! {
                  <a href=format!("{}{}", location.pathname.get(), if search.len() > 0 { format!("?{}", search) } else { "".into() }) on:click={ move |e: MouseEvent| { e.prevent_default(); o.run(e); } } class="btn btn-sm">
                    "Retry"
                  </a>
                }.into_any()
              } else {
                view! {
                  <span on:click={ move |e: MouseEvent| { e.prevent_default(); o.run(e); } } class="btn btn-sm">
                    "Retry"
                  </span>
                }.into_any()
              }}
            } else {
              {if let Some(true) = get_url {
                view! {
                  <a href=format!("{}{}", location.pathname.get(), if search.len() > 0 { format!("?{}", search) } else { "".into() }) class="btn btn-sm">
                    "Retry"
                  </a>
                }.into_any()
              } else {
                view! {}.into_any()
              }}
            }}
          </summary>
          <div class="overflow-auto">
            <pre> { format!("{:#?}", error.error_type) } </pre>
            <pre> { error.content } </pre>
          </div>
        </details>
      </div>
    </div>
  }
  .into_any()
}

#[component]
pub fn Loading(loading: bool) -> impl IntoView {
  if loading {
    view! {
      <div class="overflow-hidden break-inside-avoid animate-[popdown_1s_step-end_1]">
        <div class="py-4 px-8">
          <div class="alert alert-info alert-soft">
            <span>"Loading..."</span>
          </div>
        </div>
      </div>
    }
    .into_any()
  } else {
    view! {}.into_any()
  }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct LemmyAppError {
  pub error_type: LemmyAppErrorType,
  pub content: String,
}

impl serde::ser::StdError for LemmyAppError {}

impl core::fmt::Debug for LemmyAppError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("debug LemmyAppError").field("error_type", &self.error_type).field("content", &self.content).finish()
  }
}

impl core::fmt::Display for LemmyAppError {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    match &self.error_type {
      LemmyAppErrorType::ApiError(inner) => {
        write!(f, "{{\"error_type\":{{\"{}\": {}}}}}", &self.error_type, serde_json::to_string(inner).unwrap_or("".to_owned()))
      }
      _ => {
        write!(f, "{{\"error_type\":\"{}\"}}", &self.error_type)
      }
    }
  }
}

impl From<LemmyErrorType> for LemmyAppError {
  fn from(error_type: LemmyErrorType) -> Self {
    LemmyAppError { error_type: LemmyAppErrorType::ApiError(error_type.clone()), content: format!("{:#?}", error_type) }
  }
}

impl From<LemmyAppErrorType> for LemmyAppError {
  fn from(error_type: LemmyAppErrorType) -> Self {
    LemmyAppError { error_type, content: "".to_owned() }
  }
}

impl From<ser::Error> for LemmyAppError {
  fn from(value: ser::Error) -> Self {
    Self { error_type: LemmyAppErrorType::InternalServerError, content: format!("{:#?}", value) }
  }
}

impl From<serde_json::error::Error> for LemmyAppError {
  fn from(value: serde_json::error::Error) -> Self {
    Self { error_type: LemmyAppErrorType::InternalServerError, content: format!("{:#?}", value) }
  }
}

impl From<ParseIntError> for LemmyAppError {
  fn from(value: ParseIntError) -> Self {
    Self { error_type: LemmyAppErrorType::ParamsError, content: format!("{:#?}", value) }
  }
}

impl From<web_sys::wasm_bindgen::JsValue> for LemmyAppError {
  fn from(value: web_sys::wasm_bindgen::JsValue) -> Self {
    Self { error_type: LemmyAppErrorType::InternalClientError, content: format!("{:#?}", value) }
  }
}

#[cfg(not(feature = "ssr"))]
impl From<gloo_net::Error> for LemmyAppError {
  fn from(value: gloo_net::Error) -> Self {
    Self { error_type: LemmyAppErrorType::InternalServerError, content: format!("{:#?}", value) }
  }
}

// #[cfg(feature = "ssr")]
// impl From<awc::error::JsonPayloadError> for LemmyAppError {
//   fn from(value: awc::error::JsonPayloadError) -> Self {
//     Self { error_type: LemmyAppErrorType::InternalServerError, content: format!("{:#?}", value) }
//   }
// }

// #[cfg(feature = "ssr")]
// impl From<awc::error::SendRequestError> for LemmyAppError {
//   fn from(value: awc::error::SendRequestError) -> Self {
//     use std::error::Error;
//     Self { error_type: LemmyAppErrorType::InternalServerError, content: format!("{} - source: {:?}", value, value.source()) }
//   }
// }

// #[cfg(feature = "ssr")]
// impl From<awc::error::PayloadError> for LemmyAppError {
//   fn from(value: awc::error::PayloadError) -> Self {
//     Self { error_type: LemmyAppErrorType::InternalServerError, content: format!("{:#?}", value) }
//   }
// }

#[cfg(feature = "ssr")]
impl From<core::str::Utf8Error> for LemmyAppError {
  fn from(value: core::str::Utf8Error) -> Self {
    Self { error_type: LemmyAppErrorType::InternalServerError, content: format!("{:#?}", value) }
  }
}

#[cfg(feature = "ssr")]
impl From<ServerFnError> for LemmyAppError {
  fn from(value: ServerFnError) -> Self {
    Self { error_type: LemmyAppErrorType::InternalServerError, content: format!("{:#?}", value) }
  }
}

#[cfg(feature = "ssr")]
impl From<LemmyAppErrorType> for ServerFnError {
  fn from(value: LemmyAppErrorType) -> Self {
    Self::ServerError(format!("{:#?}", value))
  }
}
