use crate::{
  errors::LemmyAppError,
  lemmy_client::*,
  ui::components::common::icon::{Icon, IconType::*},
};
use ev::{MouseEvent, SubmitEvent, TouchEvent};
use lemmy_api_common::{
  comment::{CreateComment, CreateCommentLike, EditComment, SaveComment},
  lemmy_db_schema::{newtypes::PersonId, source::person::Person},
  lemmy_db_views::structs::{CommentView, LocalUserView},
  site::{GetSiteResponse, MyUserInfo},
};
use leptos::*;
use leptos_dom::helpers::TimeoutHandle;
use leptos_router::{Form, A};
use web_sys::{wasm_bindgen::JsCast, HtmlAnchorElement, HtmlImageElement};

#[component]
pub fn CommentNode(
  ssr_site: Resource<Option<bool>, Result<GetSiteResponse, LemmyAppError>>,
  comment: MaybeSignal<CommentView>,
  comments: MaybeSignal<Vec<CommentView>>,
  level: usize,
  parent_comment_id: i32,
  now_in_millis: u64,
  hidden_comments: RwSignal<Vec<i32>>,
  highlight_user_id: RwSignal<Option<PersonId>>,
  #[prop(into)] on_toggle: Callback<i32>,
) -> impl IntoView {
  let logged_in = Signal::derive(move || {
    if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site.get() {
      Some(true)
    } else {
      Some(false)
    }
  });

  let current_person = Signal::derive(move || {
    if let Some(Ok(GetSiteResponse {
      my_user: Some(MyUserInfo {
        local_user_view: LocalUserView { person, .. },
        ..
      }),
      ..
    })) = ssr_site.get()
    {
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

  let reply_content = RwSignal::new(String::default());

  let duration_in_text = pretty_duration::pretty_duration(
    &std::time::Duration::from_millis(now_in_millis - comment_view.get().post.published.timestamp_millis() as u64),
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

  let error = expect_context::<RwSignal<Vec<Option<(LemmyAppError, Option<RwSignal<bool>>)>>>>();

  let cancel = move |ev: MouseEvent| {
    ev.stop_propagation();
  };

  let on_vote_submit = move |ev: SubmitEvent, score: i16| {
    ev.prevent_default();
    create_local_resource(
      move || (),
      move |()| async move {
        let form = CreateCommentLike {
          comment_id: comment_view.get().comment.id,
          score,
        };
        let result = LemmyClient.like_comment(form).await;
        match result {
          Ok(o) => {
            comment_view.set(o.comment_view);
          }
          Err(e) => {
            error.update(|es| es.push(Some((e, None))));
          }
        }
      },
    );
  };

  let on_up_vote_submit = move |ev: SubmitEvent| {
    let score = if Some(1) == comment_view.get().my_vote { 0 } else { 1 };
    on_vote_submit(ev, score);
  };

  let on_down_vote_submit = move |ev: SubmitEvent| {
    let score = if Some(-1) == comment_view.get().my_vote { 0 } else { -1 };
    on_vote_submit(ev, score);
  };

  let on_save_submit = move |ev: SubmitEvent| {
    ev.prevent_default();
    create_local_resource(
      move || (),
      move |()| async move {
        let form = SaveComment {
          comment_id: comment_view.get().comment.id,
          save: !comment_view.get().saved,
        };
        let result = LemmyClient.save_comment(form).await;
        match result {
          Ok(o) => {
            comment_view.set(o.comment_view);
          }
          Err(e) => {
            error.update(|es| es.push(Some((e, None))));
          }
        }
      },
    );
  };

  let on_reply_click = move |ev: MouseEvent| {
    ev.prevent_default();
    create_local_resource(
      move || (),
      move |()| async move {
        let form = CreateComment {
          content: reply_content.get(),
          post_id: comment_view.get().comment.post_id,
          parent_id: Some(comment_view.get().comment.id),
          language_id: None,
        };
        let result = LemmyClient.reply_comment(form).await;
        match result {
          Ok(o) => {
            children.update(|cs| cs.push(o.comment_view));
            reply_show.set(false);
          }
          Err(e) => {
            error.update(|es| es.push(Some((e, None))));
          }
        }
      },
    );
  };

  let on_edit_click = move |ev: MouseEvent| {
    ev.prevent_default();
    create_local_resource(
      move || (),
      move |()| async move {
        let form = EditComment {
          content: Some(comment_view.get().comment.content),
          comment_id: comment_view.get().comment.id,
          language_id: None,
        };
        let result = LemmyClient.edit_comment(form).await;
        match result {
          Ok(o) => {
            edit_show.set(false);
          }
          Err(e) => {
            error.update(|es| es.push(Some((e, None))));
          }
        }
      },
    );
  };

  let on_cancel_click = move |ev: MouseEvent| {
    ev.prevent_default();
    comment_view.update(|cv| cv.comment.content = comment_copy.get().comment.content);
    edit_show.set(false);
  };

  view! {
    <div class={move || {
      format!(
        "pl-4{}{}",
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
              " border-l-4 pl-2" // border-yellow"
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
                let _ = window().location().set_href(&i.src());
              } else if let Some(_l) = t.dyn_ref::<HtmlAnchorElement>() {} else {
                on_toggle.call(comment_view.get().comment.id.0);
              }
            }
          }
        }}
        on:mousedown={move |_e: MouseEvent| {
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
        }}
        on:touchstart={move |_e: TouchEvent| {
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
        }}
        on:touchend={move |_e: TouchEvent| {
          if let Some(h) = still_handle.get() {
            h.clear();
          }
        }}
        on:touchmove={move |_e: TouchEvent| {
          if let Some(h) = still_handle.get() {
            h.clear();
          }
        }}
        on:mouseup={move |_e: MouseEvent| {
          if let Some(h) = still_handle.get() {
            h.clear();
          }
        }}
        on:dblclick={move |_e: MouseEvent| {
          vote_show.set(!vote_show.get());
        }}
      >
        // on:mouseover={move |e: MouseEvent| {
        // e.stop_propagation();
        // highlight_show.set(true);
        // }}
        // on:mouseout={move |e: MouseEvent| {
        // e.stop_propagation();
        // highlight_show.set(false);
        // }}
        <div class={move || format!("max-w-none prose{}", if highlight_show.get() { " brightness-200" } else { "" })} inner_html={safe_html} />
        <Show when={move || vote_show.get()} fallback={|| view! {}}>
          <div on:click={cancel} class="flex flex-wrap gap-x-2 items-center">
            <Form on:submit={on_up_vote_submit} action="POST" class="flex items-center">
              <input type="hidden" name="post_id" value={format!("{}", comment_view.get().post.id)} />
              <input type="hidden" name="score" value={move || if Some(1) == comment_view.get().my_vote { 0 } else { 1 }} />
              <button
                type="submit"
                class={move || {
                  format!(
                    "{}{}",
                    { if Some(1) == comment_view.get().my_vote { "text-secondary" } else { "" } },
                    { if Some(true) != logged_in.get() { " text-base-content/50" } else { " hover:text-secondary/50" } },
                  )
                }}
                title="Up vote"
                disabled={move || Some(true) != logged_in.get()}
              >
                <Icon icon={Upvote} />
              </button>
            </Form>
            <span class="text-sm">{move || comment_view.get().counts.score}</span>
            <Form on:submit={on_down_vote_submit} action="POST" class="flex items-center">
              <input type="hidden" name="post_id" value={format!("{}", comment_view.get().post.id)} />
              <input type="hidden" name="score" value={move || if Some(-1) == comment_view.get().my_vote { 0 } else { -1 }} />
              <button
                type="submit"
                class={move || {
                  format!(
                    "{}{}",
                    { if Some(-1) == comment_view.get().my_vote { "text-primary" } else { "" } },
                    { if Some(true) != logged_in.get() { " text-base-content/50" } else { " hover:text-primary/50" } },
                  )
                }}
                title="Down vote"
                disabled={move || Some(true) != logged_in.get()}
              >
                <Icon icon={Downvote} />
              </button>
            </Form>
            <Form action="POST" on:submit={on_save_submit} class="flex items-center">
              <input type="hidden" name="post_id" value={format!("{}", comment_view.get().post.id)} />
              <input type="hidden" name="save" value={move || format!("{}", !comment_view.get().saved)} />
              <button
                type="submit"
                title="Save comment"
                class={move || {
                  format!(
                    "{}{}",
                    { if comment_view.get().saved { "text-accent" } else { "" } },
                    { if Some(true) != logged_in.get() { " text-base-content/50" } else { " hover:text-accent/50" } },
                  )
                }}
                disabled={move || Some(true) != logged_in.get()}
              >
                <Icon icon={Save} />
              </button>
            </Form>
            <span on:click={move |_| { edit_show.set(false); reply_show.update(|b| *b = !*b); }} title="Reply">
              <Icon icon={Reply} />
            </span>
            <span on:click={move |_| { reply_show.set(false); edit_show.update(|b| *b = !*b); }} class=move || format!("{}", if current_person.get().eq(&Some(comment_view.get().creator)) { "" } else { "pointer-events-none text-base-content/50" }) title="Edit">
              <Icon icon={Pencil} />
            </span>
              <span  on:click={move |_| { if highlight_user_id.get().is_some() { highlight_user_id.set(None) } else { highlight_user_id.set(Some(comment_view.get().creator.id)) } }} title="Highlight">
              <Icon icon={Highlighter} />
            </span>
            <span class="overflow-hidden break-words">
              <span>{abbr_duration.clone()}</span>
              " ago, by "
              <A href={move || format!("{}", comment_view.get().creator.actor_id)} class="text-sm break-words hover:text-secondary">
              // let creator_name = post_view.get().creator.name.clone();
              // let creator_name_encoded = html_escape::encode_safe(&comment_view.get().creator.name).to_string();
                <span inner_html={html_escape::encode_safe(&comment_view.get().creator.name).to_string()} />

                // {comment_view.get().creator.name}
              </A>
            </span>
          </div>
        </Show>
        <span class={move || {
          format!(
            "badge badge-neutral inline-block whitespace-nowrap{}",
            if hidden_comments.get().contains(&comment_view.get().comment.id.0) && children.get().len() > 0 { "" } else { " hidden" },
          )
        }}>{children.get().len() + descendants.get().len()} " replies"</span>
      </div>
      <Show when={move || reply_show.get() || edit_show.get()} fallback={|| {}}>
        <div class="mb-3 space-y-3">
          // <label class="form-control">
          //   <textarea
          //     class="h-24 text-base textarea textarea-bordered"
          //     placeholder="Comment text"
          //     prop:value={move || content.get()}
          //     on:input={move |ev| content.set(event_target_value(&ev))}
          //   >
          //     {content.get_untracked()}
          //   </textarea>
          // </label>
          <Show when={move || reply_show.get()} fallback={|| {}}>
            <label class="form-control">
              <textarea
                class="h-24 text-base textarea textarea-bordered"
                placeholder="Comment text"
                prop:value={move || reply_content.get()}
                on:input={move |ev| reply_content.set(event_target_value(&ev))}
              >
                {reply_content.get_untracked()}
              </textarea>
            </label>
            <button on:click={on_reply_click} type="button" class="btn btn-neutral">
              "Reply"
            </button>
            <button on:click={move |_| reply_show.set(false)} type="button" class="btn btn-neutral">
              "Cancel"
            </button>
          </Show>
          <Show when={move || edit_show.get()} fallback={|| {}}>
            <label class="form-control">
              <textarea
                class="h-24 text-base textarea textarea-bordered"
                placeholder="Comment text"
                prop:value={move || comment_view.get_untracked().comment.content}
                on:input={move |ev| comment_view.update(|cv| cv.comment.content = event_target_value(&ev))}
              >
                {comment_view.get_untracked().comment.content}
              </textarea>
            </label>
            <button on:click={on_edit_click} type="button" class="btn btn-neutral">
              "Edit"
            </button>
            <button on:click={on_cancel_click} type="button" class="btn btn-neutral">
              "Cancel"
            </button>
          </Show>
      </div>
      </Show>
      <For each={move || children.get()} key={|cv| cv.comment.id} let:cv>
        <CommentNode
          ssr_site
          parent_comment_id={comment_view.get().comment.id.0}
          hidden_comments={hidden_comments}
          on_toggle
          comment={cv.into()}
          comments={descendants.get().into()}
          level={level + 1}
          now_in_millis
          highlight_user_id
        />
      </For>
    </div>
  }
}
