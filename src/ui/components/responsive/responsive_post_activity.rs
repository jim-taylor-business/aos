use crate::{
  errors::{message_from_error, LemmyAppError, LemmyAppErrorType},
  lemmy_client::*,
  ui::components::{
    post::post_listing::PostListing,
    responsive::{
      responsive_comment_nodes::ResponsiveCommentNodes, responsive_nav::ResponsiveTopNav, responsive_post_listing::ResponsivePostListing,
      responsive_post_toolbar::ResponsivePostToolbar,
    },
  },
  OnlineSetter,
};
use ev::MouseEvent;
use lemmy_api_common::{
  comment::{CreateComment, GetComments},
  lemmy_db_schema::{newtypes::PostId, CommentSortType, SortType},
  post::{GetPost, GetPostResponse},
  site::GetSiteResponse,
};
use leptos::{html::Div, logging::log, *};
use leptos_meta::*;
use leptos_router::{use_location, use_params_map, use_query_map, A};
use leptos_use::{use_cookie_with_options, FromToStringCodec, SameSite, UseCookieOptions};
use web_sys::{wasm_bindgen::JsCast, HtmlAnchorElement, HtmlImageElement, WheelEvent};

#[cfg(not(feature = "ssr"))]
use crate::indexed_db::csr_indexed_db::*;

