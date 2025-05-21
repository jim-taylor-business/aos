use crate::config::{AOS_LEMMY_HOST, AOS_LEMMY_HTTPS};

#[cfg(feature = "ssr")]
pub fn get_ssr_host() -> String {
  std::env::var("AOS_LEMMY_HOST").unwrap_or_else(|_| AOS_LEMMY_HOST.into())
}

#[cfg(not(feature = "ssr"))]
pub fn get_csr_host() -> String {
  if let Some(s) = option_env!("AOS_LEMMY_HOST") {
    s.into()
  } else {
    AOS_LEMMY_HOST.into()
  }
}

pub fn get_host() -> String {
  #[cfg(feature = "ssr")]
  {
    get_ssr_host()
  }

  #[cfg(not(feature = "ssr"))]
  {
    get_csr_host()
  }
}

pub fn get_https() -> String {
  #[cfg(feature = "ssr")]
  {
    std::env::var("AOS_LEMMY_HTTPS").unwrap_or(format!("{AOS_LEMMY_HTTPS}"))
  }

  #[cfg(not(feature = "ssr"))]
  {
    if let Some(s) = option_env!("AOS_LEMMY_HTTPS") {
      s.into()
    } else {
      format!("{AOS_LEMMY_HTTPS}")
    }
  }
}
