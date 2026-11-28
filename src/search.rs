use crate::prelude::*;
#[cfg(not(feature = "ssr"))]
use leptos_use_element_visibility::use_element_visibility;
#[cfg(not(feature = "ssr"))]
use leptos_use_intersection_observer::{UseIntersectionObserverOptions, UseIntersectionObserverReturn, use_intersection_observer_with_options};
#[cfg(not(feature = "ssr"))]
use leptos_use_scroll::{UseScrollOptions, UseScrollReturn, use_scroll_with_options};

#[component]
pub fn PostSearch() -> impl IntoView {
  let _endpoint = InternalEndpoint::new(PostSearch, InternalEndpointRoute::PostSearch);
  view! { <Search /> }
}

#[component]
pub fn CommunitySearch() -> impl IntoView {
  let _endpoint = InternalEndpoint::new(CommunitySearch, InternalEndpointRoute::CommunitySearch);
  view! { <Search /> }
}

#[component]
pub fn UserSearch() -> impl IntoView {
  let _endpoint = InternalEndpoint::new(UserSearch, InternalEndpointRoute::UserSearch);
  view! { <Search /> }
}

#[component]
pub fn DefaultSearch() -> impl IntoView {
  let _endpoint = InternalEndpoint::new(DefaultSearch, InternalEndpointRoute::Search);
  view! { <Search /> }
}

