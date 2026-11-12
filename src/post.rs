use crate::prelude::*;
use ev::MouseEvent;
#[cfg(not(feature = "ssr"))]
use leptos_use_intersection_observer::{UseIntersectionObserverOptions, use_intersection_observer_with_options};
#[cfg(not(feature = "ssr"))]
use leptos_use_scroll::{UseScrollOptions, UseScrollReturn, use_scroll_with_options};
// use leptos_use::*;

#[component]
pub fn Post() -> impl IntoView {
  let ssr_site = expect_context::<Resource<Result<GetSiteResponse, LemmyAppError>>>();
  let params = use_params_map();
  let query = use_query_map();

  let post_id = Signal::derive(move || params.get().get("id").unwrap_or_default().parse::<i32>().ok());
  let online = expect_context::<RwSignal<OnlineSetter>>();
  let ssr_sort = move || serde_json::from_str::<CommentSortType>(&query.get().get("sort").unwrap_or("".into())).unwrap_or(CommentSortType::Top);

  let reply_show = RwSignal::new(false);
  let content = RwSignal::new(String::default());
  let loading = RwSignal::new(false);
  let error = RwSignal::new(None::<LemmyAppError>);

  let post_view = RwSignal::new(None::<GetPostResponse>);

  let ReadAuthCookie(get_auth_cookie) = expect_context::<ReadAuthCookie>();
  let ReadInstanceCookie(get_instance_cookie) = expect_context::<ReadInstanceCookie>();
  let response_cache = expect_context::<RwSignal<BTreeMap<(usize, GetPosts, Option<String>, bool), (i64, LemmyAppResult<GetPostsResponse>)>>>();

  let post_resource = Resource::new(
    move || (post_id.get(), get_auth_cookie.get(), get_instance_cookie.get()),
    move |(id_string, auth, instance)| async move {
      if let Some(id) = id_string {
        let form = GetPost { id: Some(PostId(id)), comment_id: None };
        #[cfg(not(feature = "ssr"))]
        loading.set(true);
        let result = LemmyClient.get_post(form.clone()).await;
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
    move || (post_id.get(), ssr_sort(), get_auth_cookie.get(), get_instance_cookie.get()),
    move |(post_id, sort_type, auth, instance)| async move {
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
        Err(_e) => {}
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
    spawn_local_scoped(async move {
      if let Some(id) = post_id.get() {
        loading.set(true);
        error.set(None);
        let form = CreateComment { content: content.get(), post_id: PostId(id), parent_id: None, language_id: None };
        let result = LemmyClient.reply_comment(form).await;
        match result {
          Ok(_o) => {
            comments_resource.refetch();
            reply_show.update(|b| *b = !*b);
            if let Ok(d) = IndexedDb::new().await {
              if let Ok(_c) = d.del(&CommentDraftKey { comment_id: id, draft: Draft::Post }).await {}
            }
          }
          Err(e) => {
            error.set(Some(e));
          }
        }
        loading.set(false);
      }
    });
  };

  let _visibility_element = NodeRef::<Textarea>::new();

  let on_scroll_element = NodeRef::<Div>::new();
  let on_scroll = move |_e: Event| {
    #[cfg(not(feature = "ssr"))]
    if let Some(se) = on_scroll_element.get() {
      spawn_local_scoped(async move {
        let _ = IndexedDb::save_scroll_position(use_location().pathname.get(), use_query_map().get().to_query_string(), se.scroll_left()).await;
      });
    }
  };

  #[cfg(not(feature = "ssr"))]
  {
    // use leptos_use_intersection_observer::{UseIntersectionObserverOptions, UseIntersectionObserverReturn, use_intersection_observer_with_options};
    // use leptos_use_scroll::{UseScrollOptions, UseScrollReturn, use_scroll_with_options};

    let UseScrollReturn { .. } = use_scroll_with_options(on_scroll_element, UseScrollOptions::default().on_scroll(on_scroll));

    let _ = use_intersection_observer_with_options(
      _visibility_element,
      move |_entries, _io| {
        if let Some(v) = _visibility_element.get() {
          let _ = v.focus();
        }
      },
      UseIntersectionObserverOptions::default(),
    );
  }

  #[cfg(not(feature = "ssr"))]
  let cancel_handle: RwSignal<Option<TimeoutHandle>> = RwSignal::new(None);

  let thumbnail = RwSignal::new(String::from(""));
  let ReadInstanceCookie(get_instance_cookie) = expect_context::<ReadInstanceCookie>();

  view! {
    <main class="flex flex-col">
      <TopNav scroll_element={on_scroll_element.into()} default_sort={SortType::TopAll.into()} post_view />
      <div class="flex flex-grow">
        <div
          on:wheel={move |e: WheelEvent| handle_horizontal_scroll_wheel(e, on_scroll_element)}
          node_ref={on_scroll_element}
          class="gap-4 min-w-full sm:overflow-x-auto sm:overflow-y-hidden sm:absolute sm:px-4 sm:h-[calc(100%-4rem)] sm:columns-[23rem]"
          style="column-fill: auto"
        >
          <div>
            <Transition fallback={|| {}}>
              {move || {
                match post_resource.get() {
                  Some(Some(Err(LemmyAppError { error_type: LemmyAppErrorType::OfflineError, .. }))) => {
                    #[cfg(not(feature = "ssr"))] loading.set(false);
                    view! {
                      <Title text="Error loading post" />
                      <Warning on_retry_click={move |_| {
                        post_resource.refetch();
                        comments_resource.refetch();
                      }} />
                    }
                      .into_any()
                  }
                  Some(Some(Err(e))) => {
                    #[cfg(not(feature = "ssr"))] loading.set(false);
                    view! {
                      <Title text="Error loading post" />
                      <Error
                        description="Error loading post"
                        error={e}
                        on_retry_click={move |_| {
                          post_resource.refetch();
                          comments_resource.refetch();
                        }}
                      />
                    }
                      .into_any()
                  }
                  Some(Some(Ok(res))) => {
                    #[cfg(not(feature = "ssr"))]
                    {
                      let rw = res.1.clone();
                      let fm = res.0.clone();
                      let post_view = RwSignal::new(rw.post_view.clone());
                      spawn_local_scoped(async move {
                        if let Ok(d) = IndexedDb::new().await {
                          if let Ok(_c) = d.set(&fm, &rw).await {}
                        }
                        response_cache
                          .update(|rc| {
                            rc.iter_mut()
                              .for_each(|(f, r)| {
                                if let (_, Ok(w)) = r {
                                  let id = post_view.get().post.id.0;
                                  for p in w.posts.iter_mut() {
                                    if p.post.id.0 == id {
                                      *p = post_view.get();
                                      let result_clone = r.1.clone();
                                      let form_clone = f.1.clone();
                                      spawn_local_scoped(async move {
                                        if let Ok(d) = IndexedDb::new().await {
                                          if let Ok(_c) = d.set::<GetPosts, Result<GetPostsResponse, LemmyAppError>>(&form_clone, &result_clone).await
                                          {}
                                        }
                                      });
                                      break;
                                    }
                                  }
                                }
                              });
                          });
                      });
                    }
                    post_view.set(Some(res.1.clone()));
                    let post_response = Memo::new(move |_| res.1.clone());
                    let text = if let Some(b) = post_response.get().post_view.post.body.clone() {
                      if b.len() > 0 { Some(b) } else { post_response.get().post_view.post.embed_description.clone() }
                    } else {
                      None
                    };
                    let title = post_response.get().post_view.post.name.clone();
                    let title_encoded = markdown_to_html(&title, true);
                    let community_title = community_title_with_federation(
                      &post_response.get().post_view.community.name,
                      post_response.get().post_view.community.local,
                      post_response.get().post_view.community.actor_id.inner().host().map(|host| host.to_string()),
                    );
                    let community_title_encoded = html_escape::encode_safe(&community_title).to_string();
                    // let creator_name = &post_response.get().post_view.creator.actor_id.to_string()[8..];
                    let a = post_response.get().post_view.creator.actor_id;
                    let u = post_response.get().post_view.creator.name;
                    let instance = get_instance_cookie.get_untracked();
                    let creator_name = federated_creator_handle(&u, a.domain().as_deref(), instance.as_deref());
                    let creator_name_encoded = html_escape::encode_safe(&creator_name).to_string();
                    let abbr_duration = relative_time(post_response.get().post_view.post.published.timestamp_millis());
                    let url = Memo::new(move |_| post_response.get().post_view.post.url);
                    let thumbnail_url = Memo::new(move |_| post_response.get().post_view.post.thumbnail_url);
                    let domain = Memo::new(move |_| {
                      external_host_attribution(
                        url.get().and_then(|url| url.inner().host_str().map(str::to_owned)),
                        get_instance_cookie.get(),
                      )
                    });
                    #[cfg(not(feature = "ssr"))] loading.set(false);
                    let languages = Memo::new(move |_| if let Some(Ok(GetSiteResponse { all_languages: l, .. })) = ssr_site.get() { l } else { vec![] });

                    view! {
                      <Title text={post_response.get().post_view.post.name} />
                      <div>
                        <PostToolbar post_view={post_response.get().post_view.into()} reply_show content post_id />
                      </div>
                      <div class="py-2 px-4">
                        <span
                          on:click={move |e: MouseEvent| {
                            if let Some(t) = e.target() {
                              if let Some(l) = t.dyn_ref::<HtmlAnchorElement>() {
                                e.prevent_default();
                                if l.host().eq(&window().location().host().unwrap_or("".to_owned())) {
                                  use_navigate()(&l.href(), Default::default());
                                } else {
                                  let _ = window().open_with_url_and_target(&l.href(), "_blank");
                                }
                              }
                            }
                          }}
                          lang={move || languages.get().get(post_response.get().post_view.post.language_id.0.abs() as usize).map(|l| if l.id.0 == 0 { "".into() } else { l.code.clone() } )}
                          class="overflow-y-auto text-2xl font-extrabold wrap-anywhere"
                          inner_html={title_encoded}
                        />
                        <span class="block mb-1 wrap-anywhere text-md">
                          <span>{abbr_duration}</span>
                          " by "
                          <A
                            href={move || {
                              // let a = post_response.get().post_view.creator.actor_id;
                              // let u = post_response.get().post_view.creator.name;
                              // if let Some(domain) = a.domain() { format!("/u/{}@{}?person_id={}", u, domain, post_response.get().post_view.creator.id.0) } else { format!("{}?person_id={}", a, post_response.get().post_view.creator.id.0) }
                              format!("/u/{}?person_id={}", creator_name, post_response.get().post_view.creator.id.0)
                            }}
                            attr:class="inline wrap-anywhere hover:text-secondary"
                          >
                            <span class="overflow-y-auto" inner_html={creator_name_encoded} />
                          </A>
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
                              e.prevent_default();
                              spawn_local_scoped(async move {
                                let path = if post_response.get().post_view.community.local {
                                  format!("/c/{}", post_response.get().post_view.community.name)
                                } else {
                                  format!(
                                    "/c/{}@{}",
                                    post_response.get().post_view.community.name,
                                    post_response.get().post_view.community.actor_id.inner().host().unwrap().to_string(),
                                  )
                                };
                                let _ = IndexedDb::save_scroll_position(path, "".into(), 0).await;
                                use_navigate()(
                                  &{
                                    if post_response.get().post_view.community.local {
                                      format!("/c/{}", post_response.get().post_view.community.name)
                                    } else {
                                      format!(
                                        "/c/{}@{}",
                                        post_response.get().post_view.community.name,
                                        post_response.get().post_view.community.actor_id.inner().host().unwrap().to_string(),
                                      )
                                    }
                                  },
                                  Default::default(),
                                );
                              });
                            }}
                          >
                            // {
                            //   let languages = Memo::new(move |_| if let Some(Ok(GetSiteResponse { all_languages: l, .. })) = ssr_site.get() { l } else { vec![] });
                            //   view! {
                            <span lang={move || languages.get().get(post_response.get().post_view.post.language_id.0.abs() as usize).map(|l| if l.id.0 == 0 { "".into() } else { l.code.clone() } )} class="overflow-y-auto" inner_html={community_title_encoded} />
                            //   }
                            // }
                          </A>
                          <span class="overflow-y-auto">{domain.get()}</span>
                        </span>
                      </div>
                      <a
                        class={move || { format!("float-left{}", if thumbnail_url.get().is_none() && url.get().is_none() { " hidden" } else { "" }) }}
                        target="_blank"
                        href={move || {
                          if let Some(d) = url.get() { d.inner().to_string() } else { format!("/p/{}", post_response.get().post_view.post.id) }
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
                        let safe_html = markdown_to_html(content, true);
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
                                <div lang={move || languages.get().get(post_response.get().post_view.post.language_id.0.abs() as usize).map(|l| if l.id.0 == 0 { "".into() } else { l.code.clone() } )} class="max-w-none prose overflow-clip" inner_html={safe_html} />
                              </div>
                            </div>
                          },
                        )
                      } else {
                        None
                      }}
                      <Transition fallback={|| {}}>
                        {move || {
                          match ssr_site.get() {
                            Some(Ok(s)) => {
                              {
                                let logged_in = Memo::new(move |_| { s.my_user.is_some() });
                                view! {
                                  <Show when={move || reply_show.get()} fallback={|| {}}>
                                    <div class="pr-4 pl-4 mb-3 space-y-3 before:content-[''] before:block before:w-24 before:overflow-hidden">
                                      <div class="form-control">
                                        <textarea
                                          class="w-full h-24 text-base textarea textarea-bordered"
                                          placeholder="Comment text"
                                          prop:value={move || content.get()}
                                          node_ref={_visibility_element}
                                          on:wheel={move |e: WheelEvent| {
                                            e.stop_propagation();
                                          }}
                                          on:input={move |ev| {
                                            content.set(event_target_value(&ev));
                                            if let Some(id) = post_id.get() {
                                              spawn_local_scoped(async move {
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
                                              "btn btn-soft{}",
                                              {
                                                if !logged_in.get() || !online.get().0 { " text-base-content/50" } else { " hover:text-secondary/50" }
                                              },
                                            )
                                          }}
                                          disabled={move || !logged_in.get() || !online.get().0}
                                        >
                                          "Comment"
                                        </button>
                                        <button on:click={move |_| reply_show.set(false)} type="button" class="btn btn-soft">
                                          "Cancel"
                                        </button>
                                      </div>
                                    </div>
                                  </Show>
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
                              description="Reply"
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
                      .into_any()
                  }
                  Some(None) | None => view! {}.into_any(),
                }
              }}
              {move || {
                comments_resource
                  .get()
                  .unwrap_or(None)
                  .map(|res| {
                    #[cfg(not(feature = "ssr"))]
                    {
                      let rw = res.1.clone();
                      let fm = res.0.clone();
                      spawn_local_scoped(async move {
                        if let Ok(d) = IndexedDb::new().await {
                          if let Ok(_c) = d.set(&fm, &rw).await {}
                        }
                      });
                      let iw = window().inner_width().ok().map(|b| b.as_f64().unwrap_or(0.0)).unwrap_or(0.0);
                      if iw > 768f64 {
                        if let Some(c) = cancel_handle.get_untracked() {
                          c.clear();
                        }
                        cancel_handle
                          .set(
                            set_timeout_with_handle(
                                move || {
                                  if let Some(s) = on_scroll_element.get() {
                                    spawn_local_scoped(async move {
                                      if let Ok(Some(l)) =
                                        IndexedDb::get_scroll_position(use_location().pathname.get(), use_query_map().get().to_query_string()).await
                                      {
                                        s.set_scroll_left(l);
                                      }
                                    });
                                  }
                                },
                                std::time::Duration::new(0, 750_000_000),
                              )
                              .ok(),
                          );
                      }
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
            {move || {
              view! { <Info loading={loading.get()} /> }
            }}
          </div>
        </div>
      </div>
    </main>
  }
}
