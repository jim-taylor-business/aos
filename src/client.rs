use crate::prelude::*;

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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GetSite {
  pub t: u64,
}

pub trait Fetch {
  async fn make_request<Response, Form>(
    &self,
    method: HttpType,
    path: &str,
    form: Form,
    cache_key: Option<GetCacheKey<Form>>,
  ) -> LemmyAppResult<Response>
  where
    Response: Serialize + DeserializeOwned + 'static + core::fmt::Debug,
    Form: Serialize + core::clone::Clone + 'static + core::fmt::Debug + Store;
}

pub trait LemmyApi: Fetch {
  async fn login(&self, form: Login) -> LemmyAppResult<LoginResponse> {
    let r = self.make_request(HttpType::Post, "user/login", form, None).await;
    r
  }

  async fn logout(&self) -> LemmyAppResult<SuccessResponse> {
    self.make_request(HttpType::Post, "user/logout", (), None).await
  }

  async fn list_communities(&self, key: GetCacheKey<ListCommunities>) -> LemmyAppResult<ListCommunitiesResponse> {
    self.make_request(HttpType::Get, "community/list", key.form.clone(), Some(key)).await
  }

  async fn follow_communities(&self, form: FollowCommunity) -> LemmyAppResult<CommunityResponse> {
    self.make_request(HttpType::Post, "community/follow", form, None).await
  }

  async fn get_community(&self, key: GetCacheKey<GetCommunity>) -> LemmyAppResult<GetCommunityResponse> {
    self.make_request(HttpType::Get, "community", key.form.clone(), Some(key)).await
  }

  async fn follow_community(&self, form: FollowCommunity) -> LemmyAppResult<CommunityResponse> {
    self.make_request(HttpType::Post, "community/follow", form, None).await
  }

  async fn get_comments(&self, key: GetCacheKey<GetComments>) -> LemmyAppResult<GetCommentsResponse> {
    self.make_request(HttpType::Get, "comment/list", key.form.clone(), Some(key)).await
  }

  async fn list_posts(&self, key: GetCacheKey<GetPosts>) -> LemmyAppResult<GetPostsResponse> {
    self.make_request(HttpType::Get, "post/list", key.form.clone(), Some(key)).await
  }

  async fn get_post(&self, key: GetCacheKey<GetPost>) -> LemmyAppResult<GetPostResponse> {
    self.make_request(HttpType::Get, "post", key.form.clone(), Some(key)).await
  }

  async fn get_site(&self, key: GetCacheKey<GetSite>) -> LemmyAppResult<GetSiteResponse> {
    self.make_request(HttpType::Get, "site", key.form.clone(), Some(key)).await
  }

  async fn report_post(&self, form: CreatePostReport) -> LemmyAppResult<PostReportResponse> {
    self.make_request(HttpType::Post, "post/report", form, None).await
  }

  async fn block_user(&self, form: BlockPerson) -> LemmyAppResult<BlockPersonResponse> {
    self.make_request(HttpType::Post, "user/block", form, None).await
  }

  async fn save_post(&self, form: SavePost) -> LemmyAppResult<PostResponse> {
    self.make_request(HttpType::Put, "post/save", form, None).await
  }

  async fn like_post(&self, form: CreatePostLike) -> LemmyAppResult<PostResponse> {
    self.make_request(HttpType::Post, "post/like", form, None).await
  }

  async fn like_comment(&self, form: CreateCommentLike) -> LemmyAppResult<CommentResponse> {
    self.make_request(HttpType::Post, "comment/like", form, None).await
  }

  async fn save_comment(&self, form: SaveComment) -> LemmyAppResult<CommentResponse> {
    self.make_request(HttpType::Put, "comment/save", form, None).await
  }

  async fn unread_count(&self, key: GetCacheKey<()>) -> LemmyAppResult<GetUnreadCountResponse> {
    self.make_request(HttpType::Get, "user/unread_count", key.form, Some(key)).await
  }

