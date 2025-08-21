use std::str;

use crate::lemmy_error::LemmyErrorType;
use crate::{
  errors::{LemmyAppError, LemmyAppErrorType, LemmyAppResult},
  host::{get_host, get_https},
};
use codee::string::FromToStringCodec;
use lemmy_api_common::private_message::PrivateMessagesResponse;
use lemmy_api_common::SuccessResponse;
use lemmy_api_common::{comment::*, community::*, person::*, post::*, private_message::GetPrivateMessages, site::* /* , LemmyErrorType */};
use leptos::{Serializable, SignalGet, SignalSet};
use leptos_use::{use_cookie_with_options, SameSite, UseCookieOptions};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq)]
pub enum HttpType {
  #[allow(dead_code)]
  Get,
  #[allow(dead_code)]
  Post,
  #[allow(dead_code)]
  Put,
}

pub struct LemmyClient;

pub trait Fetch {
  async fn make_request<Response, Form>(&self, method: HttpType, path: &str, form: Form) -> LemmyAppResult<Response>
  where
    Response: Serializable + Serialize + for<'de> Deserialize<'de> + 'static,
    Form: Serialize + core::clone::Clone + 'static + core::fmt::Debug;
}

pub trait LemmyApi: Fetch {
  async fn login(&self, form: Login) -> LemmyAppResult<LoginResponse> {
    self.make_request(HttpType::Post, "user/login", form).await
  }

  async fn logout(&self) -> LemmyAppResult<SuccessResponse> {
    self.make_request(HttpType::Post, "user/logout", ()).await
  }

  async fn list_communities(&self, form: ListCommunities) -> LemmyAppResult<ListCommunitiesResponse> {
    self.make_request(HttpType::Get, "community/list", form).await
  }

  async fn get_comments(&self, form: GetComments) -> LemmyAppResult<GetCommentsResponse> {
    self.make_request(HttpType::Get, "comment/list", form).await
  }

  async fn list_posts(&self, form: GetPosts) -> LemmyAppResult<GetPostsResponse> {
    self.make_request(HttpType::Get, "post/list", form).await
  }

  async fn get_post(&self, form: GetPost) -> LemmyAppResult<GetPostResponse> {
    self.make_request(HttpType::Get, "post", form).await
  }

  async fn get_site(&self) -> LemmyAppResult<GetSiteResponse> {
    self.make_request(HttpType::Get, "site", ()).await
  }

  async fn report_post(&self, form: CreatePostReport) -> LemmyAppResult<PostReportResponse> {
    self.make_request(HttpType::Post, "post/report", form).await
  }

  async fn block_user(&self, form: BlockPerson) -> LemmyAppResult<BlockPersonResponse> {
    self.make_request(HttpType::Post, "user/block", form).await
  }

  async fn save_post(&self, form: SavePost) -> LemmyAppResult<PostResponse> {
    self.make_request(HttpType::Put, "post/save", form).await
  }

  async fn like_post(&self, form: CreatePostLike) -> LemmyAppResult<PostResponse> {
    self.make_request(HttpType::Post, "post/like", form).await
  }

  async fn like_comment(&self, form: CreateCommentLike) -> LemmyAppResult<CommentResponse> {
    self.make_request(HttpType::Post, "comment/like", form).await
  }

  async fn save_comment(&self, form: SaveComment) -> LemmyAppResult<CommentResponse> {
    self.make_request(HttpType::Put, "comment/save", form).await
  }

  async fn unread_count(&self) -> LemmyAppResult<GetUnreadCountResponse> {
    self.make_request(HttpType::Get, "user/unread_count", ()).await
  }

  async fn replies_user(&self, form: GetReplies) -> LemmyAppResult<GetRepliesResponse> {
    self.make_request(HttpType::Get, "user/replies", form).await
  }

  async fn mention_user(&self, form: GetPersonMentions) -> LemmyAppResult<GetPersonMentionsResponse> {
    self.make_request(HttpType::Get, "user/mention", form).await
  }

  async fn messages_user(&self, form: GetPrivateMessages) -> LemmyAppResult<PrivateMessagesResponse> {
    self.make_request(HttpType::Get, "private_message/list", form).await
  }

