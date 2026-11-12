pub use crate::db::csr_indexed_db::{CommentDraft, CommentDraftKey, Draft, IndexedDb, IndexedDbError, ScrollPositionKey, Store};
pub use crate::icon::Icon;
pub use crate::{
  App, BanType, NotificationsRefresh, OnlineSetter, PassedUrl, ReadAuthCookie, ReadInstanceCookie, ReadThemeCookie, ResponseLoad, WriteAuthCookie,
  WriteInstanceCookie, WriteThemeCookie, html_template,
};
pub use crate::{
  client, client::*, comment, comment::*, db, errors, errors::*, hero, hero::*, icon, icon::*, listing, listing::*, login, nav, nav::*, overview,
  overview::*, post, post::*, search, search::*, toolbar, toolbar::*, user, user::*,
};
pub use codee::string::FromToStringCodec;
pub use lemmy_api_common::{
  LemmyErrorType, SuccessResponse, comment::*, community::*, lemmy_db_schema::aggregates::structs::*, lemmy_db_schema::newtypes::*,
  lemmy_db_schema::*, lemmy_db_views::structs::*, lemmy_db_views_actor::structs::*, person::*, post::*, private_message::*, site::*,
};
pub use leptos::config::LeptosOptions;
pub use leptos::{
  html::{Div, Img, Textarea},
  logging::*,
  prelude::*,
  task::*,
  *,
};
pub use leptos_dom::helpers::TimeoutHandle;
pub use leptos_meta::*;
pub use leptos_router::{components::*, hooks, hooks::*, location::State, *};
pub use leptos_use_cookie::*;
pub use send_wrapper::SendWrapper;
pub use serde::{Deserialize, Serialize, de::DeserializeOwned};
pub use std::collections::BTreeMap;
pub use web_sys::{
  Event, HtmlAnchorElement, HtmlImageElement, KeyboardEvent, MouseEvent, SubmitEvent, TouchEvent, VisibilityState, WheelEvent, wasm_bindgen::JsCast,
};

pub fn markdown_to_html(content: &str, linkify: bool) -> String {
  let mut options = pulldown_cmark::Options::empty();
  options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
  options.insert(pulldown_cmark::Options::ENABLE_TABLES);
  options.insert(pulldown_cmark::Options::ENABLE_SUPERSCRIPT);
  options.insert(pulldown_cmark::Options::ENABLE_SUBSCRIPT);
  options.insert(pulldown_cmark::Options::ENABLE_CONTAINER_EXTENSIONS);
  if linkify {
    options.insert(pulldown_cmark::Options::ENABLE_LINKIFY_LEMMY);
    options.insert(pulldown_cmark::Options::ENABLE_LINKIFY_HTTP);
  }
  markdown_to_html_with_options(content, options)
}

pub fn markdown_to_html_with_options(content: &str, options: pulldown_cmark::Options) -> String {
  let parser = pulldown_cmark::Parser::new_ext(content, options);

  let custom = parser.map(|event| match event {
    pulldown_cmark::Event::Html(text) => {
      let er = format!("<p>{}</p>", html_escape::encode_safe(&text).to_string());
      pulldown_cmark::Event::Html(er.into())
    }
    pulldown_cmark::Event::InlineHtml(text) => {
      let er = html_escape::encode_safe(&text).to_string();
      pulldown_cmark::Event::InlineHtml(er.into())
    }
    _ => event,
  });
  let mut safe_html = String::new();
  pulldown_cmark::html::push_html(&mut safe_html, custom);

  safe_html
}

fn InfoComponent(title: &str, content: &str) -> impl IntoView {
  view! {
    <div class="alert alert-info alert-soft">
      <div class="flex flex-col gap-2">
        <span class="font-bold">{title}</span>
        <span>{content}</span>
      </div>
    </div>
  }
}
