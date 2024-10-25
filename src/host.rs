use crate::config::{AOS_LEMMY_HOST, AOS_LEMMY_HTTPS};
use cfg_if::cfg_if;

#[cfg(feature = "ssr")]
pub fn get_internal_host() -> String {
  std::env::var("AOS_LEMMY_HOST").unwrap_or_else(|_| AOS_LEMMY_HOST.into())
}

#[cfg(not(feature = "ssr"))]
pub fn get_external_host() -> String {
  if let Some(s) = option_env!("AOS_LEMMY_HOST") {
    s.into()
  } else {
    AOS_LEMMY_HOST.into()
  }
}

pub fn get_host() -> String {
  cfg_if! {
    if #[cfg(feature="ssr")] {
      get_internal_host()
    } else {
      get_external_host()
    }
  }
}

pub fn get_https() -> String {
  cfg_if! {
    if #[cfg(feature="ssr")] {
      std::env::var("AOS_LEMMY_HTTPS").unwrap_or(format!("{AOS_LEMMY_HTTPS}"))
    } else {
      if let Some(s) = option_env!("AOS_LEMMY_HTTPS") {
        s.into()
      } else {
        format!("{AOS_LEMMY_HTTPS}")
      }
    }
  }
}