  async fn get_user(&self, key: GetCacheKey<GetPersonDetails>) -> LemmyAppResult<GetPersonDetailsResponse> {
    self.make_request(HttpType::Get, "user", key.form.clone(), Some(key)).await
  }

  async fn replies_user(&self, key: GetCacheKey<GetReplies>) -> LemmyAppResult<GetRepliesResponse> {
    self.make_request(HttpType::Get, "user/replies", key.form.clone(), Some(key)).await
  }

  async fn mention_user(&self, key: GetCacheKey<GetPersonMentions>) -> LemmyAppResult<GetPersonMentionsResponse> {
    self.make_request(HttpType::Get, "user/mention", key.form.clone(), Some(key)).await
  }

  async fn messages_user(&self, key: GetCacheKey<GetPrivateMessages>) -> LemmyAppResult<PrivateMessagesResponse> {
    self.make_request(HttpType::Get, "private_message/list", key.form.clone(), Some(key)).await
  }

  async fn mark_comment(&self, form: MarkCommentReplyAsRead) -> LemmyAppResult<CommentReplyResponse> {
    self.make_request(HttpType::Post, "comment/mark_as_read", form, None).await
  }

  async fn reply_comment(&self, form: CreateComment) -> LemmyAppResult<CommentResponse> {
    self.make_request(HttpType::Post, "comment", form, None).await
  }

  async fn edit_comment(&self, form: EditComment) -> LemmyAppResult<CommentResponse> {
    self.make_request(HttpType::Put, "comment", form, None).await
  }

  async fn search(&self, key: GetCacheKey<Search>) -> LemmyAppResult<SearchResponse> {
    self.make_request(HttpType::Get, "search", key.form.clone(), Some(key)).await
  }

  async fn get_comment(&self, key: GetCacheKey<GetComment>) -> LemmyAppResult<CommentResponse> {
    self.make_request(HttpType::Get, "comment", key.form.clone(), Some(key)).await
  }

  async fn get_mod_log(&self, key: GetCacheKey<GetModlog>) -> LemmyAppResult<GetModlogResponse> {
    self.make_request(HttpType::Get, "modlog", key.form.clone(), Some(key)).await
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
    async fn make_request<Response, Form>(
      &self,
      method: HttpType,
      path: &str,
      form: Form,
      _cache_key: Option<GetCacheKey<Form>>,
    ) -> LemmyAppResult<Response>
    where
      Response: Serialize + DeserializeOwned + 'static + core::fmt::Debug,
      Form: Serialize + core::clone::Clone + 'static + core::fmt::Debug + Store,
    {
      let route = build_route(path);
      let WriteAuthCookie(set_auth_cookie) = expect_context::<WriteAuthCookie>();
      let ReadAuthCookie(get_auth_cookie) = expect_context::<ReadAuthCookie>();
      let jwt = get_auth_cookie.get();

      log!("{}", format!("{}?{}", route, serde_urlencoded::to_string(&form).unwrap_or("".to_owned())));

      let client = reqwest::Client::builder().brotli(true).build().unwrap();

      let m = match method {
        HttpType::Get => client.get(&route).maybe_bearer_auth(jwt.clone()).query(&form).send(),
        HttpType::Post => client.post(&route).maybe_bearer_auth(jwt.clone()).json(&form).send(),
        HttpType::Put => client.put(&route).maybe_bearer_auth(jwt.clone()).json(&form).send(),
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
    async fn make_request<Response, Form>(
      &self,
      method: HttpType,
      path: &str,
      form: Form,
      cache_key: Option<GetCacheKey<Form>>,
    ) -> LemmyAppResult<Response>
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
                  if let Some(key) = &cache_key {
                    if let Ok(_c) = d.set(key, e).await {}
                  }
                }
              }
            }
            o
          }
        } else {
          if method == HttpType::Get {
            if let Ok(d) = IndexedDb::new().await {
              let cached = if let Some(key) = &cache_key { d.get::<GetCacheKey<Form>, Response>(key).await } else { Ok(None) };
              if let Ok(c) = cached {
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
