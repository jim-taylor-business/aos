use std::str;

use crate::lemmy_error::LemmyErrorType;
use crate::{
  errors::{AosAppError, AosAppErrorType, AosAppResult},
  host::{get_host, get_https},
};
// use cfg_if::cfg_if;
use codee::string::FromToStringCodec;
use lemmy_api_common::private_message::PrivateMessagesResponse;
use lemmy_api_common::SuccessResponse;
use lemmy_api_common::{comment::*, community::*, person::*, post::*, private_message::GetPrivateMessages, site::* /* , LemmyErrorType */};
// use leptos::prelude::Get;
use leptos::{logging, prelude::*};
// use leptos_router::{components::*, hooks::*, *};
// use leptos::{Serializable, SignalGet};
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

// use std::str;

// use crate::lemmy_error::LemmyErrorType;
// use crate::{
//   errors::{LemmyAppError, LemmyAppErrorType, LemmyAppResult},
//   host::{get_host, get_https},
// };
// use cfg_if::cfg_if;
// use codee::string::FromToStringCodec;
// use lemmy_api_common::private_message::PrivateMessagesResponse;
// use lemmy_api_common::SuccessResponse;
// use lemmy_api_common::{comment::*, community::*, person::*, post::*, private_message::GetPrivateMessages, site::* /* , LemmyErrorType */};
// use leptos::{logging, Serializable, SignalGet};
// use leptos_use::{use_cookie_with_options, SameSite, UseCookieOptions};
// use serde::{Deserialize, Serialize};
// use web_sys::{RequestCache, RequestMode};

// #[derive(Clone, PartialEq)]
// pub enum HttpType {
//   #[allow(dead_code)]
//   Get,
//   #[allow(dead_code)]
//   Post,
//   #[allow(dead_code)]
//   Put,
// }

pub struct LemmyClient;

pub trait Fetch {
  async fn make_request<Response, Form>(&self, method: HttpType, path: &str, form: Form) -> AosAppResult<Response>
  where
    Response: Serialize + for<'de> Deserialize<'de> + 'static,
    Form: Serialize + core::clone::Clone + 'static + core::fmt::Debug;
}

pub trait LemmyApi: Fetch {
  async fn login(&self, form: Login) -> AosAppResult<LoginResponse> {
    self.make_request(HttpType::Post, "user/login", form).await
  }

  async fn logout(&self) -> AosAppResult<SuccessResponse> {
    self.make_request(HttpType::Post, "user/logout", ()).await
  }

  async fn list_communities(&self, form: ListCommunities) -> AosAppResult<ListCommunitiesResponse> {
    self.make_request(HttpType::Get, "community/list", form).await
  }

  async fn get_comments(&self, form: GetComments) -> AosAppResult<GetCommentsResponse> {
    self.make_request(HttpType::Get, "comment/list", form).await
  }

  async fn list_posts(&self, form: GetPosts) -> AosAppResult<GetPostsResponse> {
    self.make_request(HttpType::Get, "post/list", form).await
  }

  async fn get_post(&self, form: GetPost) -> AosAppResult<GetPostResponse> {
    self.make_request(HttpType::Get, "post", form).await
  }

  async fn get_site(&self) -> AosAppResult<GetSiteResponse> {
    self.make_request(HttpType::Get, "site", ()).await
  }

  async fn report_post(&self, form: CreatePostReport) -> AosAppResult<PostReportResponse> {
    self.make_request(HttpType::Post, "post/report", form).await
  }

  async fn block_user(&self, form: BlockPerson) -> AosAppResult<BlockPersonResponse> {
    self.make_request(HttpType::Post, "user/block", form).await
  }

  async fn save_post(&self, form: SavePost) -> AosAppResult<PostResponse> {
    self.make_request(HttpType::Put, "post/save", form).await
  }

  async fn like_post(&self, form: CreatePostLike) -> AosAppResult<PostResponse> {
    self.make_request(HttpType::Post, "post/like", form).await
  }

  async fn like_comment(&self, form: CreateCommentLike) -> AosAppResult<CommentResponse> {
    self.make_request(HttpType::Post, "comment/like", form).await
  }

