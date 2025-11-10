use crate::{
  client::*,
  db::csr_indexed_db::*,
  errors::{LemmyAppError, LemmyAppErrorType},
  icon::{IconType::*, *},
  OnlineSetter, ReadInstanceCookie,
};
use lemmy_api_common::{lemmy_db_views::structs::*, person::*, post::*, site::GetSiteResponse};
use leptos::{html::Img, prelude::*, task::spawn_local_scoped_with_cancellation};
use leptos_router::{components::A, hooks::*};
use web_sys::MouseEvent;

#[server(VotePostFn, "/serverfn")]
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

#[server(SavePostFn, "/serverfn")]
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

#[server(BlockUserFn, "/serverfn")]
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

#[server(ReportPostFn, "/serverfn")]
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
pub fn ResponsivePostToolbar(
  post_view: Signal<PostView>,
  post_number: usize,
  reply_show: RwSignal<bool>,
  content: RwSignal<String>,
  post_id: Signal<Option<i32>>,
) -> impl IntoView {
  let ssr_site_signal = expect_context::<RwSignal<Option<Result<GetSiteResponse, LemmyAppError>>>>();
  let logged_in = Signal::derive(move || {
    if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site_signal.get() {
      Some(true)
    } else {
      Some(false)
    }
  });
  let ReadInstanceCookie(get_instance_cookie) = expect_context::<ReadInstanceCookie>();
  let online = expect_context::<RwSignal<OnlineSetter>>();
  let post_view = RwSignal::new(post_view.get());
  let vote_action = ServerAction::<VotePostFn>::new();

  let on_vote_submit = move |e: MouseEvent, score: i16| {
    e.prevent_default();
    spawn_local_scoped_with_cancellation(async move {
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
    });
  };

  let on_up_vote_submit = move |e: MouseEvent| {
    let score = if Some(1) == post_view.get().my_vote { 0 } else { 1 };
    on_vote_submit(e, score);
  };

  let on_down_vote_submit = move |e: MouseEvent| {
    let score = if Some(-1) == post_view.get().my_vote { 0 } else { -1 };
    on_vote_submit(e, score);
  };

  let save_post_action = ServerAction::<SavePostFn>::new();

  let on_save_submit = move |e: MouseEvent| {
    e.prevent_default();
    spawn_local_scoped_with_cancellation(async move {
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
    });
  };

  let block_user_action = ServerAction::<BlockUserFn>::new();

  let on_block_submit = move |e: MouseEvent| {
    e.prevent_default();
    spawn_local_scoped_with_cancellation(async move {
      let form = BlockPerson {
        person_id: post_view.get().creator.id,
        block: true,
      };
      let result = LemmyClient.block_user(form).await;
      match result {
        Ok(_o) => {}
        Err(_e) => {}
      }
    });
  };

  let report_post_action = ServerAction::<ReportPostFn>::new();
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
      Err(_) => {}
    }
  }

  let reason = RwSignal::new(String::new());

  let on_report_submit = move |e: MouseEvent| {
    e.prevent_default();
    spawn_local_scoped_with_cancellation(async move {
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
    });
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
      post_view.get().community.actor_id.inner().host().unwrap().to_string()
    )
  };
  let _community_title_encoded = html_escape::encode_safe(&community_title).to_string();
  let creator_name = &post_view.get().creator.actor_id.to_string()[8..];
  let _creator_name_encoded = html_escape::encode_safe(creator_name).to_string();

  let now_in_millis = {
    #[cfg(not(feature = "ssr"))]
    {
      chrono::offset::Utc::now().timestamp_millis() as u64
    }
    #[cfg(feature = "ssr")]
    {
      std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64
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
  let _abbr_duration = if let Some((index, _)) = duration_in_text.match_indices(' ').nth(1) {
    duration_in_text.split_at(index)
  } else {
    (&duration_in_text[..], "")
  }
  .0
  .to_string();

  #[cfg(not(feature = "ssr"))]
  let _thumbnail_element = NodeRef::<Img>::new();
  let _thumbnail = RwSignal::new(String::from(""));

  view! {
    <div class="px-4 break-inside-avoid">
      <div class="flex flex-wrap gap-x-2 items-center pb-2">
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
        <ActionForm action={vote_action} attr:class="flex items-center">
          <input type="hidden" name="post_id" value={format!("{}", post_view.get().post.id)} />
          <input type="hidden" name="score" value={move || if Some(-1) == post_view.get().my_vote { 0 } else { -1 }} />
          <button
            type="submit"
            on:click={on_down_vote_submit}
            class={move || {
              format!(
                "{}{}",
                { if Some(-1) == post_view.get().my_vote { "text-primary" } else { "" } },
                { if Some(true) != logged_in.get() || !online.get().0 { " text-base-content/50" } else { " hover:text-primary/50" } },
              )
            }}
            disabled={move || Some(true) != logged_in.get() || !online.get().0}
            title="Down vote"
          >
            <Icon icon={Downvote} />
          </button>
        </ActionForm>
        <span
          class="flex items-center"
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
          {if post_view.get().unread_comments != post_view.get().counts.comments && post_view.get().unread_comments > 0 {
            format!(" ({})", post_view.get().unread_comments)
          } else {
            "".to_string()
          }}
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
          <button
            class={move || {
              format!(
                "cursor-pointer{}",
                { if Some(true) != logged_in.get() || !online.get().0 { " text-base-content/50" } else { " hover:text-accent/50" } },
              )
            }}
            on:click={move |_| {
              if let Some(id) = post_id.get() {
                #[cfg(not(feature = "ssr"))]
                spawn_local_scoped_with_cancellation(async move {
                  if let Ok(d) = IndexedDb::new().await {
                    if let Ok(Some(c)) = d
                      .get(
                        &CommentDraftKey {
                          comment_id: id,
                          draft: Draft::Post,
                        },
                      )
                      .await
                    {
                      content.set(c);
                    }
                  }
                });
              }
              reply_show.update(|b| *b = !*b);
            }}
            title="Reply"
            disabled={move || Some(true) != logged_in.get() || !online.get().0}
          >
            <Icon icon={Reply} />
          </button>
          <span class={format!("text-base-content{}", if post_view.get().post.local { " hidden" } else { "" })} title="Original">
            <A href={post_view.get().post.ap_id.inner().to_string()}>
              <Icon icon={External} />
            </A>
          </span>
          <span
            class={format!(
              "text-base-content{}",
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
          >
            <a
              target="_blank"
              href={format!(
                "https://archive.ph/submit/?url={}",
                { if let Some(d) = post_view.get().post.url { d.inner().to_string() } else { "".to_string() } },
              )}
            >
              <Icon icon={History} />
            </a>
          </span>
          <span class="flex ml-auto item-center">
            <div class="dropdown max-sm:dropdown-end">
              <label tabindex="0">
                <Icon icon={VerticalDots} />
              </label>
              <ul tabindex="0" class="shadow menu dropdown-content z-[1] bg-base-100 rounded-box">
                <li>
                  <ActionForm action={report_post_action} attr:class="flex flex-col items-start">
                    <input type="hidden" name="post_id" value={format!("{}", post_view.get().post.id)} />
                    <input
                      class={move || format!("input input-bordered {}", report_validation.get())}
                      type="text"
                      on:click={on_report_submit}
                      on:input={move |e| reason.update(|r| *r = event_target_value(&e))}
                      name="reason"
                      placeholder="Reason for reporting post"
                    />
                    <button class="text-xs whitespace-nowrap" title="Report post" type="submit">
                      <Icon icon={Report} class={"inline-block".into()} />
                      "Report post"
                    </button>
                  </ActionForm>
                </li>
                <li>
                  <ActionForm action={block_user_action}>
                    <input type="hidden" name="person_id" value={format!("{}", post_view.get().creator.id.0)} />
                    <input type="hidden" name="block" value="true" />
                    <button on:click={on_block_submit} class="text-xs whitespace-nowrap" title="Block user" type="submit">
                      <Icon icon={Block} class={"inline-block".into()} />
                      "Block user"
                    </button>
                  </ActionForm>
                </li>
              </ul>
            </div>
          </span>
        </Show>
      </div>
    </div>
  }
}