  async fn mark_comment(&self, form: MarkCommentReplyAsRead) -> LemmyAppResult<CommentReplyResponse> {
    self.make_request(HttpType::Post, "comment/mark_as_read", form).await
  }

  async fn reply_comment(&self, form: CreateComment) -> LemmyAppResult<CommentResponse> {
    self.make_request(HttpType::Post, "comment", form).await
  }

  async fn edit_comment(&self, form: EditComment) -> LemmyAppResult<CommentResponse> {
    self.make_request(HttpType::Put, "comment", form).await
  }

  async fn search(&self, form: Search) -> LemmyAppResult<SearchResponse> {
    self.make_request(HttpType::Get, "search", form).await
  }

  async fn get_comment(&self, form: GetComment) -> LemmyAppResult<CommentResponse> {
    self.make_request(HttpType::Get, "comment", form).await
  }

  async fn get_mod_log(&self, form: GetModlog) -> LemmyAppResult<GetModlogResponse> {
    self.make_request(HttpType::Get, "modlog", form).await
  }
}

impl LemmyApi for LemmyClient {}

fn build_route(route: &str) -> String {
  format!("http{}://{}/api/v3/{}", if get_https() == "true" { "s" } else { "" }, get_host(), route)
}

#[cfg(feature = "ssr")]
mod client {

  use super::*;
  use actix_web::web;
  use awc::{Client, ClientRequest};
  use leptos_actix::extract;

  trait MaybeBearerAuth {
    fn maybe_bearer_auth(self, token: Option<impl core::fmt::Display>) -> Self;
  }

  impl MaybeBearerAuth for ClientRequest {
    fn maybe_bearer_auth(self, token: Option<impl core::fmt::Display>) -> Self {
      if let Some(token) = token {
        self.bearer_auth(token)
      } else {
        self
      }
    }
  }

  impl Fetch for LemmyClient {
    async fn make_request<Response, Form>(&self, method: HttpType, path: &str, form: Form) -> LemmyAppResult<Response>
    where
      Response: Serializable + Serialize + for<'de> Deserialize<'de> + 'static,
      Form: Serialize + core::clone::Clone + 'static + core::fmt::Debug,
    {
      let (get_auth_cookie, set_auth_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
        "jwt",
        UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax),
      );
      let jwt = get_auth_cookie.get();
      let route = build_route(path);

      leptos::logging::log!(
        "{}",
        format!("{}?{}", route, serde_urlencoded::to_string(&form).unwrap_or("".to_string()))
      );
      let client = extract::<web::Data<Client>>().await?;

      let mut r = match method {
        HttpType::Get => client.get(&route).maybe_bearer_auth(jwt.clone()).query(&form)?.send(),
        HttpType::Post => client.post(&route).maybe_bearer_auth(jwt.clone()).send_json(&form),
        HttpType::Put => client.put(&route).maybe_bearer_auth(jwt.clone()).send_json(&form),
      }
      .await?;

      match r.status().as_u16() {
        400..=599 => {
          let api_result = r.json::<LemmyErrorType>().await;

          match api_result {
            Ok(LemmyErrorType::IncorrectLogin) => {
              set_auth_cookie.set(None);
              return Err(LemmyAppError {
                error_type: LemmyAppErrorType::ApiError(LemmyErrorType::IncorrectLogin),
                content: format!("{:#?}", LemmyErrorType::IncorrectLogin),
              });
            }
            Ok(le) => {
              return Err(LemmyAppError {
                error_type: LemmyAppErrorType::ApiError(le.clone()),
                content: format!("{:#?}", le),
              })
            }
            Err(e) => {
              return Err(LemmyAppError {
                error_type: LemmyAppErrorType::Unknown,
                content: format!("{:#?}", e),
              })
            }
          }
        }
        _ => {}
      };

      let s = r.body().limit(10485760).await?;

      if s.len() == 0 {
        serde_json::from_str::<Response>("{}").map_err(Into::into)
      } else {
        serde_json::from_str::<Response>(&str::from_utf8(&s)?).map_err(Into::into)
      }
    }
  }
}

#[cfg(not(feature = "ssr"))]
mod client {

