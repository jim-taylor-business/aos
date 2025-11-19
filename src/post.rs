use crate::{
  OnlineSetter, ReadInstanceCookie, WriteInstanceCookie,
  client::*,
  comments::Comments,
  db::csr_indexed_db::*,
  errors::{LemmyAppError, LemmyAppErrorType},
  nav::TopNav,
  toolbar::PostToolbar,
};
use ev::MouseEvent;
use lemmy_api_common::{
  comment::{CreateComment, GetComments},
  lemmy_db_schema::{CommentSortType, SortType, newtypes::PostId},
  post::{GetPost, GetPostResponse},
  site::GetSiteResponse,
};
use leptos::{
  html::{Div, Textarea},
  logging::log,
  prelude::*,
  task::*,
  *,
};
use leptos_meta::*;
use leptos_router::{components::A, hooks::*};
use leptos_use::{UseIntersectionObserverOptions, use_intersection_observer_with_options};
use web_sys::{HtmlAnchorElement, HtmlImageElement, WheelEvent, wasm_bindgen::JsCast};

#[component]
pub fn Post() -> impl IntoView {
  let ssr_site_signal = expect_context::<RwSignal<Option<Result<GetSiteResponse, LemmyAppError>>>>();

  let params = use_params_map();
  let query = use_query_map();

  let post_id = Signal::derive(move || params.get().get("id").unwrap_or_default().parse::<i32>().ok());
  let logged_in =
    Signal::derive(move || if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site_signal.get() { Some(true) } else { Some(false) });
  let online = expect_context::<RwSignal<OnlineSetter>>();

  // let scroll_element = expect_context::<RwSignal<Option<NodeRef<Div>>>>();
  // scroll_element.set(None);

  let ssr_sort = move || serde_json::from_str::<CommentSortType>(&query.get().get("sort").unwrap_or("".into())).unwrap_or(CommentSortType::Top);

  let reply_show = RwSignal::new(false);
  let content = RwSignal::new(String::default());
  let loading = RwSignal::new(true);

  let post_view = RwSignal::new(None::<GetPostResponse>);

  let post_resource = Resource::new(
    move || post_id.get(),
    move |id_string| async move {
      if let Some(id) = id_string {
        let form = GetPost { id: Some(PostId(id)), comment_id: None };
        let result = LemmyClient.get_post(form.clone()).await;
        loading.set(false);
        match result {
          Ok(o) => Some(Ok((form, o))),
          Err(e) => Some(Err(e)),
        }
      } else {
        None
      }
    },
  );

  let comments_resource = Resource::new(
    move || (post_id.get(), ssr_sort()),
    move |(post_id, sort_type)| async move {
      if let Some(id) = post_id {
        let form = GetComments {
          post_id: Some(PostId(id)),
          community_id: None,
          type_: None,
          sort: Some(sort_type),
          max_depth: Some(128),
          page: None,
          limit: None,
          community_name: None,
          parent_id: None,
          saved_only: None,
          disliked_only: None,
          liked_only: None,
        };
        let result = LemmyClient.get_comments(form.clone()).await;
        match result {
          Ok(o) => Some((form, o)),
          Err(_e) => None,
        }
      } else {
        None
      }
    },
  );

  let _on_sort_click = move |s: CommentSortType| {
    move |_e: MouseEvent| {
      let r = serde_json::to_string::<CommentSortType>(&s);
      let mut query_params = query.get();
      match r {
        Ok(o) => {
          query_params.insert("sort", o);
        }
        Err(e) => {}
      }
      if CommentSortType::Top == s {
        query_params.remove("sort");
      }
      let navigate = use_navigate();
      navigate(&format!("{}{}", use_location().pathname.get(), query_params.to_query_string()), Default::default());
    }
  };

  let on_reply_click = move |e: MouseEvent| {
    e.prevent_default();
    spawn_local_scoped_with_cancellation(async move {
      if let Some(id) = post_id.get() {
        let form = CreateComment { content: content.get(), post_id: PostId(id), parent_id: None, language_id: None };
        let result = LemmyClient.reply_comment(form).await;
        match result {
          Ok(_o) => {
            comments_resource.refetch();
            reply_show.update(|b| *b = !*b);
            #[cfg(not(feature = "ssr"))]
            if let Ok(d) = IndexedDb::new().await {
              if let Ok(_c) = d.del(&CommentDraftKey { comment_id: id, draft: Draft::Post }).await {}
            }
          }
          Err(_e) => {}
        }
      }
    });
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

  let on_scroll_element = NodeRef::<Div>::new();
  let thumbnail = RwSignal::new(String::from(""));
  let ReadInstanceCookie(get_instance_cookie) = expect_context::<ReadInstanceCookie>();

  view! {
    <main class="flex flex-col">
      <TopNav scroll_element=on_scroll_element.into() default_sort={SortType::TopAll.into()} post_view />//={post_view.into()} />
      <div class="flex flex-grow">
        <div
          on:wheel={move |e: WheelEvent| {
            if let Some(se) = on_scroll_element.get() {
              se.set_scroll_left(se.scroll_left() + e.delta_y() as i32);
            }
          }}
          node_ref={on_scroll_element}
          class="gap-4 min-w-full sm:overflow-x-auto sm:overflow-y-hidden sm:absolute sm:px-4 sm:h-[calc(100%-4rem)] sm:columns-sm"
          style="column-fill: auto"
        >
          <div>
            <Transition fallback={|| {}}>
              {move || {
                match post_resource.get() {
                  Some(Some(Err(LemmyAppError { error_type: LemmyAppErrorType::OfflineError, .. }))) => {
                    view! {
                      <Title text="Error loading post" />
                      <div class="py-4 px-8">
                        <div class="flex justify-between alert alert-warning alert-soft">
                          <span>"Offline"</span>
                          <div>
                            <button
                              on:click={move |_| {
                                post_resource.refetch();
                                comments_resource.refetch();
                              }}
                              class="btn btn-sm"
                            >
                              "Retry"
                            </button>
                          </div>
                        </div>
                      </div>
                    }
                      .into_any()
                  }
                  Some(Some(Err(_))) => {
                    view! {
                      <Title text="Error loading post" />
                      <div class="py-4 px-8">
                        <div class="flex justify-between alert alert-error alert-soft">
                          <span>"Error"</span>
                          <div>
                            <button
                              on:click={move |_| {
                                post_resource.refetch();
                                comments_resource.refetch();
                              }}
                              class="btn btn-sm"
                            >
                              "Retry"
                            </button>
                          </div>
                        </div>
                      </div>
                    }
                      .into_any()
                  }
                  Some(Some(Ok(res))) => {
                    #[cfg(not(feature = "ssr"))]
                    {
                      let rw = res.1.clone();
                      let fm = res.0.clone();
                      use crate::db::csr_indexed_db::*;
                      spawn_local_scoped_with_cancellation(async move {
                        if let Ok(d) = IndexedDb::new().await {
                          if let Ok(_c) = d.set(&fm, &rw).await {}
                        }
                      });
                    }
                    let res2 = res.1.clone();
                    let res = res.1.clone();
                    post_view.set(Some(res));
                    let post_response = RwSignal::new(res2);
                    let text = if let Some(b) = post_response.get().post_view.post.body.clone() {
                      if b.len() > 0 { Some(b) } else { post_response.get().post_view.post.embed_description.clone() }
                    } else {
                      None
                    };
                    let title = post_response.get().post_view.post.name.clone();
                    let mut options = pulldown_cmark::Options::empty();
                    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
                    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
                    options.insert(pulldown_cmark::Options::ENABLE_SUPERSCRIPT);
                    options.insert(pulldown_cmark::Options::ENABLE_SUBSCRIPT);
                    options.insert(pulldown_cmark::Options::ENABLE_CONTAINER_EXTENSIONS);
                    let parser = pulldown_cmark::Parser::new_ext(&title, options);
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
                    let mut title_encoded = String::new();
                    pulldown_cmark::html::push_html(&mut title_encoded, custom);
                    let community_title = if post_response.get().post_view.community.local {
                      format!("{}", post_response.get().post_view.community.name)
                    } else {
                      format!(
                        "{}@{}",
                        post_response.get().post_view.community.name,
                        post_response.get().post_view.community.actor_id.inner().host().unwrap().to_string(),
                      )
                    };
                    let community_title_encoded = html_escape::encode_safe(&community_title).to_string();
                    let creator_name = &post_response.get().post_view.creator.actor_id.to_string()[8..];
                    let creator_name_encoded = html_escape::encode_safe(creator_name).to_string();
                    let now_in_millis = {
                      #[cfg(not(feature = "ssr"))] { chrono::offset::Utc::now().timestamp_millis() as u64 }
                      #[cfg(feature = "ssr")] { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64 }
                    };
                    let duration_in_text = pretty_duration::pretty_duration(
                      &std::time::Duration::from_millis(now_in_millis - post_response.get().post_view.post.published.timestamp_millis() as u64),
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
                    let url = Memo::new(move |_| post_response.get().post_view.post.url);
                    let thumbnail_url = Memo::new(move |_| post_response.get().post_view.post.thumbnail_url);

                    view! {
                      <Title text={post_response.get().post_view.post.name} />
                      <div>
                        <PostToolbar post_view={post_response.get().post_view.into()} reply_show content post_id />
                      </div>
                      <div class="py-2 px-4">
                        <span class="overflow-y-auto text-xl wrap-anywhere" inner_html={title_encoded} />
                        <span class="block mb-1 wrap-anywhere text-md">
                          <span>{abbr_duration}</span>
                          " ago by "
                          <a
                            href={move || format!("{}", post_response.get().post_view.creator.actor_id)}
                            target="_blank"
                            class="inline wrap-anywhere hover:text-secondary"
                          >
                            <span class="overflow-y-auto" inner_html={creator_name_encoded} />
                          </a>
                          " in "
                          <A
                            attr:class="inline wrap-anywhere hover:text-secondary"
                            href={if post_response.get().post_view.community.local {
                              format!("/c/{}", post_response.get().post_view.community.name)
                            } else {
                              format!(
                                "/c/{}@{}",
                                post_response.get().post_view.community.name,
                                post_response.get().post_view.community.actor_id.inner().host().unwrap().to_string(),
                              )
                            }}
                            on:click={move |e: MouseEvent| {
                              #[cfg(not(feature = "ssr"))]
                              spawn_local_scoped_with_cancellation(async move {
                                if let Ok(d) = IndexedDb::new().await {
                                  let _ = d
                                    .set(
                                      &ScrollPositionKey {
                                        path: use_location().pathname.get(),
                                        query: use_query_map().get().to_query_string(),
                                      },
                                      &0i32,
                                    )
                                    .await;
                                }
                              });
                              // if let Some(on_scroll_element) = scroll_element.get() {
                              //   if let Some(se) = on_scroll_element.get() {
                              //     se.set_scroll_left(0i32);
                              //   }
                              // }
                            }}
                          >
                            <span class="overflow-y-auto" inner_html={community_title_encoded} />
                          </A>
                          <span
                            class="overflow-y-auto"
                            inner_html={move || if let Some(d) = url.get() {
                              if let Some(f) = d.inner().host_str() {
                                if f.to_string().ne(&get_instance_cookie.get().unwrap_or("".into())) { format!(" from {}", f) } else { "".to_owned() }
                              } else {
                                "".to_owned()
                              }
                            } else {
                              "".to_owned()
                            }}
                          />
                        </span>
                      </div>
                      <a
                        class={move || {
                          format!(
                            "float-left{}",
                            if thumbnail_url.get().is_none()
                              && url.get().is_none()
                            {
                              " hidden"
                            } else {
                              ""
                            },
                          )
                        }}
                        target="_blank"
                        href={move || {
                          if let Some(d) = url.get() {
                            d.inner().to_string()
                          } else {
                            format!("/post/{}", post_response.get().post_view.post.id)
                          }
                        }}
                      >
                        {move || {
                          if let Some(t) = thumbnail_url.get() {
                            let h = t.inner().to_string();
                            thumbnail.set(h);
                            view! {
                              <div class="py-2 px-4">
                                <div class="block">
                                  <img
                                    loading="lazy"
                                    class={move || { format!("w-auto{}", if thumbnail.get().eq(&"/lemmy.svg".to_owned()) { " h-16" } else { "" }) }}
                                    src={move || thumbnail.get()}
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
                              <div class="py-2 px-4">
                                <div class="block">
                                  <img class="h-16" src="/lemmy.svg" />
                                </div>
                              </div>
                            }
                              .into_any()
                          }
                        }}
                      </a>

                      {if let Some(ref content) = text {
                        let mut options = pulldown_cmark::Options::empty();
                        options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
                        options.insert(pulldown_cmark::Options::ENABLE_TABLES);
                        options.insert(pulldown_cmark::Options::ENABLE_SUPERSCRIPT);
                        options.insert(pulldown_cmark::Options::ENABLE_SUBSCRIPT);
                        options.insert(pulldown_cmark::Options::ENABLE_CONTAINER_EXTENSIONS);
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
                            <div class="pr-4 pl-4 before:content-[''] before:block before:w-24 before:overflow-hidden">
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
                      <Show when={move || reply_show.get()} fallback={|| {}}>
                        <div class="mb-3 space-y-3 before:content-[''] before:block before:w-24 before:overflow-hidden">
                          <div class="form-control">
                            <textarea
                              class="h-24 text-base textarea textarea-bordered"
                              placeholder="Comment text"
                              prop:value={move || content.get()}
                              node_ref={_visibility_element}
                              on:wheel={move |e: WheelEvent| {
                                e.stop_propagation();
                              }}
                              on:input={move |ev| {
                                content.set(event_target_value(&ev));
                                if let Some(id) = post_id.get() {
                                  #[cfg(not(feature = "ssr"))]
                                  spawn_local_scoped_with_cancellation(async move {
                                    if let Ok(d) = IndexedDb::new().await {
                                      if let Ok(_c) = d
                                        .set(
                                          &CommentDraftKey {
                                            comment_id: id,
                                            draft: Draft::Post,
                                          },
                                          &content.get(),
                                        )
                                        .await
                                      {}
                                    }
                                  });
                                }
                              }}
                            >
                              {content.get_untracked()}
                            </textarea>
                          </div>
                          <div class="form-control">
                            <button
                              on:click={on_reply_click}
                              type="button"
                              class={move || {
                                format!(
                                  "btn btn-neutral{}",
                                  {
                                    if Some(true) != logged_in.get() || !online.get().0 { " text-base-content/50" } else { " hover:text-secondary/50" }
                                  },
                                )
                              }}
                              disabled={move || Some(true) != logged_in.get() || !online.get().0}
                            >
                              "Comment"
                            </button>
                            <button on:click={move |_| reply_show.set(false)} type="button" class="btn btn-neutral">
                              "Cancel"
                            </button>
                          </div>
                        </div>
                      </Show>
                    }
                      .into_any()
                  }
                  Some(None) | None => {
                    view! {
                      <div class="overflow-hidden animate-[popdown_1s_step-end_1]">
                        <div class="py-4 px-8">
                          <div class="alert alert-info alert-soft">
                            <span>"Loading"</span>
                          </div>
                        </div>
                      </div>
                    }
                      .into_any()
                  }
                }
              }}
            </Transition>
            <Transition fallback={|| {}}>
              {move || {
                comments_resource
                  .get()
                  .unwrap_or(None)
                  .map(|res| {
                    #[cfg(not(feature = "ssr"))]
                    {
                      let rw = res.1.clone();
                      let fm = res.0.clone();
                      use crate::db::csr_indexed_db::*;
                      spawn_local_scoped_with_cancellation(async move {
                        if let Ok(d) = IndexedDb::new().await {
                          if let Ok(_c) = d.set(&fm, &rw).await {}
                        }
                      });
                    }
                    let res = res.1.clone();

                    view! {
                      <div class="w-full before:content-[''] before:block before:w-24 before:overflow-hidden">
                        <Comments comments={res.comments.into()} post_id />
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