  async fn save_comment(&self, form: SaveComment) -> AosAppResult<CommentResponse> {
    self.make_request(HttpType::Put, "comment/save", form).await
  }

  async fn unread_count(&self) -> AosAppResult<GetUnreadCountResponse> {
    self.make_request(HttpType::Get, "user/unread_count", ()).await
  }

  async fn replies_user(&self, form: GetReplies) -> AosAppResult<GetRepliesResponse> {
    self.make_request(HttpType::Get, "user/replies", form).await
  }

  async fn mention_user(&self, form: GetPersonMentions) -> AosAppResult<GetPersonMentionsResponse> {
    self.make_request(HttpType::Get, "user/mention", form).await
  }

  async fn messages_user(&self, form: GetPrivateMessages) -> AosAppResult<PrivateMessagesResponse> {
    self.make_request(HttpType::Get, "private_message/list", form).await
  }

  async fn mark_comment(&self, form: MarkCommentReplyAsRead) -> AosAppResult<CommentReplyResponse> {
    self.make_request(HttpType::Post, "comment/mark_as_read", form).await
  }

  async fn reply_comment(&self, form: CreateComment) -> AosAppResult<CommentResponse> {
    self.make_request(HttpType::Post, "comment", form).await
  }

  async fn edit_comment(&self, form: EditComment) -> AosAppResult<CommentResponse> {
    self.make_request(HttpType::Put, "comment", form).await
  }
}

impl LemmyApi for LemmyClient {}

fn build_route(route: &str) -> String {
  format!("http{}://{}/api/v3/{}", if get_https() == "true" { "s" } else { "" }, get_host(), route)
}

#[cfg(feature = "ssr")]
mod client {

  use super::*;

  use crate::OnlineSetter;
  use actix_web::web;
  // use awc::{Client, ClientRequest};
  use leptos_actix::extract;

  trait MaybeBearerAuth {
    fn maybe_bearer_auth(self, token: Option<impl core::fmt::Display>) -> Self;
  }

  // impl MaybeBearerAuth for ClientRequest {
  //   fn maybe_bearer_auth(self, token: Option<impl core::fmt::Display>) -> Self {
  //     if let Some(token) = token {
  //       self.bearer_auth(token)
  //     } else {
  //       self
  //     }
  //   }
  // }

  impl MaybeBearerAuth for reqwest::RequestBuilder {
    fn maybe_bearer_auth(self, token: Option<impl core::fmt::Display>) -> Self {
      if let Some(token) = token {
        self.bearer_auth(token)
      } else {
        self
      }
    }
  }

  impl Fetch for LemmyClient {
    async fn make_request<Response, Form>(&self, method: HttpType, path: &str, form: Form) -> AosAppResult<Response>
    where
      Response: Serialize + for<'de> Deserialize<'de> + 'static,
      Form: Serialize + core::clone::Clone + 'static + core::fmt::Debug,
    {
      let (get_auth_cookie, _) = use_cookie_with_options::<String, FromToStringCodec>(
        "jwt",
        UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax),
      );
      let jwt = get_auth_cookie.get();
      let route = build_route(path);

      logging::log!(
        "{}",
        format!("{}?{}", route, serde_urlencoded::to_string(&form).unwrap_or("".to_string()))
      );

      // let client = extract::<web::Data<Client>>().await?;

      // let mut r = match method {
      //   HttpType::Get => client.get(&route).maybe_bearer_auth(jwt.clone()).query(&form)?.send(),
      //   HttpType::Post => client.post(&route).maybe_bearer_auth(jwt.clone()).send_json(&form),
      //   HttpType::Put => client.put(&route).maybe_bearer_auth(jwt.clone()).send_json(&form),
      // }
      // .await?;

      let client = reqwest::Client::new();

      let mut r = match method {
        HttpType::Get => client.get(&route).maybe_bearer_auth(jwt.clone()).form(&form).send(), //reqwest::get(&route), //client.get(&route).maybe_bearer_auth(jwt.clone()).query(&form).unwrap().send(),
        HttpType::Post => client.post(&route).maybe_bearer_auth(jwt.clone()).form(&form).send(),
        HttpType::Put => client.put(&route).maybe_bearer_auth(jwt.clone()).form(&form).send(), //client.put(&route).maybe_bearer_auth(jwt.clone()).send_json(&form),
      }
      .await
      .unwrap();

