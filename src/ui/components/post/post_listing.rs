use crate::{
  errors::{LemmyAppError, LemmyAppErrorType},
  lemmy_client::*,
  ui::components::common::icon::{Icon, IconType::*},
};
use ev::MouseEvent;
use lemmy_api_common::{lemmy_db_views::structs::*, person::*, post::*, site::GetSiteResponse};
use leptos::*;
use leptos_router::*;
use web_sys::SubmitEvent;

#[server(VotePostFn, "/serverfn")]
pub async fn vote_post_fn(post_id: i32, score: i16) -> Result<Option<PostResponse>, ServerFnError> {
  use lemmy_api_common::lemmy_db_schema::newtypes::PostId;
  let form = CreatePostLike {
    post_id: PostId(post_id),
    score,
  };
  let result = LemmyClient.like_post(form).await;
  use leptos_actix::redirect;
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
  use leptos_actix::redirect;
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
  use leptos_actix::redirect;
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
  use leptos_actix::redirect;
  match result {
    Ok(o) => Ok(Some(o)),
    Err(e) => {
      redirect(&format!("/?error={}", serde_json::to_string(&e)?)[..]);
      Ok(None)
    }
  }
}

#[component]
pub fn PostListing(
  post_view: MaybeSignal<PostView>,
  ssr_site: Resource<Option<bool>, Result<GetSiteResponse, LemmyAppError>>,
  post_number: usize,
  reply_show: RwSignal<bool>,
) -> impl IntoView {
  let error = expect_context::<RwSignal<Vec<Option<(LemmyAppError, Option<RwSignal<bool>>)>>>>();
  let logged_in = Signal::derive(move || {
    if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site.get() {
      Some(true)
    } else {
      Some(false)
    }
  });

  let post_view = RwSignal::new(post_view.get());
  let vote_action = create_server_action::<VotePostFn>();

  let on_vote_submit = move |ev: SubmitEvent, score: i16| {
    ev.prevent_default();
    create_local_resource(
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
          Err(e) => {
            error.update(|es| es.push(Some((e, None))));
          }
        }
      },
    );
  };

  let on_up_vote_submit = move |ev: SubmitEvent| {
    let score = if Some(1) == post_view.get().my_vote { 0 } else { 1 };
    on_vote_submit(ev, score);
  };

  let on_down_vote_submit = move |ev: SubmitEvent| {
    let score = if Some(-1) == post_view.get().my_vote { 0 } else { -1 };
    on_vote_submit(ev, score);
  };

  let save_post_action = create_server_action::<SavePostFn>();

  let on_save_submit = move |ev: SubmitEvent| {
    ev.prevent_default();
    create_local_resource(
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
          Err(e) => {
            error.update(|es| es.push(Some((e, None))));
          }
        }
      },
    );
  };

  let block_user_action = create_server_action::<BlockUserFn>();

  let on_block_submit = move |ev: SubmitEvent| {
    ev.prevent_default();
    create_local_resource(
      move || (),
      move |()| async move {
        let form = BlockPerson {
          person_id: post_view.get().creator.id,
          block: true,
        };
        let result = LemmyClient.block_user(form).await;
        match result {
          Ok(_o) => {}
          Err(e) => {
            error.update(|es| es.push(Some((e, None))));
          }
        }
      },
    );
  };

  let report_post_action = create_server_action::<ReportPostFn>();
  let report_validation = RwSignal::new(String::from(""));

  let query = use_query_map();
  let ssr_error = move || query.with(|params| params.get("error").cloned());

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
        logging::error!("error decoding error - log and ignore in UI?");
      }
    }
  }

  let reason = RwSignal::new(String::new());

  let on_report_submit = move |ev: SubmitEvent| {
    ev.prevent_default();
    create_local_resource(
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
            error.update(|es| es.push(Some((e.clone(), None))));

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
  let title_encoded = html_escape::encode_safe(&title).to_string();
  let community_title = post_view.get().community.title.clone();
  let community_title_encoded = html_escape::encode_safe(&community_title).to_string();
  let creator_name = post_view.get().creator.name.clone();
  let creator_name_encoded = html_escape::encode_safe(&creator_name).to_string();

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
  let abbr_duration = if let Some((index, _)) = duration_in_text.match_indices(' ').nth(1) {
    duration_in_text.split_at(index)
  } else {
    (&duration_in_text[..], "")
  }
  .0
  .to_string();

  view! {
    <div class="grid flex-row gap-y-3 gap-x-4 py-3 px-4 grid-cols-[6rem_1fr] grid-rows-[1fr_2rem] break-inside-avoid sm:grid-cols-[2rem_6rem_1fr] sm:grid-rows-[1fr_2rem]">
      <div class="hidden items-start pt-2 sm:flex sm:flex-row sm:col-span-1 sm:row-span-2">
        <div class="flex flex-col items-center w-8 text-center">
          <ActionForm action={vote_action} on:submit={on_up_vote_submit}>
            <input type="hidden" name="post_id" value={format!("{}", post_view.get().post.id)} />
            <input type="hidden" name="score" value={move || if Some(1) == post_view.get().my_vote { 0 } else { 1 }} />
            <button
              type="submit"
              class={move || {
                format!(
                  "align-bottom{}{}",
                  { if Some(true) != logged_in.get() { " text-base-content/50" } else { " hover:text-secondary/50" } },
                  { if Some(1) == post_view.get().my_vote { " text-secondary" } else { "" } },
                )
              }}
              disabled={move || Some(true) != logged_in.get()}
              title="Up vote"
            >
              <Icon icon={Upvote} />
            // <Icon icon=Upvote class="absolute animate-ping".into() />
            </button>
          </ActionForm>
          <span class="block text-sm">{move || post_view.get().counts.score}</span>
          <ActionForm action={vote_action} on:submit={on_down_vote_submit}>
            <input type="hidden" name="post_id" value={format!("{}", post_view.get().post.id)} />
            <input type="hidden" name="score" value={move || if Some(-1) == post_view.get().my_vote { 0 } else { -1 }} />
            <button
              type="submit"
              class={move || {
                format!(
                  "align-top{}{}",
                  { if Some(true) != logged_in.get() { " text-base-content/50" } else { " hover:text-primary/50" } },
                  { if Some(-1) == post_view.get().my_vote { " text-primary" } else { "" } },
                )
              }}
              disabled={move || Some(true) != logged_in.get()}
              title="Down vote"
            >
              <Icon icon={Downvote} />
            </button>
          </ActionForm>
        </div>
      </div>
      <div class={move || {
        format!(
          "row-span-1 col-span-1 sm:col-span-1 sm:row-span-2 flex items-start pt-2{}",
          if post_view.get().post.thumbnail_url.is_none() && post_view.get().post.url.is_none() { " hidden" } else { "" },
        )
      }}>
        <a
          class="flex flex-col h-full"
          href={move || { if let Some(d) = post_view.get().post.url { d.inner().to_string() } else { format!("/post/{}", post_view.get().post.id) } }}
        >
          {move || {
            if let Some(t) = post_view.get().post.thumbnail_url {
              let h = t.inner().to_string();
              view! {
                <div class="flex shrink grow basis-0 min-h-16">
                  <div class="shrink grow basis-0 truncate">
                    <img class="w-24" src={h} />
                  </div>
                </div>
              }
            } else {
              view! {
                <div class="block w-24 truncate">
                  <img class="w-24 h-16" src="/lemmy.svg" />
                </div>
              }
            }
          }}
        </a>
      </div>
      <div class={move || {
        format!(
          "row-span-1 col-span-1 sm:col-span-1 sm:row-span-1{}",
          if post_view.get().post.thumbnail_url.is_none() && post_view.get().post.url.is_none() { " col-span-2 sm:col-span-2" } else { "" },
        )
      }}>
        <A href={move || format!("/post/{}", post_view.get().post.id)} class="block hover:text-accent">
          <span class="text-lg break-words" inner_html={title_encoded} />
        </A>
        <span class="block mb-1">
          <span>{abbr_duration}</span>
          " ago by "
          <A href={move || format!("/u/{}", post_view.get().creator.name)} class="inline-block text-sm break-words hover:text-secondary">
            <span inner_html={creator_name_encoded} />
          </A>
          " in "
          <A
            class="inline-block text-sm break-words hover:text-secondary"
            href={if post_view.get().community.local {
              format!("/c/{}", post_view.get().community.name)
            } else {
              format!("/c/{}@{}", post_view.get().community.name, post_view.get().community.actor_id.inner().host().unwrap().to_string())
            }}
          >
            <span inner_html={community_title_encoded} />
          </A>
        </span>
      </div>
      <div class={move || {
        format!(
          "row-span-1 col-span-2 sm:col-span-1 sm:row-span-1 flex items-center gap-x-2{}",
          if post_view.get().post.thumbnail_url.is_none() && post_view.get().post.url.is_none() { " sm:col-span-2" } else { "" },
        )
      }}>
        <ActionForm action={vote_action} on:submit={on_up_vote_submit} class="flex items-center sm:hidden">
          <input type="hidden" name="post_id" value={format!("{}", post_view.get().post.id)} />
          <input type="hidden" name="score" value={move || if Some(1) == post_view.get().my_vote { 0 } else { 1 }} />
          <button
            type="submit"
            class={move || {
              format!(
                "{}{}",
                { if Some(1) == post_view.get().my_vote { "text-secondary" } else { "" } },
                { if Some(true) != logged_in.get() { " text-base-content/50" } else { " hover:text-secondary/50" } },
              )
            }}
            disabled={move || Some(true) != logged_in.get()}
            title="Up vote"
          >
            <Icon icon={Upvote} />
          </button>
        </ActionForm>
        <span class="block text-sm sm:hidden">{move || post_view.get().counts.score}</span>
        <ActionForm action={vote_action} on:submit={on_down_vote_submit} class="flex items-center sm:hidden">
          <input type="hidden" name="post_id" value={format!("{}", post_view.get().post.id)} />
          <input type="hidden" name="score" value={move || if Some(-1) == post_view.get().my_vote { 0 } else { -1 }} />
          <button
            type="submit"
            class={move || {
              format!(
                "{}{}",
                { if Some(-1) == post_view.get().my_vote { "text-primary" } else { "" } },
                { if Some(true) != logged_in.get() { " text-base-content/50" } else { " hover:text-primary/50" } },
              )
            }}
            disabled={move || Some(true) != logged_in.get()}
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
          <A href={move || { format!("/post/{}", post_view.get().post.id) }} class="text-sm whitespace-nowrap hover:text-accent">
            <Icon icon={Comments} class={"inline".into()} />
            " "
            {post_view.get().counts.comments}
            {if post_view.get().unread_comments != post_view.get().counts.comments && post_view.get().unread_comments > 0 {
              format!(" ({})", post_view.get().unread_comments)
            } else {
              "".to_string()
            }}
          </A>
        </span>
        <ActionForm action={save_post_action} on:submit={on_save_submit} class="flex items-center">
          <input type="hidden" name="post_id" value={format!("{}", post_view.get().post.id)} />
          <input type="hidden" name="save" value={move || format!("{}", !post_view.get().saved)} />
          <button
            type="submit"
            title="Save post"
            class={move || {
              format!(
                "{}{}",
                { if post_view.get().saved { "text-accent" } else { "" } },
                { if Some(true) != logged_in.get() { " text-base-content/50" } else { " hover:text-accent/50" } },
              )
            }}
            disabled={move || Some(true) != logged_in.get()}
          >
            <Icon icon={Save} />
          </button>
        </ActionForm>
        <Show when={move || { post_number == 0 }} fallback={|| {}}>
          <span class="cursor-pointer" on:click={move |_| reply_show.update(|b| *b = !*b)} title="Reply">
            <Icon icon={Reply} />
          </span>
        </Show>
        <span class={format!("text-base-content{}", if post_view.get().post.local { " hidden" } else { "" })} title="Original post">
          <A href={post_view.get().post.ap_id.inner().to_string()}>
            <Icon icon={External} />
          </A>
        </span>
        <Show when={move || { post_number == 0 }} fallback={|| {}}>
          <span
            class="text-base-content/50"
            title="Cross post"
            on:click={move |e: MouseEvent| {
              if e.ctrl_key() && e.shift_key() {
                let _ = window().location().set_href(&format!("//lemmy.world/post/{}", post_view.get().post.id));
              }
            }}
          >
            <Icon icon={Crosspost} />
          </span>
          <div class="dropdown max-sm:dropdown-end">
            <label tabindex="0">
              <Icon icon={VerticalDots} />
            </label>
            <ul tabindex="0" class="shadow menu dropdown-content z-[1] bg-base-100 rounded-box">
              <li>
                <ActionForm action={report_post_action} on:submit={on_report_submit} class="flex flex-col items-start">
                  <input type="hidden" name="post_id" value={format!("{}", post_view.get().post.id)} />
                  <input
                    class={move || format!("input input-bordered {}", report_validation.get())}
                    type="text"
                    on:input={move |e| update!(| reason | * reason = event_target_value(& e))}
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
                <ActionForm action={block_user_action} on:submit={on_block_submit}>
                  <input type="hidden" name="person_id" value={format!("{}", post_view.get().creator.id.0)} />
                  <input type="hidden" name="block" value="true" />
                  <button class="text-xs whitespace-nowrap" title="Block user" type="submit">
                    <Icon icon={Block} class={"inline-block".into()} />
                    "Block user"
                  </button>
                </ActionForm>
              </li>
            </ul>
          </div>
        </Show>
        <span class="text-right grow text-base-content/25">{if post_number != 0 { format!("{}", post_number) } else { "".into() }}</span>
      </div>
    </div>
  }
}
