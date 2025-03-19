use crate::{i18n::*, lemmy_error::LemmyErrorType};
use core::num::ParseIntError;
// use leptos::*;
use leptos::{logging, prelude::*, text_prop::TextProp};
use leptos_router::{components::*, hooks::*, *};
use serde::{Deserialize, Serialize};
use serde_urlencoded::ser;
use strum_macros::Display;

#[derive(Default, Display, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "error", content = "message", rename_all = "snake_case")]
pub enum AosAppErrorType {
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

#[derive(Clone, Serialize, Deserialize)]
pub struct AosAppError {
  pub context: String,
  pub error_type: AosAppErrorType,
  pub description: String,
}

pub type AosAppResult<T> = Result<T, AosAppError>;

pub fn message_from_error(error: &AosAppError) -> String {
  let i18n = use_i18n();
  let s = match error.error_type {
    AosAppErrorType::ApiError(LemmyErrorType::IncorrectLogin) => t!(i18n, invalid_login)().to_html(),
    AosAppErrorType::EmptyUsername => t!(i18n, empty_username)().to_html(),
    AosAppErrorType::EmptyPassword => t!(i18n, empty_password)().to_html(),
    AosAppErrorType::MissingReason => t!(i18n, empty_reason)().to_html(),
    AosAppErrorType::InternalServerError => t!(i18n, internal)().to_html(),
    AosAppErrorType::Unknown => t!(i18n, unknown)().to_html(),
    AosAppErrorType::OfflineError => "App is offline at the moment".to_string(),
    _ => "An error without description".to_string(),
  };
  logging::error!("{}\n{:#?}", s, error);
  s
}

impl serde::ser::StdError for AosAppError {}

impl core::fmt::Debug for AosAppError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("debug LemmyAppError")
      .field("error_type", &self.error_type)
      .field("content", &self.description)
      .finish()
  }
}

impl core::fmt::Display for AosAppError {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    match &self.error_type {
      AosAppErrorType::ApiError(inner) => {
        write!(
          f,
          "{{\"error_type\":{{\"{}\": {}}}}}",
          &self.error_type,
          serde_json::to_string(inner).ok().unwrap()
        )
      }
      _ => {
        write!(f, "{{\"error_type\":\"{}\"}}", &self.error_type)
      }
    }
  }
}

impl From<LemmyErrorType> for AosAppError {
  fn from(error_type: LemmyErrorType) -> Self {
    AosAppError {
      context: "LemmyError error".into(),
      error_type: AosAppErrorType::ApiError(error_type.clone()),
      description: format!("{:#?}", error_type),
    }
  }
}

impl From<AosAppErrorType> for AosAppError {
  fn from(error_type: AosAppErrorType) -> Self {
    AosAppError {
      context: "AosAppErrorType error".into(),
      error_type,
      description: "".to_string(),
    }
  }
}

impl From<ser::Error> for AosAppError {
  fn from(value: ser::Error) -> Self {
    Self {
      context: "Serde URL error".into(),
      error_type: AosAppErrorType::InternalServerError,
      description: format!("{:#?}", value),
    }
  }
}

impl From<serde_json::error::Error> for AosAppError {
  fn from(value: serde_json::error::Error) -> Self {
    Self {
      context: "Serde JSON error".into(),
      error_type: AosAppErrorType::InternalServerError,
      description: format!("{:#?}", value),
    }
  }
}

impl From<ParseIntError> for AosAppError {
  fn from(value: ParseIntError) -> Self {
    Self {
      context: "ParseIntError error".into(),
      error_type: AosAppErrorType::ParamsError,
      description: format!("{:#?}", value),
    }
  }
}

impl From<web_sys::wasm_bindgen::JsValue> for AosAppError {
  fn from(value: web_sys::wasm_bindgen::JsValue) -> Self {
    Self {
      context: "JsValue error".into(),
      error_type: AosAppErrorType::InternalClientError,
      description: format!("{:#?}", value),
    }
  }
}

#[cfg(not(feature = "ssr"))]
impl From<gloo_net::Error> for AosAppError {
  fn from(value: gloo_net::Error) -> Self {
    Self {
      context: "Gloo error".into(),
      error_type: AosAppErrorType::InternalServerError,
      description: format!("{:#?}", value),
    }
  }
}

// #[cfg(not(feature = "ssr"))]
// impl From<wasm_cookies::FromUrlEncodingError> for AosAppError {
//   fn from(value: wasm_cookies::FromUrlEncodingError) -> Self {
//     Self {
//       context: "WASM Cookie error".into(),
//       error_type: AosAppErrorType::InternalServerError,
//       description: format!("{:#?}", value),
//     }
//   }
// }

// #[cfg(feature = "ssr")]
// impl From<awc::error::JsonPayloadError> for AosAppError {
//   fn from(value: awc::error::JsonPayloadError) -> Self {
//     Self {
//       context: "JsonPayloadError error".into(),
//       error_type: AosAppErrorType::InternalServerError,
//       description: format!("{:#?}", value),
//     }
//   }
// }

// #[cfg(feature = "ssr")]
// impl From<awc::error::SendRequestError> for AosAppError {
//   fn from(value: awc::error::SendRequestError) -> Self {
//     use std::error::Error;
//     Self {
//       context: "SendRequestError error".into(),
//       error_type: AosAppErrorType::InternalServerError,
//       description: format!("{} - source: {:?}", value, value.source()),
//     }
//   }
// }

#[cfg(feature = "ssr")]
impl From<actix_http::error::PayloadError> for AosAppError {
  fn from(value: actix_http::error::PayloadError) -> Self {
    Self {
      context: "Payload error".into(),
      error_type: AosAppErrorType::InternalServerError,
      description: format!("{:#?}", value),
    }
  }
}

#[cfg(feature = "ssr")]
impl From<core::str::Utf8Error> for AosAppError {
  fn from(value: core::str::Utf8Error) -> Self {
    Self {
      context: "Utf8Error error".into(),
      error_type: AosAppErrorType::InternalServerError,
      description: format!("{:#?}", value),
    }
  }
}

#[cfg(feature = "ssr")]
impl From<ServerFnError> for AosAppError {
  fn from(value: ServerFnError) -> Self {
    Self {
      context: "ServerFnError error".into(),
      error_type: AosAppErrorType::InternalServerError,
      description: format!("{:#?}", value),
    }
  }
}