      logging::log!("1");

      match r.status().as_u16() {
        400..=599 => {
          let api_result = r.json::<LemmyErrorType>().await;

          match api_result {
            Ok(le) => {
              return Err(AosAppError {
                error_type: AosAppErrorType::ApiError(le.clone()),
                context: format!("{:#?}", le),
                description: "".into(),
              })
            }
            Err(e) => {
              return Err(AosAppError {
                error_type: AosAppErrorType::Unknown,
                context: format!("{:#?}", e),
                description: "".into(),
              })
            }
          }
        }
        _ => {}
      };

      logging::log!("2");

      // Ok(r.json::<Response>().await.unwrap()) //.limit(10485760).await.map_err(Into::into)

      // let s = r.body().limit(10485760).await?;
      let t = r.text().await.ok().unwrap_or("".to_string());
      let s = t; //.limit(10485760).await?;

      logging::log!("3");

      if s.len() == 0 {
        serde_json::from_str::<Response>("{}").map_err(Into::into)
      } else {
        // serde_json::from_str::<Response>(&str::from_utf8(&s)?).map_err(Into::into)
        serde_json::from_str::<Response>(&s).map_err(Into::into)
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
  // use leptos::{expect_context, RwSignal, SignalSet};
  use leptos::{html::*, *};
  use web_sys::{AbortController, RequestCache};

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
    async fn make_request<Response, Form>(&self, method: HttpType, path: &str, form: Form) -> AosAppResult<Response>
    where
      Response: Serialize + for<'de> Deserialize<'de> + 'static,
      Form: Serialize + core::clone::Clone + 'static + core::fmt::Debug,
    {
      let route = &build_route(path);
      let (get_auth_cookie, _) = use_cookie_with_options::<String, FromToStringCodec>(
        "jwt",
        UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax),
      );
      let jwt = get_auth_cookie.get();

      send_wrapper::SendWrapper::new(async move {
        let online = expect_context::<RwSignal<OnlineSetter>>();

        let abort_controller = AbortController::new().ok();
        let abort_signal = abort_controller.as_ref().map(AbortController::signal);
        // leptos::on_cleanup(move || {
        //   if let Some(abort_controller) = abort_controller {
        //     abort_controller.abort()
        //   }
        // });

        if online.get().0 {
          //   let result = http::Request::get(&build_fetch_query(path, form))
          //     .cache(RequestCache::Default)
          //     .maybe_bearer_auth(jwt.as_deref())
          //     .abort_signal(abort_signal.as_ref())
          //     .build()
          //     .expect_throw("Could not parse query params")
          //     .send()
          //     .await;
          //   result?
          // } else {
          // }
          let mut r = match method {
            HttpType::Get => http::Request::get(&build_fetch_query(path, form.clone()))
              .cache(RequestCache::Default)
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
                  let authenticated = expect_context::<RwSignal<Option<bool>>>();
                  authenticated.set(Some(false));
                  return Err(AosAppError {
                    error_type: AosAppErrorType::ApiError(LemmyErrorType::IncorrectLogin),
                    context: format!("{:#?}", LemmyErrorType::IncorrectLogin),
                    description: "".into(),
                  });
                }
                Ok(le) => {
                  return Err(AosAppError {
                    error_type: AosAppErrorType::ApiError(le.clone()),
                    context: format!("{:#?}", le),
                    description: "".into(),
                  })
                }
                Err(e) => {
                  return Err(AosAppError {
                    error_type: AosAppErrorType::Unknown,
                    context: format!("{:#?}", e),
                    description: "".into(),
                  })
                }
              }
            }
            _ => {
              // match result {
              //   Ok(o) => {
              // if let Ok(Some(s)) = window().local_storage() {
              //   if let Ok(Some(_)) = s.get_item(&serde_json::to_string(&form).ok().unwrap()) {}
              //   let _ = s.set_item(&serde_json::to_string(&form).ok().unwrap(), &serde_json::to_string(&o).ok().unwrap());
              // }
              //     return Ok(o);
              //   }
              //   Err(e) => {
              //     return Err(e);
              //   }
              // }
            }
          };

          let t = r.text().await?;

          if method == HttpType::Get {
            if let Ok(Some(s)) = window().local_storage() {
              // if let Ok(Some(_)) = s.get_item(&serde_json::to_string(&form).ok().unwrap()) {}
              let _ = s.set_item(&serde_json::to_string(&form).ok().unwrap(), &t);
            }
          }

          if t.is_empty() {
            serde_json::from_str::<Response>("{}").map_err(Into::into)
            // // Ok(()::Response)
          } else {
            serde_json::from_str::<Response>(&t).map_err(Into::into)
            // r.json::<Response>().await.map_err(Into::into)
          }
        } else {
          if method == HttpType::Get {
            if let Ok(Some(s)) = window().local_storage() {
              // logging::log!("off {:#?}", &form);
              if let Ok(Some(c)) = s.get_item(&serde_json::to_string(&form).ok().unwrap()) {
                if let Ok(o) = serde_json::from_str::<Response>(&c) {
                  return Ok(o);
                }
              }
            }
          }
          let e = AosAppError {
            error_type: AosAppErrorType::OfflineError,
            context: String::from(""),
            description: "".into(),
          };
          Err(e)
        }
      })
      .await
    }
  }

  fn build_fetch_query<T: Serialize>(path: &str, form: T) -> String {
    let form_str = serde_urlencoded::to_string(&form).unwrap_or("".to_string());
    format!("{}?{}", build_route(path), form_str)
  }
}

