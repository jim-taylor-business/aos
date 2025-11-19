use crate::{
  OnlineSetter,
  client::*,
  db::csr_indexed_db::*,
  errors::LemmyAppError,
  icon::{Icon, IconType::*},
};
use lemmy_api_common::{
  comment::{CreateComment, CreateCommentLike, EditComment, GetComment, SaveComment},
  lemmy_db_schema::newtypes::PersonId,
  lemmy_db_views::structs::{CommentView, LocalUserView},
  site::{GetModlog, GetSiteResponse, MyUserInfo},
};
use leptos::{html::Textarea, logging::log, prelude::*, task::*, *};
use leptos_dom::helpers::TimeoutHandle;
use leptos_router::components::Form;
use leptos_use::{UseIntersectionObserverOptions, use_intersection_observer_with_options};
use web_sys::{HtmlAnchorElement, HtmlImageElement, MouseEvent, PointerEvent, WheelEvent, wasm_bindgen::JsCast};

#[component]
pub fn Comment(
  comment: Signal<CommentView>,
  comments: Signal<Vec<CommentView>>,
  level: usize,
  parent_comment_id: i32,
  now_in_millis: RwSignal<u64>,
  hidden_comments: RwSignal<Vec<i32>>,
  highlight_user_id: RwSignal<Option<PersonId>>,
  post_id: Signal<Option<i32>>,
) -> impl IntoView {
  let ssr_site_signal = expect_context::<RwSignal<Option<Result<GetSiteResponse, LemmyAppError>>>>();

  let logged_in =
    Signal::derive(move || if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site_signal.get() { Some(true) } else { Some(false) });
  let online = expect_context::<RwSignal<OnlineSetter>>();

  let on_toggle = move |i: i32| {
    if hidden_comments.get().contains(&i) {
      hidden_comments.update(|hc| hc.retain(|c| i != *c));
    } else {
      hidden_comments.update(|hc| hc.push(i));
    }
    #[cfg(not(feature = "ssr"))]
    spawn_local_scoped_with_cancellation(async move {
      if let Some(p) = post_id.get() {
        if let Ok(d) = IndexedDb::new().await {
          if let Ok(_) = d.set(&p, &hidden_comments.get()).await {}
        }
      }
    });
  };

  let current_person = Signal::derive(move || {
    if let Some(Ok(GetSiteResponse { my_user: Some(MyUserInfo { local_user_view: LocalUserView { person, .. }, .. }), .. })) = ssr_site_signal.get() {
      Some(person)
    } else {
      None
    }
  });

  let mut comments_descendants = comments.get().clone();
  let id = comment.get().comment.id.to_string();

  let mut comments_children: Vec<CommentView> = vec![];

  comments_descendants.retain(|ct| {
    let tree = ct.comment.path.split('.').collect::<Vec<_>>();
    if tree.len() == level + 2 {
      if tree.get(level).unwrap_or(&"").eq(&id) {
        comments_children.push(ct.clone());
      }
      false
    } else if tree.len() > level + 2 {
      tree.get(level).unwrap_or(&"").eq(&id)
    } else {
      false
    }
  });

  let children = RwSignal::new(comments_children);
  let descendants = RwSignal::new(comments_descendants);

  let comment_view = RwSignal::new(comment.get());
  let comment_copy = RwSignal::new(comment.get());

  let safe_html = Signal::derive(move || {
    let content = comment_view.get().comment.content;

    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    options.insert(pulldown_cmark::Options::ENABLE_SUPERSCRIPT);
    options.insert(pulldown_cmark::Options::ENABLE_SUBSCRIPT);
    options.insert(pulldown_cmark::Options::ENABLE_CONTAINER_EXTENSIONS);
    let parser = pulldown_cmark::Parser::new_ext(&content, options);

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
  });

  let highlight_show = RwSignal::new(false);
  let still_down = RwSignal::new(false);
  let vote_show = RwSignal::new(false);
  let reply_show = RwSignal::new(false);
  let edit_show = RwSignal::new(false);
  let still_handle: RwSignal<Option<TimeoutHandle>> = RwSignal::new(None);
  let loading = RwSignal::new(false);

  let reply_content = RwSignal::new(String::default());
  let edit_content = RwSignal::new(String::default());

  let duration_in_text = pretty_duration::pretty_duration(
    &std::time::Duration::from_millis(now_in_millis.get() - comment_view.get().comment.published.timestamp_millis() as u64),
    Some(pretty_duration::PrettyDurationOptions {
      output_format: Some(pretty_duration::PrettyDurationOutputFormat::Compact),
      singular_labels: None,
      plural_labels: None,
    }),
  );
  let abbr_duration =
    if let Some((index, _)) = duration_in_text.match_indices(' ').nth(1) { duration_in_text.split_at(index) } else { (&duration_in_text[..], "") }
      .0
      .to_string();

  let cancel = move |e: MouseEvent| {
    e.stop_propagation();
  };

  let on_vote_submit = move |e: MouseEvent, score: i16| {
    e.prevent_default();
    spawn_local_scoped_with_cancellation(async move {
      let form = CreateCommentLike { comment_id: comment_view.get().comment.id, score };
      let result = LemmyClient.like_comment(form).await;
      match result {
        Ok(o) => {
          comment_view.set(o.comment_view);
        }
        Err(_e) => {}
      }
    });
  };

  let on_up_vote_submit = move |e: MouseEvent| {
    let score = if Some(1) == comment_view.get().my_vote { 0 } else { 1 };
    on_vote_submit(e, score);
  };

  let on_down_vote_submit = move |e: MouseEvent| {
    let score = if Some(-1) == comment_view.get().my_vote { 0 } else { -1 };
    on_vote_submit(e, score);
  };

  let on_save_submit = move |e: MouseEvent| {
    e.prevent_default();
    spawn_local_scoped_with_cancellation(async move {
      let form = SaveComment { comment_id: comment_view.get().comment.id, save: !comment_view.get().saved };
      let result = LemmyClient.save_comment(form).await;
      match result {
        Ok(o) => {
          comment_view.set(o.comment_view);
        }
        Err(_e) => {}
      }
    });
  };

  let on_get_click = move |e: MouseEvent| {
    e.stop_propagation();
    spawn_local_scoped_with_cancellation(async move {
      let form = GetComment { id: comment_view.get().comment.id };
      let result = LemmyClient.get_comment(form).await;
      match result {
        Ok(o) => {
          comment_view.set(o.comment_view);
        }
        Err(_e) => {}
      }
    });
  };

  let on_mod_log_click = move |e: MouseEvent| {
    e.stop_propagation();
    spawn_local_scoped_with_cancellation(async move {
      let form = GetModlog {
        comment_id: Some(comment_view.get().comment.id),
        community_id: None,
        limit: None,
        mod_person_id: None,
        other_person_id: None,
        page: None,
        post_id: None,
        type_: None,
      };
      let result = LemmyClient.get_mod_log(form).await;
      match result {
        Ok(o) => {
          if let Some(c) = o.removed_comments.get(0) {
            comment_view.update(|u| {
              u.comment = c.comment.clone();
            });
          }
        }
        Err(_e) => {}
      }
    });
  };

  let on_reply_click = move |e: MouseEvent| {
    e.prevent_default();
    spawn_local_scoped_with_cancellation(async move {
      loading.set(true);
      let form = CreateComment {
        content: reply_content.get(),
        post_id: comment_view.get().comment.post_id,
        parent_id: Some(comment_view.get().comment.id),
        language_id: None,
      };
      let result = LemmyClient.reply_comment(form).await;
      match result {
        Ok(o) => {
          loading.set(false);
          now_in_millis.set(chrono::offset::Utc::now().timestamp_millis() as u64);
          children.update(|cs| cs.push(o.comment_view));
          reply_show.set(false);
          #[cfg(not(feature = "ssr"))]
          if let Ok(d) = IndexedDb::new().await {
            if let Ok(_c) = d.del(&CommentDraftKey { comment_id: comment_view.get().comment.id.0, draft: Draft::Reply }).await {}
          }
        }
        Err(_e) => {
          loading.set(false);
        }
      }
    });
  };

  let on_edit_click = move |e: MouseEvent| {
    e.prevent_default();
    spawn_local_scoped_with_cancellation(async move {
      loading.set(true);
      let form = EditComment { content: Some(edit_content.get()), comment_id: comment_view.get().comment.id, language_id: None };
      let result = LemmyClient.edit_comment(form).await;
      match result {
        Ok(_o) => {
          loading.set(false);
          edit_show.set(false);
          #[cfg(not(feature = "ssr"))]
          if let Ok(d) = IndexedDb::new().await {
            if let Ok(_c) = d.del(&CommentDraftKey { comment_id: comment_view.get().comment.id.0, draft: Draft::Edit }).await {}
          }
        }
        Err(_e) => {
          loading.set(false);
        }
      }
    });
  };

  let on_cancel_click = move |e: MouseEvent| {
    e.prevent_default();
    comment_view.update(|cv| cv.comment.content = comment_copy.get().comment.content);
    edit_show.set(false);
  };

  let _visibility_element = NodeRef::<Textarea>::new();

  #[cfg(not(feature = "ssr"))]
  {
    use_intersection_observer_with_options(
      _visibility_element,
      move |_entries, _io| {
        if let Some(v) = _visibility_element.get() {
          v.focus();
        }
      },
      UseIntersectionObserverOptions::default(),
    );
  }

  view! {
    <div class={move || {
      format!(
        "{}{}{}",
        if level > 8 { "" } else { "pl-4" },
        if level == 1 { " odd:bg-base-200 pr-4 pt-2 pb-1" } else { "" },
        if !hidden_comments.get().contains(&parent_comment_id) { "" } else { " hidden" },
      )
    }}>
      <div
        class={move || {
          format!(
            "pb-2 cursor-pointer{}{}",
            if comment_view.get().creator.id.eq(&comment_view.get().post.creator_id) { " border-l-4 pl-2 border-accent" } else { "" },
            if highlight_user_id.get().is_some() && highlight_user_id.get().eq(&Some(comment_view.get().creator.id)) {
              " border-l-4 pl-2"
            } else if let Some(v) = comment_view.get().my_vote {
              if v == 1 { " border-l-4 pl-2 border-secondary" } else if v == -1 { " border-l-4 pl-2 border-primary" } else { "" }
            } else {
              ""
            },
          )
        }}
        on:click={move |e: MouseEvent| {
          if still_down.get() {
            still_down.set(false);
          } else {
            if let Some(t) = e.target() {
              if let Some(i) = t.dyn_ref::<HtmlImageElement>() {
                let _ = window().open_with_url_and_target(&i.src(), "_blank");
              } else if let Some(l) = t.dyn_ref::<HtmlAnchorElement>() {
                let _ = window().open_with_url_and_target(&l.href(), "_blank");
                e.prevent_default();
              } else if let Some(s) = t.dyn_ref::<web_sys::Element>() {
                if s.tag_name().eq("SUMMARY") {} else {
                  on_toggle(comment_view.get().comment.id.0);
                }
              } else {
                on_toggle(comment_view.get().comment.id.0);
              }
            }
          }
        }}
        on:pointerdown={move |e: PointerEvent| {
          if e.buttons() == 1 {
            still_handle
              .set(
                set_timeout_with_handle(
                    move || {
                      vote_show.set(!vote_show.get());
                      still_down.set(true);
                    },
                    std::time::Duration::from_millis(500),
                  )
                  .ok(),
              );
          } else {
            if let Some(h) = still_handle.get() {
              h.clear();
            }
          }
        }}
        on:pointerup={move |_e: PointerEvent| {
          if let Some(h) = still_handle.get() {
            h.clear();
          }
        }}
        on:pointerleave={move |_e: PointerEvent| {
          if let Some(h) = still_handle.get() {
            h.clear();
          }
        }}
        on:pointermove={move |_e: PointerEvent| {
          if let Some(h) = still_handle.get() {
            h.clear();
          }
        }}
        on:dblclick={move |_e: MouseEvent| {
          vote_show.set(!vote_show.get());
        }}
      >
        <Show
          when={move || !(comment_view.get().creator_banned_from_community || comment_view.get().creator.banned)}
          fallback={move || view! { <Icon on:click={on_mod_log_click} icon={Hammer} /> }}
        >
          <Show when={move || !(comment_view.get().comment.removed)} fallback={move || view! { <Icon on:click={on_mod_log_click} icon={Block} /> }}>
            <Show when={move || !(comment_view.get().comment.deleted)} fallback={move || view! { <Icon icon={Eraser} /> }}>
              <Show
                when={move || !(comment_view.get().creator_blocked)}
                fallback={move || view! { <Icon on:click={on_get_click} icon={EyeSlash} /> }}
              >
                {()}
              </Show>
            </Show>
          </Show>
        </Show>

        <div class={move || format!("prose select-none{}", if highlight_show.get() { " brightness-200" } else { "" })} inner_html={safe_html} />

        <Show when={move || vote_show.get()} fallback={|| view! {}}>
          <div on:click={cancel} class="flex flex-wrap gap-x-2 items-center break-inside-avoid">
            <Form action="POST" attr:class="flex items-center">
              <input type="hidden" name="post_id" value={format!("{}", comment_view.get().post.id)} />
              <input type="hidden" name="score" value={move || if Some(1) == comment_view.get().my_vote { 0 } else { 1 }} />
              <button
                type="submit"
                class={move || {
                  format!(
                    "{}{}",
                    { if Some(1) == comment_view.get().my_vote { "text-secondary" } else { "" } },
                    { if Some(true) != logged_in.get() || !online.get().0 { " text-base-content/50" } else { " hover:text-secondary/50" } },
                  )
                }}
                title="Up vote"
                disabled={move || Some(true) != logged_in.get() || !online.get().0}
                on:click={on_up_vote_submit}
              >
                <Icon icon={Upvote} />
              </button>
            </Form>
            <span class="text-sm">{move || comment_view.get().counts.score}</span>
            <Form action="POST" attr:class="flex items-center">
              <input type="hidden" name="post_id" value={format!("{}", comment_view.get().post.id)} />
              <input type="hidden" name="score" value={move || if Some(-1) == comment_view.get().my_vote { 0 } else { -1 }} />
              <button
                type="submit"
                class={move || {
                  format!(
                    "{}{}",
                    { if Some(-1) == comment_view.get().my_vote { "text-primary" } else { "" } },
                    { if Some(true) != logged_in.get() || !online.get().0 { " text-base-content/50" } else { " hover:text-primary/50" } },
                  )
                }}
                title="Down vote"
                disabled={move || Some(true) != logged_in.get() || !online.get().0}
                on:click={on_down_vote_submit}
              >
                <Icon icon={Downvote} />
              </button>
            </Form>
            <Form action="POST" attr:class="flex items-center">
              <input type="hidden" name="post_id" value={format!("{}", comment_view.get().post.id)} />
              <input type="hidden" name="save" value={move || format!("{}", !comment_view.get().saved)} />
              <button
                type="submit"
                title="Save"
                class={move || {
                  format!(
                    "{}{}",
                    { if comment_view.get().saved { "text-accent" } else { "" } },
                    { if Some(true) != logged_in.get() || !online.get().0 { " text-base-content/50" } else { " hover:text-accent/50" } },
                  )
                }}
                disabled={move || Some(true) != logged_in.get() || !online.get().0}
                on:click={on_save_submit}
              >
                <Icon icon={Save} />
              </button>
            </Form>
            <button
              on:click={move |_| {
                edit_show.set(false);
                reply_show.update(|b| *b = !*b);
                spawn_local_scoped_with_cancellation(async move {
                  #[cfg(not(feature = "ssr"))]
                  if let Ok(d) = IndexedDb::new().await {
                    if let Ok(Some(c)) = d
                      .get(
                        &CommentDraftKey {
                          comment_id: comment_view.get().comment.id.0,
                          draft: Draft::Reply,
                        },
                      )
                      .await
                    {
                      reply_content.set(c);
                    }
                  }
                });
              }}
              title="Reply"
              class={move || {
                format!("{}", { if Some(true) != logged_in.get() || !online.get().0 { " text-base-content/50" } else { " hover:text-accent/50" } })
              }}
              disabled={move || Some(true) != logged_in.get() || !online.get().0}
            >
              <Icon icon={Reply} />
            </button>
            <button
              on:click={move |_| {
                reply_show.set(false);
                edit_show.update(|b| *b = !*b);
                spawn_local_scoped_with_cancellation(async move {
                  #[cfg(not(feature = "ssr"))]
                  if let Ok(d) = IndexedDb::new().await {
                    if let Ok(Some(c)) = d
                      .get(
                        &CommentDraftKey {
                          comment_id: comment_view.get().comment.id.0,
                          draft: Draft::Edit,
                        },
                      )
                      .await
                    {
                      edit_content.set(c);
                    } else {
                      edit_content.set(comment_view.get_untracked().comment.content);
                    }
                  }
                });
              }}
              class={move || {
                format!(
                  "{}{}",
                  if current_person.get().eq(&Some(comment_view.get().creator)) { "" } else { "pointer-events-none text-base-content/50" },
                  { if Some(true) != logged_in.get() || !online.get().0 { " text-base-content/50" } else { " hover:text-accent/50" } },
                )
              }}
              disabled={move || Some(true) != logged_in.get() || !online.get().0}
              title="Edit"
            >
              <Icon icon={Pencil} />
            </button>
            <span
              on:click={move |_| {
                if highlight_user_id.get().eq(&Some(comment_view.get().creator.id)) {
                  highlight_user_id.set(None)
                } else {
                  highlight_user_id.set(Some(comment_view.get().creator.id))
                }
              }}
              title="Highlight"
            >
              <Icon icon={Highlighter} />
            </span>
            <span class="overflow-hidden wrap-anywhere">
              <span>{abbr_duration.clone()}</span>
              " ago, by "
              <a href={move || format!("{}", comment_view.get().creator.actor_id)} target="_blank" class="text-sm hover:text-secondary">
                <span inner_html={html_escape::encode_safe(&comment_view.get().creator.actor_id.to_string()[8..]).to_string()} />
              </a>
            </span>
          </div>
        </Show>
        <Show
          when={move || hidden_comments.get().contains(&comment_view.get().comment.id.0) && (children.get().len() + descendants.get().len()) > 0}
          fallback={|| {}}
        >
          <span class="inline-block whitespace-nowrap badge badge-neutral">{children.get().len() + descendants.get().len()} " replies"</span>
        </Show>
      </div>
      <Show when={move || reply_show.get() || edit_show.get()} fallback={|| {}}>
        <div class="mb-3 space-y-3 break-inside-avoid">
          <Show when={move || reply_show.get()} fallback={|| {}}>
            <div class="form-control">
              <textarea
                class="h-24 text-base textarea textarea-bordered"
                placeholder="Comment text"
                prop:value={move || reply_content.get()}
                node_ref={_visibility_element}
                on:wheel={move |e: WheelEvent| {
                  e.stop_propagation();
                }}
                on:input={move |ev| {
                  reply_content.set(event_target_value(&ev));
                  spawn_local_scoped_with_cancellation(async move {
                    #[cfg(not(feature = "ssr"))]
                    if let Ok(d) = IndexedDb::new().await {
                      if let Ok(_comment_ids) = d
                        .set(
                          &CommentDraftKey {
                            comment_id: comment_view.get().comment.id.0,
                            draft: Draft::Reply,
                          },
                          &reply_content.get(),
                        )
                        .await
                      {} else {}
                    }
                  });
                }}
              >
                {reply_content.get_untracked()}
              </textarea>
            </div>
            <div class="form-control">
              <button
                on:click={on_reply_click}
                type="button"
                disabled={move || Some(true) != logged_in.get() || !online.get().0}
                class={move || format!("btn btn-neutral{}", if loading.get() { " btn-disabled" } else { "" })}
              >
                "Reply"
              </button>
              <button on:click={move |_| reply_show.set(false)} type="button" class="btn btn-neutral">
                "Cancel"
              </button>
            </div>
          </Show>
          <Show when={move || edit_show.get()} fallback={|| {}}>
            <div class="form-control">
              <textarea
                class="h-24 text-base textarea textarea-bordered"
                placeholder="Comment text"
                prop:value={move || edit_content.get()}
                on:wheel={move |e: WheelEvent| {
                  e.stop_propagation();
                }}
                on:input={move |ev| {
                  edit_content.set(event_target_value(&ev));
                  comment_view.update(|cv| cv.comment.content = event_target_value(&ev));
                  spawn_local_scoped_with_cancellation(async move {
                    #[cfg(not(feature = "ssr"))]
                    if let Ok(d) = IndexedDb::new().await {
                      if let Ok(_comment_ids) = d
                        .set(
                          &CommentDraftKey {
                            comment_id: comment_view.get().comment.id.0,
                            draft: Draft::Edit,
                          },
                          &edit_content.get(),
                        )
                        .await
                      {}
                    }
                  });
                }}
              >
                {edit_content.get_untracked()}
              </textarea>
            </div>
            <div class="form-control">
              <button
                on:click={on_edit_click}
                type="button"
                disabled={move || Some(true) != logged_in.get() || !online.get().0}
                class={move || format!("btn btn-neutral{}", if loading.get() { " btn-disabled" } else { "" })}
              >
                "Edit"
              </button>
              <button on:click={on_cancel_click} type="button" class="btn btn-neutral">
                "Cancel"
              </button>
            </div>
          </Show>
        </div>
      </Show>
      <For each={move || children.get()} key={|cv| cv.comment.id} let:cv>
        <Comment
          parent_comment_id={comment_view.get().comment.id.0}
          hidden_comments={hidden_comments}
          comment={cv.into()}
          comments={descendants.get().into()}
          level={level + 1usize}
          now_in_millis
          highlight_user_id
          post_id
        />
      </For>
    </div>
  }
  .into_any()
}