#[component]
pub fn Search() -> impl IntoView {
  let i18n = use_i18n();
  let endpoint = use_internal_endpoint();
  let ssr_name = move || endpoint.get().community_name().map(str::to_owned);
  let ssr_kind = move || endpoint.get().search_query().and_then(|query| query.kind).unwrap_or(SearchType::Posts);
  let ssr_list = move || endpoint.get().search_query().and_then(|query| query.list);
  let ssr_sort = move || endpoint.get().search_query().and_then(|query| query.sort).unwrap_or(SortType::Active);
  let ssr_page = move || endpoint.get().search_query().and_then(|query| query.page.clone()).unwrap_or(vec![1usize]);
  let ssr_term = move || endpoint.get().search_query().and_then(|query| query.term.clone()).unwrap_or_default();
  let person_id = move || endpoint.get().search_query().and_then(|query| query.person_id);
  let next_page_cursor: RwSignal<(usize, usize)> = RwSignal::new((0, 1));

  let loading = RwSignal::new(false);

  let intersection_element = NodeRef::<Div>::new();
  let on_scroll_element = NodeRef::<Div>::new();

  #[cfg(not(feature = "ssr"))]
  {
    let on_scroll = move |_e: Event| {
      if ssr_page().len() > 1 {
        if let Some(se) = on_scroll_element.get() {
          #[cfg(not(feature = "ssr"))]
          spawn_local_scoped(async move {
            let current_endpoint = endpoint.get();
            let _ = IndexedDb::save_scroll_position(current_endpoint.path(), current_endpoint.query_string(), se.scroll_left()).await;
          });
        }
      }
    };

    let UseScrollReturn { .. } = use_scroll_with_options(on_scroll_element, UseScrollOptions::default().on_scroll(on_scroll));
    let UseIntersectionObserverReturn { .. } = use_intersection_observer_with_options(
      intersection_element,
      move |intersections, _| {
        if intersections[0].is_intersecting() {
          let (key, len) = next_page_cursor.get();
          if key > 1 && len > 0 {
            let mut st = ssr_page();
            if st.iter().find(|s| **s == key).is_none() {
              st.push(key);
            }
            let destination = endpoint.get().with_page(st);
            let params = destination.query_string();
            spawn_local_scoped(async move {
              if let Some(se) = on_scroll_element.get() {
                let _ = IndexedDb::save_scroll_position(destination.path(), params, se.scroll_left()).await;
              }
              navigate_internal_pagination(InternalNavigation::new(destination));
            });
          }
        }
      },
      UseIntersectionObserverOptions::default(),
    );
  }

  #[cfg(not(feature = "ssr"))]
  let is_visible = use_element_visibility(intersection_element);

  #[cfg(not(feature = "ssr"))]
  let cancel_handle: RwSignal<Option<TimeoutHandle>> = RwSignal::new(None);

  let search_cache = expect_context::<RwSignal<BTreeMap<(usize, GetCacheKey<Search>), (i64, LemmyAppResult<SearchResponse>)>>>();

  let ReadAuthCookie(get_auth_cookie) = expect_context::<ReadAuthCookie>();
  let ReadInstanceCookie(get_instance_cookie) = expect_context::<ReadInstanceCookie>();

  let search_cache_resource = Resource::new(
    move || (ssr_list(), ssr_sort(), ssr_name(), ssr_page(), ssr_term(), get_auth_cookie.get(), get_instance_cookie.get(), ssr_kind(), person_id()),
    move |(list, sort, name, pages, term, auth, instance, kind, person_id)| async move {
      let sc = search_cache.get_untracked();
      let mut new_pages: Vec<(usize, GetCacheKey<Search>, i64, LemmyAppResult<SearchResponse>)> = Vec::new();
      for p in pages {
        let form = Search {
          q: term.clone(),
          type_: Some(kind),
          sort: Some(sort),
          community_name: if person_id.is_some() { None } else { name.clone() },
          community_id: None,
          page: Some(p.try_into().unwrap_or(1i64)),
          limit: Some(50),
          creator_id: person_id.and_then(|i| Some(PersonId(i))),
          listing_type: list,
          post_title_only: None,
        };
        let cache_key = GetCacheKey { form, auth: auth.clone(), instance: instance.clone(), ban: false };
        #[cfg(not(feature = "ssr"))]
        loading.set(true);
        #[cfg(not(feature = "ssr"))]
        if let Some((t, Ok(r))) = sc.get(&(p, cache_key.clone())) {
          new_pages.push((p, cache_key.clone(), t.clone(), Ok(r.clone())));
          if p == 1 && r.posts.len() == 0 {
            loading.set(false);
          }
          continue;
        }
        let t = jiff::Timestamp::now().as_millisecond();
        let result = LemmyClient.search(cache_key.clone()).await;
        match result {
          Ok(o) => {
            if p == 1 && o.posts.len() == 0 {
              loading.set(false);
            }
            new_pages.push((p, cache_key.clone(), t, Ok(o)));
          }
          Err(e) => {
            #[cfg(not(feature = "ssr"))]
            loading.set(false);
            error!("err {:#?}", e);
            new_pages.push((p, cache_key, t, Err(e)));
          }
        }
      }
      new_pages
    },
  );

  let on_retry_click = move |_e: MouseEvent| {
    search_cache_resource.refetch();
  };

  let show_next = RwSignal::new(true);
  let now_in_millis = RwSignal::new(u64::try_from(jiff::Timestamp::now().as_millisecond()).unwrap_or(0));

  view! {
    <main class="flex flex-col">
      <TopNav scroll_element={on_scroll_element.into()} />
      <div class="flex flex-grow">
        <div
          on:wheel={handle_horizontal_scroll_wheel(on_scroll_element)}
          node_ref={on_scroll_element}
          class="min-w-full sm:overflow-x-auto sm:overflow-y-hidden sm:absolute sm:px-4 gap-4{} sm:h-[calc(100%-4rem)] sm:columns-[23rem]"
        >
          <Transition fallback={|| {}}>
            <Title text={move || t_string!(i18n, search)} />
            <For each={move || search_cache_resource.get().unwrap_or(vec![])} key={|(page, _, timestamp, _)| (*page, *timestamp)} let:r>
              {
                let (page_number, cache_key, timestamp, result) = &r;
                match result {
                  Ok(o) => {
                    #[cfg(not(feature = "ssr"))]
                    {
                      let result_clone = result.clone();
                      let page_index = *page_number;
                      let fetched_at = *timestamp;
                      let cache_key = cache_key.clone();
                      spawn_local_scoped(async move {
                        search_cache
                          .update(move |sc| {
                            sc.insert((page_index, cache_key), (fetched_at, result_clone));
                          });
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
                                      let current_endpoint = endpoint.get();
                                      if let Ok(Some(l)) = IndexedDb::get_scroll_position(current_endpoint.path(), current_endpoint.query_string()).await {
                                        s.set_scroll_left(l);
                                      }
                                    });
                                  }
                                },
                                std::time::Duration::new(0, 0),
                              )
                              .ok(),
                          );
                      }
                    }
                    #[cfg(not(feature = "ssr"))] loading.set(false);
                    next_page_cursor.set((*page_number + 1, o.users.len() + o.communities.len() + o.posts.len() + o.comments.len()));
                    let comments_clone = RwSignal::new(o.comments.clone());
                    view! {
                      <Listings hide=false posts={o.posts.clone().into()} page_number={RwSignal::new((*page_number - 1) * 50)} />
                      <For each={move || comments_clone.get()} key={|cv| cv.comment.id} let:cv>
                        <div class="odd:bg-base-300 pt-2 pr-4 pb-2 pl-4">
                          <Comment
                            parent_comment_id=0
                            hidden_comments={RwSignal::new(vec![])}
                            comment={cv.clone().into()}
                            comments={vec![].into()}
                            level=0
                            now_in_millis
                            highlight_user_id={RwSignal::new(None)}
                            post_id={Signal::derive(move || Some(cv.post.id.0))}
                            selected_drag_offset={RwSignal::new((0, 0f64, 0))}
                          />
                        </div>
                      </For>
                      <CommunityListings hide=false communities={o.communities.clone().into()} page_number={RwSignal::new((*page_number - 1) * 50)} />
                      <UserListings hide=false people={o.users.clone().into()} page_number={RwSignal::new((*page_number - 1) * 50)} />
                      {
                        let (key, len) = next_page_cursor.get_untracked();
                        if key > 1 && len > 0 {
                          let mut st = vec![];
                          st.push(key);
                          let link_destination = endpoint.get().with_page(st.clone());
                          #[cfg(not(feature = "ssr"))]
                          set_timeout_with_handle(
                            move || {
                              show_next.set(false);
                            },
                            std::time::Duration::new(0, 0),
                          );
                          #[cfg(not(feature = "ssr"))]
                          set_timeout_with_handle(
                            move || {
                              if is_visible.get() {
                                if let Some(c) = cancel_handle.get_untracked() {
                                  c.clear();
                                }
                                let (key, len) = next_page_cursor.get_untracked();
                                if key > 1 && len > 0 {
                                  let mut st = ssr_page();
                                  if st.iter().find(|s| **s == key).is_none() {
                                    st.push(key);
                                  }
                                  let destination = endpoint.get().with_page(st);
                                  let params = destination.query_string();
                                  spawn_local_scoped(async move {
                                    if let Some(se) = on_scroll_element.get_untracked() {
                                      let _ = IndexedDb::save_scroll_position(destination.path(), params, se.scroll_left()).await;
                                    }
                                    navigate_internal_pagination(InternalNavigation::new(destination));
                                  });
                                }

                              };
                            },
                            std::time::Duration::new(0, 250_000_000),
                          );
                          view! {
                            <Show when={move || { show_next.get() }} fallback={|| {}}>
                              <div class="flex py-4 px-8 justify-end{}">
                                <A
                                  href={internal_navigation_url(&InternalNavigation::new(link_destination.clone()))}
                                  attr:class="btn btn-soft"
                                >
                                  {t!(i18n, next)}
                                </A>
                              </div>
                            </Show>
                          }.into_any()
                        } else {
                          view! {}.into_any()
                        }
                      }
                    }.into_any()
                  }
                  Err(LemmyAppError { error_type: LemmyAppErrorType::OfflineError, .. }) => {
                    #[cfg(not(feature = "ssr"))] loading.set(false);
                    view! { <Warning on_retry_click={on_retry_click} /> }.into_any()
                  }
                  Err(e) => {
                    #[cfg(not(feature = "ssr"))] loading.set(false);
                    error!("{:#?}", e);
                    view! { <Error description={t_string!(i18n, error_loading_posts).to_string()} error={e.clone()} on_retry_click={on_retry_click} /> }.into_any()
                  }
                }
              }
            </For>
          </Transition>
          {move || {
            view! { <Info loading={loading.get()} /> }
          }}
          <div node_ref={intersection_element} class="block bg-transparent h-[1px]" />
        </div>
      </div>
    </main>
  }
}