// use crate::lemmy_error::LemmyErrorType;
// use crate::{
//   errors::{AosAppError, AosAppErrorType, AosAppResult},
//   host::{get_host, get_https},
// };
// // use cfg_if::cfg_if;
// use codee::string::FromToStringCodec;
// use lemmy_api_common::private_message::PrivateMessagesResponse;
// use lemmy_api_common::{comment::*, community::*, person::*, post::*, private_message::GetPrivateMessages, site::* /* , LemmyErrorType */};
// use leptos::prelude::Get;
// // use leptos::{logging, prelude::*};
// // use leptos_router::{components::*, hooks::*, *};
// // use leptos::{Serializable, SignalGet};
// use leptos_use::{use_cookie_with_options, SameSite, UseCookieOptions};
// use serde::{Deserialize, Serialize};

// #[derive(Clone)]
// pub enum HttpType {
//   #[allow(dead_code)]
//   Get,
//   #[allow(dead_code)]
//   Post,
//   #[allow(dead_code)]
//   Put,
// }

// pub struct LemmyClient;

// // mod fetch {
// //   use super::*;

// pub trait Fetch {
//   async fn make_request<Response, Form>(&self, method: HttpType, path: &str, form: Form) -> AosAppResult<Response>
//   where
//     Response: Serialize + for<'de> Deserialize<'de> + 'static,
//     Form: Serialize + core::clone::Clone + 'static + core::fmt::Debug;
// }

// pub trait LemmyApi: Fetch {
//   async fn test(&self, form: SavePost, ok: impl FnMut() -> (), err: impl FnMut() -> ()) -> AosAppResult<LoginResponse> {
//     let result = self.make_request(HttpType::Put, "post/save", form).await;
//     result
//   }

//   async fn login(&self, form: Login) -> AosAppResult<LoginResponse> {
//     leptos::logging::log!("B7");
//     self.make_request(HttpType::Post, "user/login", form).await
//   }

//   async fn logout(&self) -> AosAppResult<()> {
//     let _ = self.make_request::<(), ()>(HttpType::Post, "user/logout", ()).await;
//     // TODO: do not ignore error due to not being able to decode empty http response cleanly
//     Ok(())
//   }

//   async fn list_communities(&self, form: ListCommunities) -> AosAppResult<ListCommunitiesResponse> {
//     self.make_request(HttpType::Get, "community/list", form).await
//   }

