use crate::prelude::*;
use core::num::ParseIntError;
use serde_urlencoded::ser;

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
  let i18n = use_i18n();

  let s = match &error.error_type {
    LemmyAppErrorType::ApiError(LemmyErrorType::IncorrectLogin) => t_string!(i18n, invalid_login).to_string(),
    LemmyAppErrorType::EmptyUsername => t_string!(i18n, empty_username).to_string(),
    LemmyAppErrorType::EmptyPassword => t_string!(i18n, empty_password).to_string(),
    LemmyAppErrorType::MissingReason => t_string!(i18n, empty_reason).to_string(),
    LemmyAppErrorType::MissingToken => t_string!(i18n, missing_token).to_string(),
    LemmyAppErrorType::InternalServerError => t_string!(i18n, internal).to_string(),
    LemmyAppErrorType::OfflineError => t_string!(i18n, offline).to_string(),
    LemmyAppErrorType::Unknown => t_string!(i18n, unknown).to_string(),
    _ => t_string!(i18n, error_without_description).to_string(),
  };

  leptos::logging::error!("{}\n{:#?}", s, error);

  s
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
