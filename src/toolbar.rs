use crate::prelude::*;

#[server]
pub async fn vote_post_fn(post_id: i32, score: i16) -> Result<Option<PostResponse>, ServerFnError> {
  provide_cookies();
  let form = CreatePostLike { post_id: PostId(post_id), score };
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
  provide_cookies();
  let form = SavePost { post_id: PostId(post_id), save };
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
  provide_cookies();
  let form = BlockPerson { person_id: PersonId(person_id), block };
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
    Some(e) => Err(LemmyAppError { error_type: e.clone(), content: format!("{}", form.post_id.0) }),
  }
}

#[server]
pub async fn report_post_fn(post_id: i32, reason: String) -> Result<Option<PostReportResponse>, ServerFnError> {
  provide_cookies();

  let form = CreatePostReport { post_id: PostId(post_id), reason };
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
pub fn PostToolbar(
  post_view: PostView,
  reply_show: RwSignal<Option<bool>>,
  content: RwSignal<String>,
  post_id: Signal<Option<i32>>,
) -> impl IntoView {
  let i18n = use_i18n();
  let ssr_site = expect_context::<Resource<Result<GetSiteResponse, LemmyAppError>>>();
  let ReadInstanceCookie(get_instance_cookie) = expect_context::<ReadInstanceCookie>();
  let online = expect_context::<RwSignal<OnlineSetter>>();
  // let pv_clone = RwSignal::new(post_view.clone());
  let post_view = RwSignal::new(post_view);
  let vote_action = ServerAction::<VotePostFn>::new();
  let loading = RwSignal::new(false);
  let error = RwSignal::new(None::<LemmyAppError>);
  let description = RwSignal::new(String::new());

  let on_vote_submit = move |e: MouseEvent, score: i16| {
    e.prevent_default();
    spawn_local_scoped(async move {
      loading.set(true);
      error.set(None);
      description.set(t_string!(i18n, voting).to_owned());
      let form = CreatePostLike { post_id: post_view.get().post.id, score };
      let result = LemmyClient.like_post(form).await;
      match result {
        Ok(o) => {
          post_view.set(o.post_view);
          let post_response_cache = expect_context::<RwSignal<BTreeMap<GetCacheKey<GetPost>, (i64, LemmyAppResult<GetPostResponse>)>>>();
          post_response_cache.update(|prc| {
            prc.iter_mut().for_each(|(f, r)| {
              if let (_, Ok(w)) = r {
                let id = post_view.get().post.id.0;
                if w.post_view.post.id.0 == id {
                  w.post_view = post_view.get();
                }
              }
            });
          });
          let response_cache = expect_context::<RwSignal<BTreeMap<(usize, GetPostsCacheKey), (i64, LemmyAppResult<GetPostsResponse>)>>>();
          response_cache.update(|rc| {
            rc.iter_mut().for_each(|((_, cache_key), (_, cached_result))| {
              // let (_, form, _, _) = f;
              // let (_, cached_result) = r;
              if let Ok(w) = cached_result {
                let id = post_view.get().post.id.0;
                for p in w.posts.iter_mut() {
                  if p.post.id.0 == id {
                    *p = post_view.get();
                    let result_clone = cached_result.clone();
                    let cache_key = cache_key.clone();
                    spawn_local_scoped(async move {
                      if let Ok(d) = IndexedDb::new().await {
                        if let Ok(_c) = d.set::<GetPostsCacheKey, Result<GetPostsResponse, LemmyAppError>>(&cache_key, &result_clone).await {}
                      }
                    });
                    break;
                  }
                }
              }
            });
          });
        }
        Err(e) => {
          error.set(Some(e));
        }
      }
      loading.set(false);
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
    spawn_local_scoped(async move {
      loading.set(true);
      error.set(None);
      description.set(t_string!(i18n, save).to_owned());
      let form = SavePost { post_id: post_view.get().post.id, save: !post_view.get().saved };
      let result = LemmyClient.save_post(form).await;
      match result {
        Ok(o) => {
          post_view.set(o.post_view);
        }
        Err(e) => {
          error.set(Some(e));
        }
      }
      loading.set(false);
    });
  };

  let block_user_action = ServerAction::<BlockUserFn>::new();

  let on_block_submit = move |e: MouseEvent| {
    e.prevent_default();
    spawn_local_scoped(async move {
      loading.set(true);
      error.set(None);
      description.set(t_string!(i18n, block).to_owned());
      let form = BlockPerson { person_id: post_view.get().creator.id, block: true };
      let result = LemmyClient.block_user(form).await;
      match result {
        Ok(_o) => {}
        Err(e) => {
          error.set(Some(e));
        }
      }
      loading.set(false);
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
        LemmyAppError { error_type: LemmyAppErrorType::MissingReason, content: c } => {
          let id = format!("{}", post_view.get().post.id);
          if c.eq(&id) {
            report_validation.set("input-error".to_owned());
          }
        }
        _ => {
          report_validation.set("".to_owned());
        }
      },
      Err(_) => {}
    }
  }

  let reason = RwSignal::new(String::new());

  let on_report_submit = move |e: MouseEvent| {
    e.prevent_default();
    spawn_local_scoped(async move {
      loading.set(true);
      error.set(None);
      description.set(t_string!(i18n, report).to_owned());
      let form = CreatePostReport { post_id: post_view.get().post.id, reason: reason.get() };
      let result = try_report(form).await;
      match result {
        Ok(_o) => {}
        Err(e) => {
          let _id = format!("{}", post_view.get().post.id);
          match e {
            LemmyAppError { error_type: LemmyAppErrorType::MissingReason, content: _id } => {
              report_validation.set("input-error".to_owned());
            }
            _ => {
              report_validation.set("".to_owned());
            }
          }
        }
      }
      loading.set(false);
    });
  };

  let title = post_view.get().post.name.clone();
  let title_encoded = markdown_to_html(&title, false);

  let community_title = community_title_with_federation(
    &post_view.get().community.name,
    post_view.get().community.local,
    post_view.get().community.actor_id.inner().host().map(|host| host.to_string()),
  );
  let creator_name = &post_view.get().creator.actor_id.to_string()[8..];

  // let abbr_duration = relative_time(post_view.get().post.published.timestamp_millis());

  // #[cfg(not(feature = "ssr"))]
  // let _thumbnail_element = NodeRef::<Img>::new();
  // let _thumbnail = RwSignal::new(String::from(""));

  let enable_reply = RwSignal::new(false);

  view! {
    <Transition fallback={|| {}}>
      {move || {
        match ssr_site.get() {
          Some(Ok(s)) => {
            {
              let logged_in = Memo::new(move |_| { s.my_user.is_some() });
              // let post_view = Memo::new(move |_| pv_clone.get());

              #[cfg(not(feature = "ssr"))]
              let _ = set_timeout_with_handle(
                  move || {
                    if reply_show.get_untracked().is_some() {
                      enable_reply.set(true);
                    } else {
                      enable_reply.set(false);
                    }
                    // enable_reply.set(true);
                  },
                  std::time::Duration::new(0, 0),
                )
                .ok();

              view! {
                <div class="break-inside-avoid">
                  <div class="flex flex-wrap gap-x-2 items-center">
                    <ActionForm action={vote_action} attr:class="flex items-center">
                      <input type="hidden" name="post_id" value={format!("{}", post_view.get_untracked().post.id)} />
                      <input type="hidden" name="score" value={move || if Some(1) == post_view.get().my_vote { 0 } else { 1 }} />
                      <button
                        type="submit"
                        on:click={on_up_vote_submit}
                        class={move || {
                          format!(
                            "{}{}",
                            { if Some(1) == post_view.get().my_vote { "text-secondary" } else { "" } },
                            { if !logged_in.get() || !online.get().0 { " text-base-content/50" } else { " hover:text-secondary/50" } },
                          )
                        }}
                        disabled={move || !logged_in.get() || !online.get().0}
                        title={move || t_string!(i18n, up_vote)}
                      >
                        <Icon icon={Icon::Upvote} />
                      </button>
                    </ActionForm>
                    <span class="block text-sm">{move || post_view.get().counts.score}</span>
                    <ActionForm action={vote_action} attr:class="flex items-center">
                      <input type="hidden" name="post_id" value={format!("{}", post_view.get_untracked().post.id)} />
                      <input type="hidden" name="score" value={move || if Some(-1) == post_view.get().my_vote { 0 } else { -1 }} />
                      <button
                        type="submit"
                        on:click={on_down_vote_submit}
                        class={move || {
                          format!(
                            "{}{}",
                            { if Some(-1) == post_view.get().my_vote { "text-primary" } else { "" } },
                            { if !logged_in.get() || !online.get().0 { " text-base-content/50" } else { " hover:text-primary/50" } },
                          )
                        }}
                        disabled={move || !logged_in.get() || !online.get().0}
                        title={move || t_string!(i18n, down_vote)}
                      >
                        <Icon icon={Icon::Downvote} />
                      </button>
                    </ActionForm>
                    <span
                      class="flex items-center"
                      // title={
                      //   // let post = post_view.get_untracked();
                      //   // let count = t_string!(i18n, comments_count, count = post_view.get_untracked().counts.comments);
                      //   if post_view.get_untracked().unread_comments != post_view.get_untracked().counts.comments && post_view.get_untracked().unread_comments > 0 {
                      //     format!(
                      //       "{} ({})",
                      //       t_string!(i18n, comments_count, count = post_view.get_untracked().counts.comments),
                      //       t_string!(i18n, unread_comments_count, count = post_view.get_untracked().unread_comments),
                      //     )
                      //   } else {
                      //     t_string!(i18n, comments_count, count = post_view.get_untracked().counts.comments)
                      //   }
                      // }
                    >
                      <Icon icon={Icon::Comments} class="inline" />
                      {post_view.get().counts.comments}
                      {if post_view.get().unread_comments != post_view.get().counts.comments
                        && post_view.get().unread_comments > 0
                      {
                        format!(
                          " ({})",
                          post_view.get().unread_comments,
                        )
                      } else {
                        "".to_owned()
                      }}
                    </span>
                    <ActionForm action={save_post_action} attr:class="flex items-center">
                      <input type="hidden" name="post_id" value={format!("{}", post_view.get_untracked().post.id)} />
                      <input type="hidden" name="save" value={move || format!("{}", !post_view.get().saved)} />
                      <button
                        type="submit"
                        on:click={on_save_submit}
                        title={move || t_string!(i18n, save)}
                        class={move || {
                          format!(
                            "{}{}",
                            { if post_view.get().saved { "text-accent" } else { "" } },
                            { if !logged_in.get() || !online.get().0 { " text-base-content/50" } else { " hover:text-accent/50" } },
                          )
                        }}
                        disabled={move || !logged_in.get() || !online.get().0}
                      >
                        <Icon icon={Icon::Save} />
                      </button>
                    </ActionForm>
                    <button
                      class={move || {
                        format!(
                          "cursor-pointer{}",
                          {
                            if !enable_reply.get() || !logged_in.get() || !online.get().0 { " text-base-content/50" } else { " hover:text-accent/50" }
                          },
                        )
                      }}
                      on:click={move |_| {
                        if let Some(id) = post_id.get_untracked() {
                          #[cfg(not(feature = "ssr"))]
                          spawn_local_scoped(async move {
                            if let Ok(Some(c)) = IndexedDb::get_comment_draft(id, Draft::Post).await {
                              content.set(c);
                            }
                          });
                        }
                        reply_show.update(|b| { *b = b.map(|b| !b); });
                      }}
                      title={move || t_string!(i18n, reply)}
                      disabled={move || !enable_reply.get() || !logged_in.get() || !online.get().0}
                    >
                      <Icon icon={Icon::Reply} />
                    </button>
                    <span class={format!("text-base-content{}", if post_view.get_untracked().post.local { " hidden" } else { "" })} title={move || t_string!(i18n, original)}>
                      <A href={post_view.get_untracked().post.ap_id.inner().to_string()}>
                        <Icon icon={Icon::External} />
                      </A>
                    </span>
                    <span
                      class={format!(
                        "text-base-content{}",
                        {
                          if let Some(d) = post_view.get_untracked().post.url {
                            if let Some(f) = d.inner().host_str() {
                              if f.to_string().ne(&get_instance_cookie.get_untracked().unwrap_or("".into())) { "" } else { " hidden" }
                            } else {
                              " hidden"
                            }
                          } else {
                            " hidden"
                          }
                        },
                      )}
                      title={move || t_string!(i18n, archive)}
                    >
                      <a
                        target="_blank"
                        href={format!(
                          "https://archive.ph/submit/?url={}",
                          { if let Some(d) = post_view.get_untracked().post.url { d.inner().to_string() } else { "".to_owned() } },
                        )}
                      >
                        <Icon icon={Icon::History} />
                      </a>
                    </span>
                    <span class="flex ml-auto item-center">
                      <div class="dropdown max-sm:dropdown-end">
                        <label tabindex="0">
                          <Icon icon={Icon::VerticalDots} />
                        </label>
                        <ul tabindex="0" class="shadow menu dropdown-content z-[1] bg-base-100 rounded-box">
                          <li>
                            <ActionForm action={report_post_action} attr:class="flex flex-col items-start">
                              <input type="hidden" name="post_id" value={format!("{}", post_view.get_untracked().post.id)} />
                              <input
                                class={move || format!("input input-bordered {}", report_validation.get_untracked())}
                                type="text"
                                on:click={on_report_submit}
                                on:input={move |e| reason.update(|r| *r = event_target_value(&e))}
                                name="reason"
                                placeholder={move || t_string!(i18n, report_reason)}
                              />
                              <button class="text-xs whitespace-nowrap" title={move || t_string!(i18n, report)} type="submit">
                                <Icon icon={Icon::Report} class="inline-block" />
                                {t!(i18n, report_post)}
                              </button>
                            </ActionForm>
                          </li>
                          <li>
                            <ActionForm action={block_user_action}>
                              <input type="hidden" name="person_id" value={format!("{}", post_view.get_untracked().creator.id.0)} />
                              <input type="hidden" name="block" value="true" />
                              <button on:click={on_block_submit} class="text-xs whitespace-nowrap" title={move || t_string!(i18n, block)} type="submit">
                                <Icon icon={Icon::Block} class="inline-block" />
                                {t!(i18n, block_user)}
                              </button>
                            </ActionForm>
                          </li>
                        </ul>
                      </div>
                    </span>
                  </div>
                </div>
              }
            }
              .into_any()
          }
          _ => view! {}.into_any(),
        }
      }}
    </Transition>
    <Show when={move || loading.get()} fallback={|| {}}>
      {move || {
        view! { <Info loading={loading.get()} /> }
      }}
    </Show>
    <Show when={move || error.get().is_some()} fallback={|| {}}>
      {move || {
        view! {
          <Error
            description={description.get()}
            error={error
              .get()
              .unwrap_or_else(|| LemmyAppError {
                error_type: LemmyAppErrorType::Unknown,
                content: "Unknown".to_owned(),
              })}
          />
        }
      }}
    </Show>
  }
}