//   async fn get_comments(&self, form: GetComments) -> AosAppResult<GetCommentsResponse> {
//     self.make_request(HttpType::Get, "comment/list", form).await
//   }

//   async fn list_posts(&self, form: GetPosts) -> AosAppResult<GetPostsResponse> {
//     self.make_request(HttpType::Get, "post/list", form).await
//   }

//   async fn get_post(&self, form: GetPost) -> AosAppResult<GetPostResponse> {
//     self.make_request(HttpType::Get, "post", form).await
//   }

//   async fn get_site(&self) -> AosAppResult<GetSiteResponse> {
//     self.make_request(HttpType::Get, "site", ()).await
//   }

//   async fn report_post(&self, form: CreatePostReport) -> AosAppResult<PostReportResponse> {
//     self.make_request(HttpType::Post, "post/report", form).await
//   }

//   async fn block_user(&self, form: BlockPerson) -> AosAppResult<BlockPersonResponse> {
//     self.make_request(HttpType::Post, "user/block", form).await
//   }

//   async fn save_post(&self, form: SavePost) -> AosAppResult<PostResponse> {
//     self.make_request(HttpType::Put, "post/save", form).await
//   }

//   async fn like_post(&self, form: CreatePostLike) -> AosAppResult<PostResponse> {
//     self.make_request(HttpType::Post, "post/like", form).await
//   }

//   async fn like_comment(&self, form: CreateCommentLike) -> AosAppResult<CommentResponse> {
//     self.make_request(HttpType::Post, "comment/like", form).await
//   }

//   async fn save_comment(&self, form: SaveComment) -> AosAppResult<CommentResponse> {
//     self.make_request(HttpType::Put, "comment/save", form).await
//   }

//   async fn unread_count(&self) -> AosAppResult<GetUnreadCountResponse> {
//     self.make_request(HttpType::Get, "user/unread_count", ()).await
//   }

//   async fn replies_user(&self, form: GetReplies) -> AosAppResult<GetRepliesResponse> {
//     self.make_request(HttpType::Get, "user/replies", form).await
//   }

//   async fn mention_user(&self, form: GetPersonMentions) -> AosAppResult<GetPersonMentionsResponse> {
//     self.make_request(HttpType::Get, "user/mention", form).await
//   }

//   async fn messages_user(&self, form: GetPrivateMessages) -> AosAppResult<PrivateMessagesResponse> {
//     self.make_request(HttpType::Get, "private_message/list", form).await
//   }

//   async fn mark_comment(&self, form: MarkCommentReplyAsRead) -> AosAppResult<CommentReplyResponse> {
//     self.make_request(HttpType::Post, "comment/mark_as_read", form).await
//   }

//   async fn reply_comment(&self, form: CreateComment) -> AosAppResult<CommentResponse> {
//     self.make_request(HttpType::Post, "comment", form).await
//   }

//   async fn edit_comment(&self, form: EditComment) -> AosAppResult<CommentResponse> {
//     self.make_request(HttpType::Put, "comment", form).await
//   }
// }
// // }
// impl LemmyApi for LemmyClient {}

// // use client::*;

// fn build_route(route: &str) -> String {
//   format!("http{}://{}/api/v3/{}", if get_https() == "true" { "s" } else { "" }, get_host(), route)
// }

// #[cfg(feature = "ssr")]
// mod client {

//   // use super::fetch::*;
//   use super::*;
//   use actix_web::web;
//   // use awc::{Client, ClientRequest};
//   use leptos_actix::extract;
//   // use reqwest::Client;

//   trait MaybeBearerAuth {
//     fn maybe_bearer_auth(self, token: Option<impl core::fmt::Display>) -> Self;
//   }

//   // impl MaybeBearerAuth for ClientRequest {
//   //   fn maybe_bearer_auth(self, token: Option<impl core::fmt::Display>) -> Self {
//   //     if let Some(token) = token {
//   //       self.bearer_auth(token)
//   //     } else {
//   //       self
//   //     }
//   //   }
//   // }

//   impl MaybeBearerAuth for reqwest::RequestBuilder {
//     fn maybe_bearer_auth(self, token: Option<impl core::fmt::Display>) -> Self {
//       if let Some(token) = token {
//         self.bearer_auth(token)
//       } else {
//         self
//       }
//     }
//   }

