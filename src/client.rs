use crate::{
  ReadAuthCookie, ReadInstanceCookie, WriteAuthCookie, WriteInstanceCookie,
  db::csr_indexed_db::*,
  errors::{LemmyAppError, LemmyAppErrorType, LemmyAppResult},
};
use lemmy_api_common::{
  LemmyErrorType, SuccessResponse,
  comment::*,
  community::*,
  person::*,
  post::*,
  private_message::{GetPrivateMessages, PrivateMessagesResponse},
  site::*,
};
use leptos::{logging::log, prelude::*};
use send_wrapper::SendWrapper;
use serde::{Serialize, de::DeserializeOwned};
use std::str;

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
    Response: Serialize + DeserializeOwned + 'static + core::fmt::Debug,
    Form: Serialize + core::clone::Clone + 'static + core::fmt::Debug + Store;
}

pub trait LemmyApi: Fetch {
  async fn login(&self, form: Login) -> LemmyAppResult<LoginResponse> {
    let r = self.make_request(HttpType::Post, "user/login", form).await;
    r
  }

  async fn logout(&self) -> LemmyAppResult<SuccessResponse> {
    self.make_request(HttpType::Post, "user/logout", ()).await
  }

  async fn list_communities(&self, form: ListCommunities) -> LemmyAppResult<ListCommunitiesResponse> {
    self.make_request(HttpType::Get, "community/list", form).await
  }

  async fn get_community(&self, form: GetCommunity) -> LemmyAppResult<GetCommunityResponse> {
    self.make_request(HttpType::Get, "community", form).await
  }

  async fn follow_community(&self, form: FollowCommunity) -> LemmyAppResult<CommunityResponse> {
    self.make_request(HttpType::Post, "community/follow", form).await
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
    #[derive(Debug, Clone, Serialize)]
    struct GetSite {
      t: u64,
    };

    impl Store for GetSite {
      fn store_name(&self) -> &'static str {
        "query_gets"
      }
    }

    let now_in_millis = u64::try_from(jiff::Zoned::now().timestamp().as_millisecond()).unwrap_or(0);
    // let now_in_millis = {
    //   #[cfg(not(feature = "ssr"))]
    //   {
    //     chrono::offset::Utc::now().timestamp_millis() as u64
    //   }
    //   #[cfg(feature = "ssr")]
    //   {
    //     std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or(std::time::Duration::new(1000, 0)).as_millis() as u64
    //   }
    // };

    self.make_request(HttpType::Get, "site", GetSite { t: now_in_millis }).await
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

  async fn get_user(&self, form: GetPersonDetails) -> LemmyAppResult<GetPersonDetailsResponse> {
    self.make_request(HttpType::Get, "user", form).await
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
  let ReadInstanceCookie(get_instance_cookie) = expect_context::<ReadInstanceCookie>();
  let WriteInstanceCookie(set_instance_cookie) = expect_context::<WriteInstanceCookie>();
  if let Some(t) = get_instance_cookie.get() {
    set_instance_cookie.set(Some(t));
  } else {
    set_instance_cookie.set(Some("lemmy.world".to_owned()));
  }
  format!("https://{}/api/v3/{}", get_instance_cookie.get().unwrap_or("".to_owned()), route)
}

#[cfg(feature = "ssr")]
mod client {

  use super::*;

  trait MaybeBearerAuth {
    fn maybe_bearer_auth(self, token: Option<impl core::fmt::Display>) -> reqwest::RequestBuilder;
  }

  trait MaybeBlockingBearerAuth {
    fn maybe_bearer_auth(self, token: Option<impl core::fmt::Display>) -> reqwest::blocking::RequestBuilder;
  }

  impl MaybeBearerAuth for reqwest::RequestBuilder {
    fn maybe_bearer_auth(self, token: Option<impl core::fmt::Display>) -> reqwest::RequestBuilder {
      if let Some(token) = token { self.bearer_auth(token) } else { self }
    }
  }

  impl MaybeBlockingBearerAuth for reqwest::blocking::RequestBuilder {
    fn maybe_bearer_auth(self, token: Option<impl core::fmt::Display>) -> reqwest::blocking::RequestBuilder {
      if let Some(token) = token { self.bearer_auth(token) } else { self }
    }
  }

