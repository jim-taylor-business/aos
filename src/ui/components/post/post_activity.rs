use crate::{
  errors::{message_from_error, LemmyAppError, LemmyAppErrorType},
  lemmy_client::*,
  ui::components::{comment::comment_nodes::CommentNodes, post::post_listing::PostListing},
  // TitleSetter,
};
use ev::MouseEvent;
use lemmy_api_common::{
  comment::{CreateComment, GetComments},
  lemmy_db_schema::{newtypes::PostId, CommentSortType},
  post::GetPost,
  site::GetSiteResponse,
};
use leptos::*;
use leptos_meta::*;
use leptos_router::use_params_map;
use web_sys::{
  wasm_bindgen::{JsCast, JsValue},
  HtmlAnchorElement, HtmlImageElement,
};

#[component]
pub fn PostActivity(ssr_site: Resource<Option<bool>, Result<GetSiteResponse, LemmyAppError>>) -> impl IntoView {
  let params = use_params_map();
  let post_id = move || params.get().get("id").cloned().unwrap_or_default().parse::<i32>().ok();
  let error = expect_context::<RwSignal<Vec<Option<(LemmyAppError, Option<RwSignal<bool>>)>>>>();
  // let title = expect_context::<RwSignal<Option<TitleSetter>>>();

  let reply_show = RwSignal::new(false);
  let refresh_comments = RwSignal::new(false);
  let content = RwSignal::new(String::default());
  let loading = RwSignal::new(true);
  let refresh = RwSignal::new(false);

  let post_resource = Resource::new(
    move || (refresh.get(), post_id()),
    move |(_refresh, id_string)| async move {
      if let Some(id) = id_string {
        let form = GetPost {
          id: Some(PostId(id)),
          comment_id: None,
        };
        let result = LemmyClient.get_post(form).await;
        loading.set(false);
        match result {
          Ok(o) => Ok(o),
          Err(e) => {
            error.update(|es| es.push(Some((e.clone(), None))));
            Err((e, Some(refresh)))
          }
        }
      } else {
        Err((
          LemmyAppError {
            error_type: LemmyAppErrorType::ParamsError,
            content: "".into(),
          },
          None,
        ))
      }
    },
  );

  let comments = Resource::new(
    move || (refresh.get(), post_id(), refresh_comments.get()),
    move |(_refresh, post_id, _refresh_comments)| async move {
      if let Some(id) = post_id {
        let form = GetComments {
          post_id: Some(PostId(id)),
          community_id: None,
          type_: None,
          sort: Some(CommentSortType::Top),
          max_depth: Some(8),
          page: None,
          limit: None,
          community_name: None,
          parent_id: None,
          saved_only: None,
          disliked_only: None,
          liked_only: None,
        };
        let result = LemmyClient.get_comments(form).await;
        match result {
          Ok(o) => Some(o),
          Err(e) => {
            error.update(|es| es.push(Some((e, None))));
            None
          }
        }
      } else {
        None
      }
    },
  );

  let on_reply_click = move |ev: MouseEvent| {
    ev.prevent_default();
    create_local_resource(
      move || (),
      move |()| async move {
        if let Some(id) = post_id() {
          let form = CreateComment {
            content: content.get(),
            post_id: PostId(id),
            parent_id: None,
            language_id: None,
          };
          let result = LemmyClient.reply_comment(form).await;
          match result {
            Ok(_o) => {
              refresh_comments.update(|b| *b = !*b);
              reply_show.update(|b| *b = !*b);
            }
            Err(e) => {
              error.update(|es| es.push(Some((e, None))));
            }
          }
        }
      },
    );
  };

  // #[cfg(not(feature = "ssr"))]
  // {
  //   window().history().map(|h| h.replace_state(&JsValue::default(), "bell end"));
  // }

  // let rez = create_local_resource(
  //   move || (),
  //   move |()| async move {
  //     std::thread::sleep(std::time::Duration::from_millis(100));
  //   },
  // );
  //
  // let rez = RwSignal::new(None);

  // #[cfg(not(feature = "ssr"))]
  // {
  //   set_timeout(move || rez.set(Some(())), std::time::Duration::from_millis(100));
  // }

  view! {
    // <Transition fallback={|| {}}>
      // {move || {
      //   match rez.get() {
      //     Some(()) => {
      //       Some(view! {
      //         <Title text="Fuck" />
      //       })
      //     },
      //     None => {
      //       Some(view! {
      //         <Title text="" />
      //       })
      //     }
      //   }
      // }}
    // </Transition>

    // <Title text={move || { post_resource.get().map(|r| r.ok().map(|p| p.post_view.post.name).unwrap_or("Post".into())).unwrap_or("Post".into()) }} />
    <main role="main" class="flex flex-col flex-grow w-full">
      <div class="flex flex-col">
        <div>
          <Transition fallback={|| {}}>
            {move || {
              match post_resource.get() {
                Some(Err(err)) => {
                  Some(
                    view! {
                      <Title text="Error loading post" />
                      <div class="py-4 px-8">
                        <div class="flex justify-between alert alert-error">
                          <span>{message_from_error(&err.0)} " - " {err.0.content}</span>
                          <div>
                            <Show when={move || { if let Some(_) = err.1 { true } else { false } }} fallback={|| {}}>
                              <button
                                on:click={move |_| {
                                  if let Some(r) = err.1 {
                                    r.set(!r.get());
                                  } else {}
                                }}
                                class="btn btn-sm"
                              >
                                "Retry"
                              </button>
                            </Show>
                          </div>
                        </div>
                      </div>
                      <div class="hidden" />
                    },
                  )
                }
                Some(Ok(res)) => {
                  // title.set(Some(TitleSetter(res.post_view.post.name.clone())));
                  let text = if let Some(b) = res.post_view.post.body.clone() {
                    if b.len() > 0 { Some(b) } else { res.post_view.post.embed_description.clone() }
                  } else {
                    None
                  };
                  Some(
                    view! {
                      <Title text=res.post_view.post.name.clone() />
                      // {loading
                      // .get()
                      // .then(move || {
                      // view! {
                      // <div class="overflow-hidden animate-[popdown_1s_step-end_1]">
                      // <div class="py-4 px-8">
                      // <div class="alert">
                      // <span>"Loading"</span>
                      // </div>
                      // </div>
                      // </div>
                      // <div class="hidden" />

                      // }
                      // })}
                      <div>
                        <PostListing post_view={res.post_view.into()} ssr_site post_number=0 reply_show />
                      </div>
                      {if let Some(ref content) = text {
                        let mut options = pulldown_cmark::Options::empty();
                        options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
                        options.insert(pulldown_cmark::Options::ENABLE_TABLES);
                        let parser = pulldown_cmark::Parser::new_ext(content, options);
                        let custom = parser
                          .map(|event| match event {
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
                        Some(
                          view! {
                            <div class="pr-4 pl-4">
                              <div
                                class="py-2"
                                on:click={move |e: MouseEvent| {
                                  if let Some(t) = e.target() {
                                    if let Some(i) = t.dyn_ref::<HtmlImageElement>() {
                                      let _ = window().location().set_href(&i.src());
                                    } else if let Some(_l) = t.dyn_ref::<HtmlAnchorElement>() {}
                                  }
                                }}
                              >
                                <div class="max-w-none prose" inner_html={safe_html} />
                              </div>
                            </div>
                          },
                        )
                      } else {
                        None
                      }}
                      <Show when={move || reply_show.get()} fallback={|| {}}>
                        <div class="mb-3 space-y-3">
                          <label class="form-control">
                            <textarea
                              class="h-24 text-base textarea textarea-bordered"
                              placeholder="Comment text"
                              prop:value={move || content.get()}
                              on:input={move |ev| content.set(event_target_value(&ev))}
                            >
                              {content.get_untracked()}
                            </textarea>
                          </label>
                          <button on:click={on_reply_click} type="button" class="btn btn-neutral">
                            "Comment"
                          </button>
                        </div>
                      </Show>
                    },
                  )
                }
                None => {
                  Some(
                    view! {
                      <Title text="Loading post" />
                      <div class="overflow-hidden animate-[popdown_1s_step-end_1]">
                        <div class="py-4 px-8">
                          <div class="alert">
                            <span>"Loading"</span>
                          </div>
                        </div>
                      </div>
                      <div class="hidden" />
                    },
                  )
                }
              }
            }}
          </Transition>
          <Transition fallback={|| {}}>
            {move || {
              comments
                .get()
                .unwrap_or(None)
                .map(|res| {
                  view! {
                    <div class="w-full">
                      <CommentNodes ssr_site comments={res.comments.into()} _post_id={post_id().into()} />
                    </div>
                  }
                })
            }}
          </Transition>
        </div>
      </div>
    </main>
  }
}