//   impl Fetch for LemmyClient {
//     async fn make_request<Response, Form>(&self, method: HttpType, path: &str, form: Form) -> AosAppResult<Response>
//     where
//       Response: Serialize + for<'de> Deserialize<'de> + 'static,
//       Form: Serialize + core::clone::Clone + 'static + core::fmt::Debug,
//     {
//       let (get_auth_cookie, _) = use_cookie_with_options::<String, FromToStringCodec>(
//         "jwt",
//         UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax),
//       );
//       let jwt = get_auth_cookie.get();
//       let route = build_route(path);

//       // leptos::logging::log!("B10");

//       // // send_wrapper::SendWrapper::new(async move {
//       leptos::logging::log!(
//         "{}",
//         format!("{}?{}", route, serde_urlencoded::to_string(&form).unwrap_or("".to_string()))
//       );
//       // let client = extract::<web::Data<Client>>().await?;

//       // let mut r = send_wrapper::SendWrapper::new(async move {
//       //   match method {
//       //     HttpType::Get => client.get(&route).maybe_bearer_auth(jwt.clone()).query(&form).unwrap().send(),
//       //     HttpType::Post => client.post(&route).maybe_bearer_auth(jwt.clone()).send_json(&form),
//       //     HttpType::Put => client.put(&route).maybe_bearer_auth(jwt.clone()).send_json(&form),
//       //   }
//       //   .await
//       // })
//       // .await?;

//       let client = reqwest::Client::new();

//       let mut r = match method {
//         HttpType::Get => client.get(&route).maybe_bearer_auth(jwt.clone()).form(&form).send(), //reqwest::get(&route), //client.get(&route).maybe_bearer_auth(jwt.clone()).query(&form).unwrap().send(),
//         HttpType::Post => client.post(&route).maybe_bearer_auth(jwt.clone()).form(&form).send(),
//         HttpType::Put => client.put(&route).maybe_bearer_auth(jwt.clone()).form(&form).send(), //client.put(&route).maybe_bearer_auth(jwt.clone()).send_json(&form),
//       }
//       .await
//       .unwrap();

//       match r.status().as_u16() {
//         400..=599 => {
//           let api_result = //send_wrapper::SendWrapper::new(async move {
//             r.json::<LemmyErrorType>().await;
//           // }).await;

//           match api_result {
//             Ok(le) => {
//               return Err(AosAppError {
//                 context: "LemmyClient server error".into(),
//                 error_type: AosAppErrorType::ApiError(le.clone()),
//                 description: format!("{:#?}", le),
//               })
//             }
//             Err(e) => {
//               return Err(AosAppError {
//                 context: "LemmyClient server error".into(),
//                 error_type: AosAppErrorType::Unknown,
//                 description: format!("{:#?}", e),
//               })
//             }
//           }
//         }
//         _ => {}
//       };
//       Ok(r.json::<Response>().await.unwrap()) //.limit(10485760).await.map_err(Into::into)

//       // match r.status().as_u16() {
//       //   400..=599 => {
//       //     let api_result = //send_wrapper::SendWrapper::new(async move {
//       //       r.json::<LemmyErrorType>().await;
//       //     // }).await;

//       //     match api_result {
//       //       Ok(le) => {
//       //         return Err(AosAppError {
//       //           context: "LemmyClient server error".into(),
//       //           error_type: AosAppErrorType::ApiError(le.clone()),
//       //           description: format!("{:#?}", le),
//       //         })
//       //       }
//       //       Err(e) => {
//       //         return Err(AosAppError {
//       //           context: "LemmyClient server error".into(),
//       //           error_type: AosAppErrorType::Unknown,
//       //           description: format!("{:#?}", e),
//       //         })
//       //       }
//       //     }
//       //   }
//       //   _ => {}
//       // };
//       // r.json::<Response>().limit(10485760).await.map_err(Into::into)
//       // })
//       // .await

//       // Err(AosAppError {
//       //   context: "".into(),
//       //   description: "".into(),
//       //   error_type: AosAppErrorType::Unknown,
//       // })
//     }
//   }
// }