  impl Fetch for LemmyClient {
    async fn make_request<Response, Form>(&self, method: HttpType, path: &str, form: Form) -> LemmyAppResult<Response>
    where
      Response: Serialize + DeserializeOwned + 'static + core::fmt::Debug,
      Form: Serialize + core::clone::Clone + 'static + core::fmt::Debug + Store,
    {
      let WriteAuthCookie(set_auth_cookie) = expect_context::<WriteAuthCookie>();
      let ReadAuthCookie(get_auth_cookie) = expect_context::<ReadAuthCookie>();
      let jwt = get_auth_cookie.get();
      let route = build_route(path);

      log!("{}", format!("{}?{}", route, serde_urlencoded::to_string(&form).unwrap_or("".to_owned())));

      let client = reqwest::Client::builder().brotli(true).build().unwrap();

      let m = match method {
        HttpType::Get => client.get(&route).maybe_bearer_auth(jwt.clone()).query(&form).send(),
        HttpType::Post => client.post(&route).maybe_bearer_auth(jwt.clone()).form(&form).send(),
        HttpType::Put => client.put(&route).maybe_bearer_auth(jwt.clone()).form(&form).send(),
      }
      .await;

      match m {
        Err(re) => {
          return Err(LemmyAppError {
            error_type: LemmyAppErrorType::ApiError(LemmyErrorType::Unknown("reqwest error".into())),
            content: format!("{:#?}", re),
          });
        }
        Ok(r) => {
          match r.status().as_u16() {
            400..=599 => {
              let api_result = r.json::<LemmyErrorType>().await;

              match api_result {
                Ok(LemmyErrorType::IncorrectLogin) => {
                  log!("{:#?}", LemmyErrorType::IncorrectLogin);
                  set_auth_cookie.set(None);
                  return Err(LemmyAppError {
                    error_type: LemmyAppErrorType::ApiError(LemmyErrorType::IncorrectLogin),
                    content: format!("{:#?}", LemmyErrorType::IncorrectLogin),
                  });
                }
                Ok(le) => {
                  log!("{:#?}", le);
                  return Err(LemmyAppError { error_type: LemmyAppErrorType::ApiError(le.clone()), content: format!("{:#?}", le) });
                }
                Err(e) => {
                  log!("{:#?}", e);
                  return Err(LemmyAppError { error_type: LemmyAppErrorType::Unknown, content: format!("{:#?}", e) });
                }
              }
            }
            _ => {}
          };

          // Ok(r.json::<Response>().await.unwrap_or()) //.limit(10485760).await.map_err(Into::into)

          // let s = r.body().limit(10485760).await?;
          let t = r.text().await.ok().unwrap_or("".to_owned());
          let s = t; //.limit(10485760).await?;

          if s.len() == 0 {
            serde_json::from_str::<Response>("{}").map_err(Into::into)
          } else {
            serde_json::from_str::<Response>(&s).map_err(Into::into)
          }
        }
      }
    }
  }
}

#[cfg(not(feature = "ssr"))]
mod client {

  use super::*;
  use crate::OnlineSetter;
  use gloo_net::{http, http::RequestBuilder};
  use leptos::wasm_bindgen::UnwrapThrowExt;

  trait MaybeBearerAuth {
    fn maybe_bearer_auth(self, token: Option<&str>) -> Self;
  }

  impl MaybeBearerAuth for RequestBuilder {
    fn maybe_bearer_auth(self, token: Option<&str>) -> Self {
      if let Some(token) = token { self.header("Authorization", format!("Bearer {token}").as_str()) } else { self }
    }
  }

  impl Fetch for LemmyClient {
    async fn make_request<Response, Form>(&self, method: HttpType, path: &str, form: Form) -> LemmyAppResult<Response>
    where
      Response: Serialize + DeserializeOwned + 'static + core::fmt::Debug,
      Form: Serialize + core::clone::Clone + 'static + core::fmt::Debug + Store,
    {
      let route = &build_route(path);

      let WriteAuthCookie(set_auth_cookie) = expect_context::<WriteAuthCookie>();
      let ReadAuthCookie(get_auth_cookie) = expect_context::<ReadAuthCookie>();
      let jwt = get_auth_cookie.get();

      let online = expect_context::<RwSignal<OnlineSetter>>();

      let s = SendWrapper::new(async move {
        let abort_controller = SendWrapper::new(web_sys::AbortController::new().ok());
        let abort_signal = abort_controller.as_ref().map(|a| a.signal());
        on_cleanup(move || {
          if let Some(abort_controller) = abort_controller.take() {
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
                  log!("{:#?}", LemmyErrorType::IncorrectLogin);
                  set_auth_cookie.set(None);
                  return Err(LemmyAppError {
                    error_type: LemmyAppErrorType::ApiError(LemmyErrorType::IncorrectLogin),
                    content: format!("{:#?}", LemmyErrorType::IncorrectLogin),
                  });
                }
                Ok(le) => {
                  log!("{:#?}", le);
                  return Err(LemmyAppError { error_type: LemmyAppErrorType::ApiError(le.clone()), content: format!("{:#?}", le) });
                }
                Err(e) => {
                  log!("{:#?}", e);
                  return Err(LemmyAppError { error_type: LemmyAppErrorType::Unknown, content: format!("{:#?}", e) });
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
                if let Ok(d) = IndexedDb::new().await {
                  if let Ok(_c) = d.set(&form, &e).await {}
                }
              }
            }
            o
          }
        } else {
          if method == HttpType::Get {
            if let Ok(d) = IndexedDb::new().await {
              if let Ok(c) = d.get(&form).await {
                if let Some(o) = c {
                  return Ok(o);
                }
              }
            }
          }
          let e = LemmyAppError { error_type: LemmyAppErrorType::OfflineError, content: String::from("") };
          Err(e)
        }
      })
      .await;

      s
    }
  }

  fn build_fetch_query<T: Serialize>(path: &str, form: T) -> String {
    let form_str = serde_urlencoded::to_string(&form).unwrap_or("".to_owned());
    format!("{}?{}", build_route(path), form_str)
  }
}
