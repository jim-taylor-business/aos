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

pub fn relative_time(published_at_millis: i64) -> String {
  let now_in_millis = jiff::Zoned::now().timestamp().as_millisecond();
  let difference = now_in_millis.saturating_sub(published_at_millis);
  let is_future = difference < 0;
  let elapsed_millis = difference.unsigned_abs();
  let duration = pretty_duration::pretty_duration(
    &std::time::Duration::from_millis(elapsed_millis),
    Some(pretty_duration::PrettyDurationOptions {
      output_format: Some(pretty_duration::PrettyDurationOutputFormat::Compact),
      singular_labels: None,
      plural_labels: None,
    }),
  );
  let abbreviated = if let Some((index, _)) = duration.match_indices(' ').nth(1) { duration[..index].to_owned() } else { duration };
  if is_future { format!("in {}", abbreviated) } else { format!("{} ago", abbreviated) }
}

pub fn community_title_with_federation(name: &str, local: bool, host: Option<String>) -> String {
  if local { name.to_owned() } else { format!("{}@{}", name, host.unwrap_or_default()) }
}

pub fn federated_creator_handle(name: &str, domain: Option<&str>, instance: Option<&str>) -> String {
  match domain {
    Some(domain) if instance != Some(domain) => format!("{}@{}", name, domain),
    _ => name.to_owned(),
  }
}

pub fn external_host_attribution(host: Option<String>, instance: Option<String>) -> String {
  match host {
    Some(host) if Some(host.as_str()) != instance.as_deref() => format!(" from {}", host),
    _ => String::new(),
  }
}

#[component]
pub fn Warning(
  #[prop(optional, into)] description: Option<String>,
  #[prop(optional, into)] on_retry_click: Option<Callback<MouseEvent>>,
) -> impl IntoView {
  let description_value = description.unwrap_or("Offline".to_owned());
  view! {
    <div class="py-4 px-8 break-inside-avoid">
      <div class="flex justify-between items-center alert alert-warning alert-soft">
        <span>{description_value}</span>
        {if let Some(o) = on_retry_click {
          view! {
            <span
              on:click={move |e: MouseEvent| {
                o.run(e);
              }}
              class="btn btn-sm"
            >
              "Retry"
            </span>
          }
            .into_any()
        } else {
          view! {}.into_any()
        }}
      </div>
    </div>
  }
  .into_any()
}

#[component]
pub fn Error(
  #[prop(optional, into)] get_url: Option<bool>,
  #[prop(optional, into)] description: Option<String>,
  error: LemmyAppError,
  #[prop(optional, into)] on_retry_click: Option<Callback<MouseEvent>>,
) -> impl IntoView {
  error!("{:#?}", error);
  let description_value = description.unwrap_or("Error".to_owned());
  let location = use_location();
  let search = location.search.get();
  view! {
    <div class="py-4 px-8 break-inside-avoid">
      <div class="flex alert alert-error alert-soft">
        <details class="w-full min-w-0">
          <summary class="flex justify-between items-center list-none">
            <span>{description_value}</span>
            {if let Some(o) = on_retry_click {
              {
                if let Some(true) = get_url {
                  view! {
                    <a
                      href={format!("{}{}", location.pathname.get(), if search.len() > 0 { format!("?{}", search) } else { "".into() })}
                      on:click={move |e: MouseEvent| {
                        e.prevent_default();
                        o.run(e);
                      }}
                      class="btn btn-sm"
                    >
                      "Retry"
                    </a>
                  }
                    .into_any()
                } else {
                  view! {
                    <span
                      on:click={move |e: MouseEvent| {
                        e.prevent_default();
                        o.run(e);
                      }}
                      class="btn btn-sm"
                    >
                      "Retry"
                    </span>
                  }
                    .into_any()
                }
              }
            } else {
              {
                if let Some(true) = get_url {
                  view! {
                    <a
                      href={format!("{}{}", location.pathname.get(), if search.len() > 0 { format!("?{}", search) } else { "".into() })}
                      class="btn btn-sm"
                    >
                      "Retry"
                    </a>
                  }
                    .into_any()
                } else {
                  view! {}.into_any()
                }
              }
            }}
          </summary>
          <div class="overflow-auto">
            <pre>{format!("{:#?}", error.error_type)}</pre>
            <pre>{error.content}</pre>
          </div>
        </details>
      </div>
    </div>
  }
  .into_any()
}

#[component]
pub fn Info(loading: bool, #[prop(optional, into)] description: Option<String>) -> impl IntoView {
  let description_value = description.unwrap_or("Loading...".to_owned());
  if loading {
    view! {
      <div class="overflow-hidden break-inside-avoid animate-[popdown_1s_step-end_1]">
        <div class="py-4 px-8">
          <div class="alert alert-info alert-soft">
            <span>{description_value}</span>
          </div>
        </div>
      </div>
    }
    .into_any()
  } else {
    view! {}.into_any()
  }
}

pub fn default_cookie_options<T, E, D>() -> UseCookieOptions<T, E, D> {
  UseCookieOptions::default().max_age(691200000).path("/").same_site(SameSite::Lax)
}

pub fn provide_auth_cookie() -> (Signal<Option<String>>, WriteSignal<Option<String>>) {
  let (get_auth_cookie, set_auth_cookie) = use_cookie_with_options::<String, FromToStringCodec>("jwt", default_cookie_options());
  provide_context(ReadAuthCookie(get_auth_cookie));
  provide_context(WriteAuthCookie(set_auth_cookie));
  (get_auth_cookie, set_auth_cookie)
}

pub fn provide_instance_cookie() -> (Signal<Option<String>>, WriteSignal<Option<String>>) {
  let (get_instance_cookie, set_instance_cookie) = use_cookie_with_options::<String, FromToStringCodec>("instance", default_cookie_options());
  provide_context(ReadInstanceCookie(get_instance_cookie));
  provide_context(WriteInstanceCookie(set_instance_cookie));
  (get_instance_cookie, set_instance_cookie)
}

pub fn provide_theme_cookie() -> (Signal<Option<String>>, WriteSignal<Option<String>>) {
  let (get_theme_cookie, set_theme_cookie) = use_cookie_with_options::<String, FromToStringCodec>("theme", default_cookie_options());
  provide_context(ReadThemeCookie(get_theme_cookie));
  provide_context(WriteThemeCookie(set_theme_cookie));
  (get_theme_cookie, set_theme_cookie)
}

pub fn provide_cookies() {
  provide_auth_cookie();
  provide_instance_cookie();
}

pub fn provide_auth_and_instance_cookies() {
  provide_cookies();
}