// #[cfg(not(feature = "ssr"))]
// mod client {

//   use super::*;
//   use gloo_net::{http, http::RequestBuilder};
//   use leptos::wasm_bindgen::UnwrapThrowExt;
//   use web_sys::AbortController;
//   // use leptos::SignalSet;

//   trait MaybeBearerAuth {
//     fn maybe_bearer_auth(self, token: Option<&str>) -> Self;
//   }

//   impl MaybeBearerAuth for RequestBuilder {
//     fn maybe_bearer_auth(self, token: Option<&str>) -> Self {
//       if let Some(token) = token {
//         self.header("Authorization", format!("Bearer {token}").as_str())
//       } else {
//         self
//       }
//     }
//   }

//   impl Fetch for LemmyClient {
//     async fn make_request<Response, Form>(&self, method: HttpType, path: &str, form: Form) -> AosAppResult<Response>
//     where
//       Response: Serialize + for<'de> Deserialize<'de> + 'static,
//       Form: Serialize + core::clone::Clone + 'static + core::fmt::Debug,
//     {
//       let route = &build_route(path);
//       let (get_auth_cookie, _) = use_cookie_with_options::<String, FromToStringCodec>(
//         "jwt",
//         UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax),
//       );
//       let jwt = get_auth_cookie.get();

//       send_wrapper::SendWrapper::new(async move {
//         // let abort_controller = AbortController::new().ok();
//         // let abort_signal = abort_controller.as_ref().map(AbortController::signal);
//         // leptos::prelude::on_cleanup(move || {
//         //   if let Some(abort_controller) = abort_controller {
//         //     abort_controller.abort()
//         //   }
//         // });

//         let r = match method {
//           HttpType::Get => http::Request::get(&build_fetch_query(path, form))
//             .maybe_bearer_auth(jwt.as_deref())
//             // .abort_signal(abort_signal.as_ref())
//             .build()
//             .expect_throw("Could not parse query params"),
//           HttpType::Post => http::Request::post(route)
//             .maybe_bearer_auth(jwt.as_deref())
//             // .abort_signal(abort_signal.as_ref())
//             .json(&form)
//             .expect_throw("Could not parse json form"),
//           HttpType::Put => http::Request::put(route)
//             .maybe_bearer_auth(jwt.as_deref())
//             // .abort_signal(abort_signal.as_ref())
//             .json(&form)
//             .expect_throw("Could not parse json form"),
//         }
//         .send()
//         .await?;

//         match r.status() {
//           400..=599 => {
//             let api_result = r.json::<LemmyErrorType>().await;
//             match api_result {
//               Ok(LemmyErrorType::IncorrectLogin) => {
//                 use leptos::prelude::Set;
//                 let authenticated = leptos::prelude::expect_context::<leptos::prelude::RwSignal<Option<bool>>>();
//                 authenticated.set(Some(false));
//                 return Err(AosAppError {
//                   context: "LemmyClient IncorrectLogin error".into(),
//                   error_type: AosAppErrorType::ApiError(LemmyErrorType::IncorrectLogin),
//                   description: format!("{:#?}", LemmyErrorType::IncorrectLogin),
//                 });
//               }
//               Ok(le) => {
//                 return Err(AosAppError {
//                   context: "LemmyClient client error".into(),
//                   error_type: AosAppErrorType::ApiError(le.clone()),
//                   description: format!("{:#?}", le),
//                 })
//               }
//               Err(e) => {
//                 return Err(AosAppError {
//                   context: "LemmyClient client error".into(),
//                   error_type: AosAppErrorType::Unknown,
//                   description: format!("{:#?}", e),
//                 })
//               }
//             }
//           }
//           _ => {}
//         };

//         r.json::<Response>().await.map_err(Into::into)
//       })
//       .await
//     }
//   }

//   fn build_fetch_query<T: Serialize>(path: &str, form: T) -> String {
//     let form_str = serde_urlencoded::to_string(&form).unwrap_or("".to_string());
//     format!("{}?{}", build_route(path), form_str)
//   }
// }
