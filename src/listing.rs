use crate::{
  client::*,
  errors::{LemmyAppError, LemmyAppErrorType},
  icon::{IconType::*, *},
  OnlineSetter, ReadInstanceCookie,
};
use lemmy_api_common::{lemmy_db_views::structs::*, person::*, post::*, site::GetSiteResponse};
use leptos::{html::Img, logging::*, prelude::*};
use leptos_router::{components::*, hooks::*};
use web_sys::MouseEvent;

#[server]
pub async fn vote_post_fn(post_id: i32, score: i16) -> Result<Option<PostResponse>, ServerFnError> {
  use lemmy_api_common::lemmy_db_schema::newtypes::PostId;
  let form = CreatePostLike {
    post_id: PostId(post_id),
    score,
  };
  let result = LemmyClient.like_post(form).await;
  use leptos_axum::redirect;
  match result {
    Ok(o) => Ok(Some(o)),
    Err(e) => {
      redirect(&format!("/?error={}", serde_json::to_string(&e)?)[..]);
      Ok(None)
    }
  }
}

#[server]
pub async fn save_post_fn(post_id: i32, save: bool) -> Result<Option<PostResponse>, ServerFnError> {
  use lemmy_api_common::lemmy_db_schema::newtypes::PostId;
  let form = SavePost {
    post_id: PostId(post_id),
    save,
  };
  let result = LemmyClient.save_post(form).await;
  use leptos_axum::redirect;
  match result {
    Ok(o) => Ok(Some(o)),
    Err(e) => {
      redirect(&format!("/?error={}", serde_json::to_string(&e)?)[..]);
      Ok(None)
    }
  }
}

#[server]
pub async fn block_user_fn(person_id: i32, block: bool) -> Result<Option<BlockPersonResponse>, ServerFnError> {
  use lemmy_api_common::lemmy_db_schema::newtypes::PersonId;
  let form = BlockPerson {
    person_id: PersonId(person_id),
    block,
  };
  let result = LemmyClient.block_user(form).await;
  use leptos_axum::redirect;
  match result {
    Ok(o) => Ok(Some(o)),
    Err(e) => {
      redirect(&format!("/?error={}", serde_json::to_string(&e)?)[..]);
      Ok(None)
    }
  }
}

fn validate_report(form: &CreatePostReport) -> Option<LemmyAppErrorType> {
  if form.reason.is_empty() {
    return Some(LemmyAppErrorType::MissingReason);
  }
  None
}

async fn try_report(form: CreatePostReport) -> Result<PostReportResponse, LemmyAppError> {
  let val = validate_report(&form);
  match val {
    None => {
      let result = LemmyClient.report_post(form).await;
      match result {
        Ok(o) => Ok(o),
        Err(e) => Err(e),
      }
    }
    Some(e) => Err(LemmyAppError {
      error_type: e.clone(),
      content: format!("{}", form.post_id.0),
    }),
  }
}

#[server]
pub async fn report_post_fn(post_id: i32, reason: String) -> Result<Option<PostReportResponse>, ServerFnError> {
  use lemmy_api_common::lemmy_db_schema::newtypes::PostId;

  let form = CreatePostReport {
    post_id: PostId(post_id),
    reason,
  };
  let result = try_report(form).await;
  use leptos_axum::redirect;
  match result {
    Ok(o) => Ok(Some(o)),
    Err(e) => {
      redirect(&format!("/?error={}", serde_json::to_string(&e)?)[..]);
      Ok(None)
    }
  }
}

