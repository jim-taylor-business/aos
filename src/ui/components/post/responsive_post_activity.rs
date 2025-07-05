use crate::{
  errors::{message_from_error, LemmyAppError},
  lemmy_client::*,
  ui::components::{
    comment::responsive_comment_nodes::ResponsiveCommentNodes,
    common::responsive_nav::ResponsiveTopNav,
    post::{post_listing::PostListing, responsive_post_listing::ResponsivePostListing},
  },
};
use ev::MouseEvent;
use lemmy_api_common::{
  comment::{CreateComment, GetComments},
  lemmy_db_schema::{newtypes::PostId, CommentSortType, SortType},
  post::{GetPost, GetPostResponse},
  site::GetSiteResponse,
};
use leptos::{html::Div, *};
use leptos_meta::*;
use leptos_router::{use_location, use_params_map, use_query_map};
use web_sys::{wasm_bindgen::JsCast, HtmlAnchorElement, HtmlImageElement, WheelEvent};

#[component]
pub fn ResponsivePostActivity(ssr_site: Resource<Option<bool>, Result<GetSiteResponse, LemmyAppError>>) -> impl IntoView {
  let params = use_params_map();
  let query = use_query_map();

  let post_id = move || params.get().get("id").cloned().unwrap_or_default().parse::<i32>().ok();
  let error = expect_context::<RwSignal<Vec<Option<(LemmyAppError, Option<RwSignal<bool>>)>>>>();
  let ssr_sort =
    move || serde_json::from_str::<CommentSortType>(&query.get().get("sort").cloned().unwrap_or("".into())).unwrap_or(CommentSortType::Top);

  let reply_show = RwSignal::new(false);
  let refresh_comments = RwSignal::new(false);
  let content = RwSignal::new(String::default());
  let loading = RwSignal::new(true);
  let refresh = RwSignal::new(false);

  let post_view = RwSignal::new(None::<GetPostResponse>);

  #[cfg(not(feature = "ssr"))]
  if let Some(id) = post_id() {
    let form = CreateComment {
      content: "".into(),
      post_id: PostId(id),
      parent_id: None,
      language_id: None,
    };
    if let Ok(Some(s)) = window().local_storage() {
      if let Ok(Some(c)) = s.get_item(&serde_json::to_string(&form).ok().unwrap()) {
        content.set(c);
      }
    }
  }

  let post_resource = Resource::new(
    move || (refresh.get(), post_id()),
    move |(_refresh, id_string)| async move {
      // logging::log!("6 {:?} {}", id_string, _refresh);
      if let Some(id) = id_string {
        let form = GetPost {
          id: Some(PostId(id)),
          comment_id: None,
        };
        let result = LemmyClient.get_post(form).await;
        loading.set(false);
        match result {
          Ok(o) => Some(Ok(o)),
          Err(e) => {
            error.update(|es| es.push(Some((e.clone(), None))));
            Some(Err((e, Some(refresh))))
          }
        }
      } else {
        // Err((
        //   LemmyAppError {
        //     error_type: LemmyAppErrorType::ParamsError,
        //     content: "".into(),
        //   },
        None //,
             // ))
      }
    },
  );

  let comments = Resource::new(
    move || (refresh.get(), post_id(), ssr_sort(), refresh_comments.get()),
    move |(_refresh, post_id, sort_type, _refresh_comments)| async move {
      if let Some(id) = post_id {
        let form = GetComments {
          post_id: Some(PostId(id)),
          community_id: None,
          type_: None,
          sort: Some(sort_type),
          // sort: Some(CommentSortType::Top),
          max_depth: Some(128),
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

  let on_sort_click = move |s: CommentSortType| {
    move |_e: MouseEvent| {
      let r = serde_json::to_string::<CommentSortType>(&s);
      let mut query_params = query.get();
      match r {
        Ok(o) => {
          query_params.insert("sort".into(), o);
        }
        Err(e) => {
          error.update(|es| es.push(Some((e.into(), None))));
        }
      }
      if CommentSortType::Top == s {
        query_params.remove("sort".into());
      }
      // query_params.remove("from".into());
      // query_params.remove("prev".into());
      let navigate = leptos_router::use_navigate();
      navigate(
        &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
        Default::default(),
      );
    }
  };

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
              let form = CreateComment {
                content: "".into(),
                post_id: PostId(id),
                parent_id: None,
                language_id: None,
              };
              if let Ok(Some(s)) = window().local_storage() {
                let _ = s.delete(&serde_json::to_string(&form).ok().unwrap());
              }
            }
            Err(e) => {
              error.update(|es| es.push(Some((e, None))));
            }
          }
        }
      },
    );
  };

  let _visibility_element = create_node_ref::<leptos_dom::html::Textarea>();

  #[cfg(not(feature = "ssr"))]
  {
    leptos_use::use_intersection_observer_with_options(
      _visibility_element,
      move |_entries, _io| {
        let _ = _visibility_element.get().unwrap().focus();
      },
      leptos_use::UseIntersectionObserverOptions::default(),
    );
  }

  let on_scroll_element = NodeRef::<Div>::new();
  let thumbnail = RwSignal::new(String::from(""));

  view! {
    <main class="flex flex-col">
      <ResponsiveTopNav ssr_site default_sort=SortType::TopAll.into() post_view=post_view.into() />
      <div class="flex flex-grow">
        <div
        on:wheel=move |e: WheelEvent| {
          if let Some(se) = on_scroll_element.get() {
            se.set_scroll_left(se.scroll_left() + e.delta_y() as i32);
          }
        }
        node_ref=on_scroll_element class="sm:h-[calc(100%-4rem)] min-w-full sm:absolute sm:overflow-x-auto sm:overflow-y-hidden sm:columns-sm sm:px-4 gap-4">
      <div>
        <Transition fallback={|| {}}>
          {move || {
            match post_resource.get() {
              Some(Some(Err(err))) => {
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
              Some(Some(Ok(res))) => {
                post_view.set(Some(res.clone()));
                let text = if let Some(b) = res.post_view.post.body.clone() {
                  if b.len() > 0 { Some(b) } else { res.post_view.post.embed_description.clone() }
                } else {
                  None
                };
                Some(
                  view! {
                    <Title text={res.post_view.post.name.clone()} />
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

                    // <div>
                    //   <ResponsivePostListing post_view={res.post_view.into()} ssr_site post_number=0 reply_show />
                    // </div>
                    //
                    <a
                      class="float-left"
                      target="_blank"
                      href={move || { if let Some(d) = post_view.get().unwrap().post_view.post.url { d.inner().to_string() } else { format!("/responsive/post/{}", post_view.get().unwrap().post_view.post.id) } }}
                    >
                    {move || {
                      if let Some(t) = post_view.get().unwrap().post_view.post.thumbnail_url {
                        let h = t.inner().to_string();
                        thumbnail.set(h);
                        view! {
                          <div class="px-4 py-2">
                            <div class="block">
                              <img
                                class={move || format!("w-auto{}", if thumbnail.get().eq(&"/lemmy.svg".to_string()) { " h-16" } else { "" })}
                                src={move || thumbnail.get()}
                                // node_ref={thumbnail_element}
                                on:error={move |_e| {
                                  thumbnail.set("/lemmy.svg".into());
                                }}
                              />
                            </div>
                          </div>
                        }
                      } else {
                        view! {
                          <div class="px-4 py-2">
                            <div class="block">
                              <img class="h-16" src="/lemmy.svg" />
                            </div>
                          </div>
                        }
                      }
                    }}
                    </a>

                    {if let Some(ref content) = text {
                      let mut options = pulldown_cmark::Options::empty();
                      options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
                      options.insert(pulldown_cmark::Options::ENABLE_TABLES);
                      options.insert(pulldown_cmark::Options::ENABLE_SUPERSCRIPT);
                      options.insert(pulldown_cmark::Options::ENABLE_SUBSCRIPT);
                      options.insert(pulldown_cmark::Options::ENABLE_SPOILER);
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
                                    let _ = window().open_with_url_and_target(&i.src(), "_blank");
                                  } else if let Some(l) = t.dyn_ref::<HtmlAnchorElement>() {
                                    e.prevent_default();
                                    let _ = window().open_with_url_and_target(&l.href(), "_blank");
                                  }
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
                    // <div id="reply_box">
                    <Show when={move || reply_show.get()} fallback={|| {}}>
                      <div class="mb-3 space-y-3">
                        <label class="form-control">
                          <textarea
                            class="h-24 text-base textarea textarea-bordered"
                            placeholder="Comment text"
                            prop:value={move || content.get()}
                            node_ref={_visibility_element}
                            // id="reply_text"
                            // autofocus=true
                            on:input={move |ev| {
                              content.set(event_target_value(&ev));
                              if let Some(id) = post_id() {
                                let form = CreateComment {
                                  content: "".into(),
                                  post_id: PostId(id),
                                  parent_id: None,
                                  language_id: None,
                                };
                                if let Ok(Some(s)) = window().local_storage() {
                                  let _ = s.set_item(&serde_json::to_string(&form).ok().unwrap(), &event_target_value(&ev));
                                }
                              }
                            }}
                          >
                            {content.get_untracked()}
                          </textarea>
                        </label>
                        <button on:click={on_reply_click} type="button" class="btn btn-neutral">
                          "Comment"
                        </button>
                      </div>
                    // {
                    // let t = document().get_element_by_id("reply_text").unwrap().dyn_ref::<HtmlTextAreaElement>().unwrap().clone();
                    // // let d = document().get_element_by_id("reply_text").unwrap();
                    // // d.a;
                    // t.focus();
                    // }
                    </Show>
                  },
                )
              }
              Some(None) | None => {
                Some(
                  // </div>
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
                    // <div class="ml-3 sm:inline-block sm:ml-0 dropdown">
                    //   <label tabindex="0" class="btn">
                    //     "Sort"
                    //   </label>
                    //   <ul tabindex="0" class="shadow menu dropdown-content z-[1] bg-base-100 rounded-box">
                    //     <li
                    //       class={move || { (if CommentSortType::Top == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                    //       on:click={on_sort_click(CommentSortType::Top)}
                    //     >
                    //       <span>"Top"</span>
                    //     </li>
                    //     <li
                    //       class={move || { (if CommentSortType::Hot == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                    //       on:click={on_sort_click(CommentSortType::Hot)}
                    //     >
                    //       <span>"Hot"</span>
                    //     </li>
                    //     <li
                    //       class={move || { (if CommentSortType::New == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                    //       on:click={on_sort_click(CommentSortType::New)}
                    //     >
                    //       <span>"New"</span>
                    //     </li>
                    //     <li
                    //       class={move || { (if CommentSortType::Old == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                    //       on:click={on_sort_click(CommentSortType::Old)}
                    //     >
                    //       <span>"Old"</span>
                    //     </li>
                    //     <li
                    //       class={move || { (if CommentSortType::Controversial == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                    //       on:click={on_sort_click(CommentSortType::Controversial)}
                    //     >
                    //       <span>"Contraversial"</span>
                    //     </li>
                    //   </ul>
                    // </div>
                    <ResponsiveCommentNodes ssr_site comments={res.comments.into()} _post_id={post_id().into()} />
                  </div>
                }
              })
          }}
        </Transition>
      </div>
        </div>
      </div>
    </main>
  }
}