#[component]
pub fn ResponsivePostActivity(ssr_site: Resource<(Option<String>, Option<String>), Result<GetSiteResponse, LemmyAppError>>) -> impl IntoView {
  let params = use_params_map();
  let query = use_query_map();

  let post_id = Signal::derive(move || params.get().get("id").cloned().unwrap_or_default().parse::<i32>().ok());
  // let error = expect_context::<RwSignal<Vec<Option<(LemmyAppError, Option<RwSignal<bool>>)>>>>();

  let logged_in = Signal::derive(move || {
    if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site.get() {
      Some(true)
    } else {
      Some(false)
    }
  });
  let online = expect_context::<RwSignal<OnlineSetter>>();

  let scroll_element = expect_context::<RwSignal<Option<NodeRef<Div>>>>();
  scroll_element.set(None);

  let ssr_sort =
    move || serde_json::from_str::<CommentSortType>(&query.get().get("sort").cloned().unwrap_or("".into())).unwrap_or(CommentSortType::Top);

  let reply_show = RwSignal::new(false);
  // let refresh_comments = RwSignal::new(false);
  let content = RwSignal::new(String::default());
  let loading = RwSignal::new(true);
  // let refresh = RwSignal::new(false);

  let post_view = RwSignal::new(None::<GetPostResponse>);

  let post_resource = Resource::new(
    move || (post_id.get()),
    move |(id_string)| async move {
      if let Some(id) = id_string {
        let form = GetPost {
          id: Some(PostId(id)),
          comment_id: None,
        };
        let result = LemmyClient.get_post(form.clone()).await;
        loading.set(false);
        match result {
          Ok(o) => Some(Ok((form, o))),
          Err(e) => {
            // error.update(|es| es.push(Some((e.clone(), None))));
            Some(Err(e))
          }
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
          Err(e) => {
            // error.update(|es| es.push(Some((e, None))));
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
          // error.update(|es| es.push(Some((e.into(), None))));
        }
      }
      if CommentSortType::Top == s {
        query_params.remove("sort".into());
      }
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
        if let Some(id) = post_id.get() {
          let form = CreateComment {
            content: content.get(),
            post_id: PostId(id),
            parent_id: None,
            language_id: None,
          };
          let result = LemmyClient.reply_comment(form).await;
          match result {
            Ok(_o) => {
              comments_resource.refetch();
              // refresh_comments.update(|b| *b = !*b);
              reply_show.update(|b| *b = !*b);
              #[cfg(not(feature = "ssr"))]
              if let Ok(d) = build_indexed_database().await {
                if let Ok(c) = del_draft(&d, id, Draft::Post).await {}
              }
            }
            Err(e) => {
              // error.update(|es| es.push(Some((e, None))));
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

  let (get_instance_cookie, set_instance_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
    "instance",
    UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax),
  );

  view! {
    <main class="flex flex-col">
      <ResponsiveTopNav ssr_site default_sort={SortType::TopAll.into()} post_view={post_view.into()} />
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
                    Some(
                      view! {
                        <Title text="Error loading post" />
                        <div class="py-4 px-8">
                          <div class="flex justify-between alert alert-warning">
                            <span>"Offline"</span>
                            <div>
                              // <Show when={move || { if let Some(_) = err.1 { true } else { false } }} fallback={|| {}}>
                              <button
                                on:click={move |_| {
                                  post_resource.refetch();
                                  comments_resource.refetch();
                                }}
                                class="btn btn-sm"
                              >
                                "Retry"
                              </button>
                            // </Show>
                            </div>
                          </div>
                        </div>
                        <div class="hidden" />
                      },
                    )
                  }
                  Some(Some(Err(_))) => {
                    Some(
                      view! {
                        <Title text="Error loading post" />
                        <div class="py-4 px-8">
                          <div class="flex justify-between alert alert-error">
                            <span>"Error"</span>
                            <div>
                              // <Show when={move || { if let Some(_) = err.1 { true } else { false } }} fallback={|| {}}>
                              <button
                                on:click={move |_| {
                                  post_resource.refetch();
                                  comments_resource.refetch();
                                }}
                                class="btn btn-sm"
                              >
                                "Retry"
                              </button>
                            // </Show>
                            </div>
                          </div>
                        </div>
                        <div class="hidden" />
                      },
                    )
                  }
                  Some(Some(Ok(res))) => {
                    #[cfg(not(feature = "ssr"))]
                    {
                      let rw = res.1.clone();
                      let fm = res.0.clone();
                      use crate::indexed_db::csr_indexed_db::*;
                      #[cfg(not(feature = "ssr"))]
                      spawn_local(async move {
                        if let Ok(d) = build_indexed_database().await {
                          if let Ok(c) = set_query_get_cache(&d, &fm, &rw).await {}
                        }
                      });
                    }
                    let res = res.1.clone();
                    post_view.set(Some(res.clone()));
                    let text = if let Some(b) = res.post_view.post.body.clone() {
                      if b.len() > 0 { Some(b) } else { res.post_view.post.embed_description.clone() }
                    } else {
                      None
                    };
                    let title = post_view.get().unwrap().post_view.post.name.clone();
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
                    let community_title = if post_view.get().unwrap().post_view.community.local {
                      format!("{}", post_view.get().unwrap().post_view.community.name)
                    } else {
                      format!(
                        "{}@{}",
                        post_view.get().unwrap().post_view.community.name,
                        post_view.get().unwrap().post_view.community.actor_id.inner().host().unwrap().to_string(),
                      )
                    };
                    let community_title_encoded = html_escape::encode_safe(&community_title).to_string();
                    let creator_name = &post_view.get().unwrap().post_view.creator.actor_id.to_string()[8..];
                    let creator_name_encoded = html_escape::encode_safe(creator_name).to_string();
                    let now_in_millis = {
                      #[cfg(not(feature = "ssr"))] { chrono::offset::Utc::now().timestamp_millis() as u64 }
                      #[cfg(feature = "ssr")] { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64 }
                    };
                    let duration_in_text = pretty_duration::pretty_duration(
                      &std::time::Duration::from_millis(now_in_millis - post_view.get().unwrap().post_view.post.published.timestamp_millis() as u64),
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
                    Some(
                      // log!("post");

                      // let title_encoded = html_escape::encode_safe(&title).to_string();

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

                        <div>
                          <ResponsivePostToolbar post_view={res.post_view.into()} ssr_site post_number=0 reply_show content post_id />
                        </div>
                        <div class="py-2 px-4">
                          // <A href={move || format!("/responsive/p/{}", post_view.get().unwrap().post_view.post.id)} class="pb-1 block hover:text-accent">
                          <span class="overflow-y-auto text-xl break-words" inner_html={title_encoded} />
                          // </A>
                          <span class="block mb-1 break-words text-md">
                            <span>{abbr_duration}</span>
                            " ago by "
                            <a
                              href={move || format!("{}", post_view.get().unwrap().post_view.creator.actor_id)}
                              target="_blank"
                              class="inline break-words hover:text-secondary"
                            >
                              <span class="overflow-y-auto" inner_html={creator_name_encoded} />
                            </a>
                            " in "
                            <A
                              class="inline break-words hover:text-secondary"
                              href={if post_view.get().unwrap().post_view.community.local {
                                format!("/r/c/{}", post_view.get().unwrap().post_view.community.name)
                              } else {
                                format!(
                                  "/r/c/{}@{}",
                                  post_view.get().unwrap().post_view.community.name,
                                  post_view.get().unwrap().post_view.community.actor_id.inner().host().unwrap().to_string(),
                                )
                              }}
                              on:click={move |e: MouseEvent| {
                                if let Ok(Some(s)) = window().local_storage() {
                                  let mut query_params = query.get();
                                  let _ = s.set_item(&format!("/r/c/{}", post_view.get().unwrap().post_view.community.name), "0");
                                }
                              }}
                            >
                              <span class="overflow-y-auto" inner_html={community_title_encoded} />
                            </A>
                            <span
                              class="overflow-y-auto"
                              inner_html={if let Some(d) = post_view.get().unwrap().post_view.post.url {
                                if let Some(f) = d.inner().host_str() {
                                  if f.to_string().ne(&get_instance_cookie.get().unwrap_or("".into())) { format!(" from {}", f) } else { "".into() }
                                } else {
                                  "".into()
                                }
                              } else {
                                "".into()
                              }}
                            />
                          // <span class="overflow-y-auto" inner_html={{ if post_view.get().unwrap().post_view.post.local { "".to_string() } else { if let Some(d) = post_view.get().unwrap().post_view.post.url { format!(" from {}", d.inner().host_str().unwrap_or("")) } else { "".to_string() } } }} />
                          </span>
                        </div>
                        <a
                          class={move || {
                            format!(
                              "float-left{}",
                              if post_view.get().unwrap().post_view.post.thumbnail_url.is_none()
                                && post_view.get().unwrap().post_view.post.url.is_none()
                              {
                                " hidden"
                              } else {
                                ""
                              },
                            )
                          }}
                          target="_blank"
                          href={move || {
                            if let Some(d) = post_view.get().unwrap().post_view.post.url {
                              d.inner().to_string()
                            } else {
                              format!("/r/post/{}", post_view.get().unwrap().post_view.post.id)
                            }
                          }}
                        >
                          {move || {
                            if let Some(t) = post_view.get().unwrap().post_view.post.thumbnail_url {
                              let h = t.inner().to_string();
                              thumbnail.set(h);
                              view! {
                                <div class="py-2 px-4">
                                  <div class="block">
                                    <img
                                      loading="lazy"
                                      class={move || {
                                        format!("w-auto max-w-9/12{}", if thumbnail.get().eq(&"/lemmy.svg".to_string()) { " h-16" } else { "" })
                                      }}
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
                                <div class="py-2 px-4">
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
                            <label class="form-control">
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
                                    create_local_resource(
                                      move || (),
                                      move |()| async move {
                                        if let Ok(d) = build_indexed_database().await {
                                          if let Ok(c) = set_draft(&d, id, Draft::Post, content.get()).await {}
                                        }
                                      },
                                    );
                                  }
                                }}
                              >
                                {content.get_untracked()}
                              </textarea>
                            </label>
                            <button
                              on:click={on_reply_click}
                              type="button"
                              class={move || {
                                format!(
                                  "btn btn-neutral{}",
                                  {
                                    if Some(true) != logged_in.get() || !online.get().0 {
                                      " text-base-content/50"
                                    } else {
                                      " hover:text-secondary/50"
                                    }
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
                comments_resource
                  .get()
                  .unwrap_or(None)
                  .map(|res| {
                    #[cfg(not(feature = "ssr"))]
                    {
                      let rw = res.1.clone();
                      let fm = res.0.clone();
                      use crate::indexed_db::csr_indexed_db::*;
                      #[cfg(not(feature = "ssr"))]
                      spawn_local(async move {
                        if let Ok(d) = build_indexed_database().await {
                          if let Ok(c) = set_query_get_cache(&d, &fm, &rw).await {}
                        }
                      });
                    }
                    let res = res.1.clone();
                    // log!("comments");

                    view! {
                      <div class="w-full before:content-[''] before:block before:w-24 before:overflow-hidden">
                        <ResponsiveCommentNodes ssr_site comments={res.comments.into()} post_id />
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
