use crate::prelude::*;
use ev::MouseEvent;
#[cfg(not(feature = "ssr"))]
use leptos_use_intersection_observer::{UseIntersectionObserverOptions, use_intersection_observer_with_options};

#[component]
pub fn Hero(post_id: PostId, hide: bool, #[prop(optional)] next_page_cursor: RwSignal<(usize, Option<PaginationCursor>)>) -> impl IntoView {
  let i18n = use_i18n();
  let endpoint = use_internal_endpoint();
  let ssr_sort = move || endpoint.get().post_query().and_then(|query| query.sort).unwrap_or(CommentSortType::Top);

  let reply_show = RwSignal::new(None);
  let content = RwSignal::new(String::default());
  let loading = RwSignal::new(true);

  let ssr_site = expect_context::<Resource<Result<GetSiteResponse, LemmyAppError>>>();
  let listing_response_cache = expect_context::<RwSignal<BTreeMap<(usize, GetPostsCacheKey), (i64, LemmyAppResult<GetPostsResponse>)>>>();
  let post_response_cache = expect_context::<RwSignal<BTreeMap<GetCacheKey<GetPost>, (i64, LemmyAppResult<GetPostResponse>)>>>();
  let comments_response_cache = expect_context::<RwSignal<BTreeMap<GetCacheKey<GetComments>, (i64, LemmyAppResult<GetCommentsResponse>)>>>();
  let ReadAuthCookie(get_auth_cookie) = expect_context::<ReadAuthCookie>();
  let ReadInstanceCookie(get_instance_cookie) = expect_context::<ReadInstanceCookie>();

  let post_resource = Resource::new(
    move || (post_id, get_auth_cookie.get(), get_instance_cookie.get()),
    move |(id, auth, instance)| async move {
      let form = GetPost { id: Some(id), comment_id: None };
      let rc = post_response_cache.get_untracked();
      let key = GetCacheKey { form: form.clone(), auth, instance, ban: false };
      let result = if let Some((_t, r)) = rc.get(&key) {
        match r {
          Ok(o) => Ok(o.clone()),
          _ => LemmyClient.get_post(key.clone()).await,
        }
      } else {
        LemmyClient.get_post(key.clone()).await
      };
      loading.set(false);
      match result {
        Ok(o) => Some(Ok((form, o))),
        Err(e) => Some(Err(e)),
      }
    },
  );

  let comments_resource = Resource::new(
    move || (post_id, ssr_sort(), get_auth_cookie.get(), get_instance_cookie.get()),
    move |(post_id, sort_type, auth, instance)| async move {
      let form = GetComments {
        post_id: Some(post_id),
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
      let rc = comments_response_cache.get_untracked();
      let key = GetCacheKey { form: form.clone(), auth, instance, ban: false };
      let result = if let Some((_t, r)) = rc.get(&key) {
        match r {
          Ok(o) => Ok(o.clone()),
          _ => LemmyClient.get_comments(key.clone()).await,
        }
      } else {
        LemmyClient.get_comments(key.clone()).await
      };
      match result {
        Ok(o) => Some((form, o)),
        Err(_e) => None,
      }
    },
  );

  let _visibility_element = NodeRef::<Textarea>::new();

  #[cfg(not(feature = "ssr"))]
  {
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

  let _on_scroll_element = NodeRef::<Div>::new();
  let thumbnail = RwSignal::new(String::from(""));
  view! {
    <div class={format!("bg-base-300 pb-1 mb-6{}", if hide { " invisible" } else { "" })}>
      <Transition fallback={|| {}}>
        {move || {
          match post_resource.get() {
            Some(Some(Err(LemmyAppError { error_type: LemmyAppErrorType::OfflineError, .. }))) => {
              view! {
                <Title text={move || t_string!(i18n, offline)} />
                <Warning />
              }
                .into_any()
            }
            Some(Some(Err(e))) => {
              view! {
                <Title text={move || t_string!(i18n, error_loading_post)} />
                <Error
                  description={t_string!(i18n, error_loading_post).to_string()}
                  error={e}
                  on_retry_click={move |_| {
                    post_resource.refetch();
                    comments_resource.refetch();
                  }}
                />
              }
                .into_any()
            }
            Some(Some(Ok((form, response)))) => {
              #[cfg(not(feature = "ssr"))]
              {
                let response_clone = response.clone();
                let form_clone = form.clone();
                let cache_key = GetCacheKey {
                  form: form_clone.clone(),
                  auth: get_auth_cookie.get_untracked(),
                  instance: get_instance_cookie.get_untracked(),
                  ban: false,
                };
                spawn_local_scoped(async move {
                  post_response_cache
                    .update(move |rc| {
                      rc.insert(cache_key, (0, Ok(response_clone)));
                    });
                });
              }
              let post_response = RwSignal::new(response.clone());
              let text = if let Some(b) = post_response.get().post_view.post.body.clone() {
                if b.len() > 0 { Some(b) } else { post_response.get().post_view.post.embed_description.clone() }
              } else {
                None
              };
              let title = post_response.get().post_view.post.name.clone();
              let title_encoded = markdown_to_html(&title, false);
              let community_title = community_title_with_federation(
                &post_response.get().post_view.community.name,
                post_response.get().post_view.community.local,
                post_response.get().post_view.community.actor_id.inner().host().map(|host| host.to_string()),
              );
              let community_title_encoded = html_escape::encode_safe(&community_title).to_string();
              let a = post_response.get().post_view.creator.actor_id;
              let u = post_response.get().post_view.creator.name;
              let instance = get_instance_cookie.get_untracked();
              let creator_name = federated_creator_handle(&u, a.domain().as_deref(), instance.as_deref());
              let creator_name_encoded = html_escape::encode_safe(&creator_name).to_string();
              let abbr_duration = relative_time(post_response.get().post_view.post.published.timestamp_millis());
              let url = Memo::new(move |_| post_response.get().post_view.post.url);
              let thumbnail_url = Memo::new(move |_| post_response.get().post_view.post.thumbnail_url);
              let display_url = Memo::new(move |_| {
                url
                  .get()
                  .map(|url| {
                    external_host_attribution(url.inner().host_str().map(str::to_owned), get_instance_cookie.get())
                  })
                  .filter(|attribution| !attribution.is_empty())
              });
              let languages = Memo::new(move |_| if let Some(Ok(GetSiteResponse { all_languages: l, .. })) = ssr_site.get() { l } else { vec![] });

              view! {
                <div class="break-inside-avoid">
                  <div class="pt-2 px-4">
                    <A
                      href={move || {
                        internal_navigation_url(&InternalNavigation::new(
                          InternalEndpointData::Post {
                            id: post_response.get().post_view.post.id.0.to_string(),
                            query: PostQuery::default(),
                          },
                        ))
                      }}
                      attr:class="block hover:text-accent"
                    >
                      <span lang={move || languages.get().get(post_response.get().post_view.post.language_id.0.abs() as usize).map(|l| if l.id.0 == 0 { "".into() } else { l.code.clone() } )} class="overflow-y-auto text-4xl font-extrabold wrap-anywhere" inner_html={title_encoded} />
                    </A>
                  </div>
                  <a
                    class={move || { format!("{}", if thumbnail_url.get().is_none() && url.get().is_none() { " hidden" } else { "" }) }}
                    target="_blank"
                    href={move || {
                      if let Some(d) = url.get() {
                        d.inner().to_string()
                      } else {
                        internal_navigation_url(&InternalNavigation::new(
                          InternalEndpointData::Post {
                            id: post_response.get().post_view.post.id.0.to_string(),
                            query: PostQuery::default(),
                          },
                        ))
                      }
                    }}
                  >
                    {move || {
                      if let Some(t) = thumbnail_url.get() {
                        let h = t.inner().to_string();
                        thumbnail.set(h);
                        view! {
                          <div class="my-3 px-4">
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
                          <div class="my-3 px-4">
                            <div class="block">
                              <img class="h-16" src="/lemmy.svg" />
                            </div>
                          </div>
                        }
                          .into_any()
                      }
                    }}
                  </a>
                  <div class="my-3 px-4">
                    <PostToolbar
                      post_view={post_response.get().post_view}
                      reply_show
                      content
                      post_id={Signal::derive(move || Some(post_response.get().post_view.post.id.0))}
                    />
                  </div>
                  <div class="mt-3 mb-1 px-4">
                    <span class="block wrap-anywhere text-md">
                      <span>{abbr_duration}</span>
                      {t!(i18n, post_by)}
                      <A
                        href={move || {
                          internal_navigation_url(&InternalNavigation::new(
                            InternalEndpointData::User {
                              name: creator_name.clone(),
                              query: UserQuery { page: None, person_id: Some(post_response.get().post_view.creator.id.0), ..UserQuery::default() },
                            },
                          ))
                        }}
                        attr:class="inline wrap-anywhere hover:text-secondary"
                      >
                        <span class="overflow-y-auto" inner_html={creator_name_encoded} />
                      </A>
                      {t!(i18n, post_in)}
                      <a
                        class="inline wrap-anywhere hover:text-secondary"
                        href={internal_navigation_url(&InternalNavigation::new(
                          InternalEndpointData::Community {
                            name: if post_response.get().post_view.community.local {
                              post_response.get().post_view.community.name.clone()
                            } else {
                              format!(
                                "{}@{}",
                                post_response.get().post_view.community.name,
                                post_response.get().post_view.community.actor_id.inner().host().unwrap().to_string(),
                              )
                            },
                            query: ListingQuery::default(),
                          },
                        ))}
                        on:click={move |e: MouseEvent| {
                          e.prevent_default();
                          let community_name = if post_response.get().post_view.community.local {
                            post_response.get().post_view.community.name.clone()
                          } else {
                            format!(
                              "{}@{}",
                              post_response.get().post_view.community.name,
                              post_response.get().post_view.community.actor_id.inner().host().unwrap().to_string(),
                            )
                          };
                          let endpoint = InternalEndpointData::Community { name: community_name, query: ListingQuery::default() };
                          let navigation = InternalNavigation::new(endpoint.clone()).with_cache_entry(
                            internal_posts_cache_entry(
                              &endpoint,
                              ListingType::All,
                              SortType::Active,
                              &[],
                              get_auth_cookie.get_untracked(),
                              get_instance_cookie.get_untracked(),
                            )
                            .expect("community navigation has a posts cache key"),
                          );
                          next_page_cursor.set((0, None));
                          navigate_internal(navigation, None);
                        }}
                      >
                        <span class="overflow-y-auto" inner_html={community_title_encoded} />
                      </a>
                      <span class="overflow-y-auto" inner_html={move || display_url.get()} />
                    </span>
                  </div>
                </div>
                {if let Some(ref content) = text {
                  let safe_html = markdown_to_html(content, true);
                  Some(
                    view! {
                      <div class="mt-3 mb-1 px-4">
                        <div
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
              }
                .into_any()
            }
            Some(None) | None => view! { <Info loading={loading.get()} /> }.into_any(),
          }
        }}
      </Transition>
      <Transition fallback={|| {}}>
        {move || {
          comments_resource
            .get()
            .unwrap_or(None)
            .map(|(form, response)| {
              #[cfg(not(feature = "ssr"))]
              {
                let response_clone = response.clone();
                let form_clone = form.clone();
                let cache_key = GetCacheKey {
                  form: form_clone.clone(),
                  auth: get_auth_cookie.get_untracked(),
                  instance: get_instance_cookie.get_untracked(),
                  ban: false,
                };
                spawn_local_scoped(async move {
                  comments_response_cache
                    .update(move |rc| {
                      rc.insert(cache_key, (0, Ok(response_clone)));
                    });
                });
              }
              let mut comments_descendants = response.comments.clone();
              let first_comment = response.comments.first().map(|f| vec![f.clone()]).unwrap_or_default();
              let highlight_user_id = RwSignal::new(None);
              let now_in_millis = RwSignal::new(u64::try_from(jiff::Timestamp::now().as_millisecond()).unwrap_or(0));
              let hidden_comments: RwSignal<Vec<i32>> = RwSignal::new(vec![]);
              #[cfg(not(feature = "ssr"))]
              spawn_local_scoped(async move {
                let p = post_id;
                if let Ok(d) = IndexedDb::new().await {
                  if let Ok(Some(mut comment_ids)) = d.get::<i32, Vec<i32>>(&p.0).await {
                    hidden_comments
                      .update(|h| {
                        h.append(&mut comment_ids);
                      });
                  }
                }
              });
              hidden_comments
                .update(|h| {
                  if let Some(ref f) = first_comment.get(0) {
                    if !h.contains(&f.comment.id.0) {
                      h.push(f.comment.id.0);
                    }
                  }
                });
              if let Some(cv) = first_comment.get(0) {
                view! {
                  <div class="pl-4">
                    <Comment
                      parent_comment_id=0
                      hidden_comments
                      comment={cv.clone().into()}
                      comments={comments_descendants.into()}
                      level=1
                      now_in_millis
                      highlight_user_id
                      post_id={Signal::derive(move || Some(post_id.0))}
                      selected_drag_offset={RwSignal::new((1, 0f64, 0))}
                    />
                  </div>
                }.into_any()
              } else {
                view! {}.into_any()
              }
            })
        }}
      </Transition>
    </div>
  }
}