  use super::*;
  use crate::indexed_db::csr_indexed_db::*;
  use crate::OnlineSetter;
  use gloo_net::{http, http::RequestBuilder};
  use leptos::wasm_bindgen::UnwrapThrowExt;
  use leptos::*;
  use leptos::{expect_context, RwSignal, SignalSet};
  use web_sys::AbortController;

  trait MaybeBearerAuth {
    fn maybe_bearer_auth(self, token: Option<&str>) -> Self;
  }

  impl MaybeBearerAuth for RequestBuilder {
    fn maybe_bearer_auth(self, token: Option<&str>) -> Self {
      if let Some(token) = token {
        self.header("Authorization", format!("Bearer {token}").as_str())
      } else {
        self
      }
    }
  }

  impl Fetch for LemmyClient {
    async fn make_request<Response, Form>(&self, method: HttpType, path: &str, form: Form) -> LemmyAppResult<Response>
    where
      Response: Serializable + Serialize + for<'de> Deserialize<'de> + 'static,
      Form: Serialize + core::clone::Clone + 'static + core::fmt::Debug,
    {
      let route = &build_route(path);
      let (get_auth_cookie, set_auth_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
        "jwt",
        UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax),
      );
      let jwt = get_auth_cookie.get();

      let online = expect_context::<RwSignal<OnlineSetter>>();

      let abort_controller = AbortController::new().ok();
      let abort_signal = abort_controller.as_ref().map(AbortController::signal);
      leptos::on_cleanup(move || {
        if let Some(abort_controller) = abort_controller {
          abort_controller.abort()
        }
      });

      if online.get().0 {
        let r = match method {
          HttpType::Get => http::Request::get(&build_fetch_query(path, form.clone()))
            .cache(web_sys::RequestCache::Default)
            .maybe_bearer_auth(jwt.as_deref())
            .abort_signal(abort_signal.as_ref())
            .build()
            .expect_throw("Could not parse query params"),
          HttpType::Post => http::Request::post(route)
            .maybe_bearer_auth(jwt.as_deref())
            .abort_signal(abort_signal.as_ref())
            .json(&form)
            .expect_throw("Could not parse json form"),
          HttpType::Put => http::Request::put(route)
            .maybe_bearer_auth(jwt.as_deref())
            .abort_signal(abort_signal.as_ref())
            .json(&form)
            .expect_throw("Could not parse json form"),
        }
        .send()
        .await?;

        match r.status() {
          400..=599 => {
            let api_result = r.json::<LemmyErrorType>().await;
            match api_result {
              Ok(LemmyErrorType::IncorrectLogin) => {
                set_auth_cookie.set(None);
                return Err(LemmyAppError {
                  error_type: LemmyAppErrorType::ApiError(LemmyErrorType::IncorrectLogin),
                  content: format!("{:#?}", LemmyErrorType::IncorrectLogin),
                });
              }
              Ok(le) => {
                return Err(LemmyAppError {
                  error_type: LemmyAppErrorType::ApiError(le.clone()),
                  content: format!("{:#?}", le),
                })
              }
              Err(e) => {
                return Err(LemmyAppError {
                  error_type: LemmyAppErrorType::Unknown,
                  content: format!("{:#?}", e),
                })
              }
            }
          }
          _ => {}
        };

        let t = r.text().await?;

        if t.is_empty() {
          serde_json::from_str::<Response>("{}").map_err(Into::into)
        } else {
          let o = serde_json::from_str::<Response>(&t).map_err(Into::into);
          if method == HttpType::Get {
            if let Ok(ref e) = o {
              if let Ok(d) = build_indexed_database().await {
                if let Ok(c) = set_query_get_cache(&d, &form, e).await {}
              }
            }
          }
          o
        }
      } else {
        if method == HttpType::Get {
          if let Ok(d) = build_indexed_database().await {
            if let Ok(c) = get_query_get_cache::<Form, Response>(&d, &form).await {
              if let Some(o) = c {
                return Ok(o);
              }
            }
          }
        }
        let e = LemmyAppError {
          error_type: LemmyAppErrorType::OfflineError,
          content: String::from(""),
        };
        Err(e)
      }
    }
  }

  fn build_fetch_query<T: Serialize>(path: &str, form: T) -> String {
    let form_str = serde_urlencoded::to_string(&form).unwrap_or("".to_string());
    format!("{}?{}", build_route(path), form_str)
  }
}