#[component]
pub fn Listing(post_view: PostView, post_number: usize, reply_show: RwSignal<bool>) -> impl IntoView {
  let ssr_site_signal = expect_context::<RwSignal<Option<Result<GetSiteResponse, LemmyAppError>>>>();
  let logged_in = Signal::derive(move || {
    if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site_signal.get() {
      Some(true)
    } else {
      Some(false)
    }
  });
  let online = expect_context::<RwSignal<OnlineSetter>>();
  let ReadInstanceCookie(get_instance_cookie) = expect_context::<ReadInstanceCookie>();
  let post_view = RwSignal::new(post_view);
  let vote_action = ServerAction::<VotePostFn>::new();

  let on_vote_submit = move |e: MouseEvent, score: i16| {
    e.prevent_default();
    Resource::new(
      move || (),
      move |()| async move {
        let form = CreatePostLike {
          post_id: post_view.get().post.id,
          score,
        };
        let result = LemmyClient.like_post(form).await;
        match result {
          Ok(o) => {
            post_view.set(o.post_view);
          }
          Err(_e) => {}
        }
      },
    );
  };

  let on_up_vote_submit = move |e: MouseEvent| {
    let score = if Some(1) == post_view.get().my_vote { 0 } else { 1 };
    on_vote_submit(e, score);
  };

  let _on_down_vote_submit = move |e: MouseEvent| {
    let score = if Some(-1) == post_view.get().my_vote { 0 } else { -1 };
    on_vote_submit(e, score);
  };

  let save_post_action = ServerAction::<SavePostFn>::new();

  let on_save_submit = move |e: MouseEvent| {
    e.prevent_default();
    Resource::new(
      move || (),
      move |()| async move {
        let form = SavePost {
          post_id: post_view.get().post.id,
          save: !post_view.get().saved,
        };
        let result = LemmyClient.save_post(form).await;
        match result {
          Ok(o) => {
            post_view.set(o.post_view);
          }
          Err(_e) => {}
        }
      },
    );
  };

  let _block_user_action = ServerAction::<BlockUserFn>::new();

  let _on_block_submit = move |e: MouseEvent| {
    e.prevent_default();
    Resource::new(
      move || (),
      move |()| async move {
        let form = BlockPerson {
          person_id: post_view.get().creator.id,
          block: true,
        };
        let result = LemmyClient.block_user(form).await;
        match result {
          Ok(_o) => {}
          Err(_e) => {}
        }
      },
    );
  };

  let _report_post_action = ServerAction::<ReportPostFn>::new();
  let report_validation = RwSignal::new(String::from(""));

  let query = use_query_map();
  let ssr_error = move || query.with(|params| params.get("error"));

  if let Some(e) = ssr_error() {
    let le = serde_json::from_str::<LemmyAppError>(&e[..]);
    match le {
      Ok(e) => match e {
        LemmyAppError {
          error_type: LemmyAppErrorType::MissingReason,
          content: c,
        } => {
          let id = format!("{}", post_view.get().post.id);
          if c.eq(&id) {
            report_validation.set("input-error".to_string());
          }
        }
        _ => {
          report_validation.set("".to_string());
        }
      },
      Err(_) => {
        error!("error decoding error - log and ignore in UI?");
      }
    }
  }

  let reason = RwSignal::new(String::new());

  let _on_report_submit = move |e: MouseEvent| {
    e.prevent_default();
    Resource::new(
      move || (),
      move |()| async move {
        let form = CreatePostReport {
          post_id: post_view.get().post.id,
          reason: reason.get(),
        };
        let result = try_report(form).await;
        match result {
          Ok(_o) => {}
          Err(e) => {
            let _id = format!("{}", post_view.get().post.id);
            match e {
              LemmyAppError {
                error_type: LemmyAppErrorType::MissingReason,
                content: _id,
              } => {
                report_validation.set("input-error".to_string());
              }
              _ => {
                report_validation.set("".to_string());
              }
            }
          }
        }
      },
    );
  };

  let title = post_view.get().post.name.clone();
  let mut options = pulldown_cmark::Options::empty();
  options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
  options.insert(pulldown_cmark::Options::ENABLE_TABLES);
  options.insert(pulldown_cmark::Options::ENABLE_SUPERSCRIPT);
  options.insert(pulldown_cmark::Options::ENABLE_SUBSCRIPT);
  options.insert(pulldown_cmark::Options::ENABLE_CONTAINER_EXTENSIONS);
  let parser = pulldown_cmark::Parser::new_ext(&title, options);

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
  let mut title_encoded = String::new();
  pulldown_cmark::html::push_html(&mut title_encoded, custom);

  let community_title = if post_view.get().community.local {
    format!("{}", post_view.get().community.name)
  } else {
    format!(
      "{}@{}",
      post_view.get().community.name,
      if let Some(h) = post_view.get().community.actor_id.inner().host() {
        h.to_string()
      } else {
        "".to_string()
      }
    )
  };
  let community_title_encoded = html_escape::encode_safe(&community_title).to_string();
  let creator_name = &post_view.get().creator.actor_id.to_string()[8..];
  let creator_name_encoded = html_escape::encode_safe(creator_name).to_string();

  let now_in_millis = {
    #[cfg(not(feature = "ssr"))]
    {
      chrono::offset::Utc::now().timestamp_millis() as u64
    }
    #[cfg(feature = "ssr")]
    {
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::new(1000, 0))
        .as_millis() as u64
    }
  };
  let duration_in_text = pretty_duration::pretty_duration(
    &std::time::Duration::from_millis(now_in_millis - post_view.get().post.published.timestamp_millis() as u64),
    Some(pretty_duration::PrettyDurationOptions {
      output_format: Some(pretty_duration::PrettyDurationOutputFormat::Compact),
      singular_labels: None,
      plural_labels: None,
    }),
  );
  let abbr_duration = if let Some((index, _)) = duration_in_text.match_indices(' ').nth(1) {
    duration_in_text.split_at(index)
  } else {
    (&duration_in_text[..], "")
  }
  .0
  .to_string();

  let thumbnail_element = NodeRef::<Img>::new();
  let thumbnail = RwSignal::new(String::from(""));

  view! {
    <div class="grid gap-x-4 px-4 pb-6 grid-cols-[6rem_1fr] grid-rows-[1fr_2rem] break-inside-avoid sm:grid-rows-[1fr_2rem]">
      <div class={move || {
        format!(
          "col-span-1 row-span-2 flex items-start pt-2{}",
          if post_view.get().post.thumbnail_url.is_none() && post_view.get().post.url.is_none() { " hidden" } else { "" },
        )
      }}>
        <a
          class="flex flex-col h-full"
          target="_blank"
          href={move || {
            if let Some(d) = post_view.get().post.url { d.inner().to_string() } else { format!("/post/{}", post_view.get().post.id) }
          }}
        >
          {move || {
            if let Some(t) = post_view.get().post.thumbnail_url {
              let h = t.inner().to_string();
              thumbnail.set(h);
              view! {
                <div class="flex shrink grow basis-0 min-h-16">
                  <div class="shrink grow basis-0 truncate">
                    <img
                      loading="lazy"
                      class={move || format!("w-24{}", if thumbnail.get().eq(&"/lemmy.svg".to_string()) { " h-16" } else { "" })}
                      src={move || thumbnail.get()}
                      node_ref={thumbnail_element}
                      on:error={move |_e| {
                        thumbnail.set("/lemmy.svg".into());
                      }}
                    />
                  </div>
                </div>
              }
                .into_any()
            } else {
              view! {
                <div class="block w-24 truncate">
                  <img class="w-24 h-16" src="/lemmy.svg" />
                </div>
              }
                .into_any()
            }
          }}
        </a>
      </div>
      <div class={move || {
        format!(
          "col-span-1 row-span-1{}",
          if post_view.get().post.thumbnail_url.is_none() && post_view.get().post.url.is_none() { " col-span-2 sm:col-span-2" } else { "" },
        )
      }}>
        <A href={move || format!("/p/{}", post_view.get().post.id)} attr:class="block hover:text-accent">
          <span class="overflow-y-auto text-lg wrap-anywhere" inner_html={title_encoded} />
        </A>
        <span class="block mt-1 mb-1 text-sm wrap-anywhere">
          <span>{abbr_duration}</span>
          " ago by "
          <span class="overflow-y-auto" inner_html={creator_name_encoded} />
          " in "
          <span class="overflow-y-auto" inner_html={community_title_encoded} />
          <span
            class="overflow-y-auto"
            inner_html={if let Some(d) = post_view.get().post.url {
              if let Some(f) = d.inner().host_str() {
                if f.to_string().ne(&get_instance_cookie.get().unwrap_or("".into())) { format!(" from {}", f) } else { "".into() }
              } else {
                "".into()
              }
            } else {
              "".into()
            }}
          />
        </span>
      </div>
      <div class={move || {
        format!(
          "row-span-1 flex items-center gap-x-1{}",
          if post_view.get().post.thumbnail_url.is_none() && post_view.get().post.url.is_none() { " col-span-2" } else { " col-span-1" },
        )
      }}>
        <ActionForm action={vote_action} attr:class="flex items-center">
          <input type="hidden" name="post_id" value={format!("{}", post_view.get().post.id)} />
          <input type="hidden" name="score" value={move || if Some(1) == post_view.get().my_vote { 0 } else { 1 }} />
          <button
            type="submit"
            on:click={on_up_vote_submit}
            class={move || {
              format!(
                "{}{}",
                { if Some(1) == post_view.get().my_vote { "text-secondary" } else { "" } },
                { if Some(true) != logged_in.get() || !online.get().0 { " text-base-content/50" } else { " hover:text-secondary/50" } },
              )
            }}
            disabled={move || Some(true) != logged_in.get() || !online.get().0}
            title="Up vote"
          >
            <Icon icon={Upvote} />
          </button>
        </ActionForm>
        <span class="block text-sm">{move || post_view.get().counts.score}</span>
        <span
          class="flex items-center pl-1"
          title={move || {
            format!(
              "{} comments{}",
              post_view.get().counts.comments,
              if post_view.get().unread_comments != post_view.get().counts.comments && post_view.get().unread_comments > 0 {
                format!(" ({} unread)", post_view.get().unread_comments)
              } else {
                "".to_string()
              },
            )
          }}
        >
          <Icon icon={Comments} class={"inline".into()} />
          {post_view.get().counts.comments}
        </span>
        <Show when={move || { post_number == 0 }} fallback={|| {}}>
          <ActionForm action={save_post_action} attr:class="flex items-center">
            <input type="hidden" name="post_id" value={format!("{}", post_view.get().post.id)} />
            <input type="hidden" name="save" value={move || format!("{}", !post_view.get().saved)} />
            <button
              type="submit"
              on:click={on_save_submit}
              title="Save post"
              class={move || {
                format!(
                  "{}{}",
                  { if post_view.get().saved { "text-accent" } else { "" } },
                  { if Some(true) != logged_in.get() || !online.get().0 { " text-base-content/50" } else { " hover:text-accent/50" } },
                )
              }}
              disabled={move || Some(true) != logged_in.get() || !online.get().0}
            >
              <Icon icon={Save} />
            </button>
          </ActionForm>
          <span
            class="cursor-pointer"
            on:click={move |_| {
              reply_show.update(|b| *b = !*b);
            }}
            title="Reply"
          >
            <Icon icon={Reply} />
          </span>
          <span class={format!("text-base-content{}", if post_view.get().post.local { " hidden" } else { "" })} title="Original">
            <A href={post_view.get().post.ap_id.inner().to_string()}>
              <Icon icon={External} />
            </A>
          </span>

        </Show>
        <span class="flex items-center ml-auto text-base-content/25">
          <a
            class={format!(
              "{}",
              {
                if let Some(d) = post_view.get().post.url {
                  if let Some(f) = d.inner().host_str() {
                    if f.to_string().ne(&get_instance_cookie.get().unwrap_or("".into())) { "" } else { " hidden" }
                  } else {
                    " hidden"
                  }
                } else {
                  " hidden"
                }
              },
            )}
            title="Archive"
            target="_blank"
            href={format!(
              "https://archive.ph/submit/?url={}",
              { if let Some(d) = post_view.get().post.url { d.inner().to_string() } else { "".to_string() } },
            )}
          >
            <Icon icon={History} />
          </a>
        </span>
        <span class="flex items-center text-base-content/25">{if post_number != 0 { format!("{}", post_number) } else { "".into() }}</span>
      </div>
    </div>
  }
}
