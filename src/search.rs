use crate::ReadAuthCookie;
use crate::{
  client::*,
  errors::*,
  listing::Listings,
  nav::TopNav,
  // i18n::*,
};
use crate::{db::csr_indexed_db::*, errors::Loading};
use lemmy_api_common::{
  lemmy_db_schema::{ListingType, SearchType, SortType},
  site::{GetSiteResponse, Search, SearchResponse},
};
use leptos::{html::Div, logging::error, prelude::*, task::*, *};
use leptos_meta::Title;
use leptos_router::{components::*, hooks::*};
use std::{collections::BTreeMap, usize, vec};
use web_sys::{MouseEvent, WheelEvent};

#[component]
pub fn Search() -> impl IntoView {
  // let i18n = use_i18n();
  let param = use_params_map();
  let ssr_name = move || param.get().get("name").unwrap_or("".into());

  let query = use_query_map();
  let ssr_list = move || serde_json::from_str::<ListingType>(&query.get().get("list").unwrap_or("".into())).unwrap_or(ListingType::All);
  let ssr_sort = move || serde_json::from_str::<SortType>(&query.get().get("sort").unwrap_or("".into())).unwrap_or(SortType::Active);
  let ssr_page = move || serde_json::from_str::<Vec<usize>>(&query.get().get("page").unwrap_or("".into())).unwrap_or(vec![1usize]);
  let ssr_term = move || query.get().get("term").unwrap_or("".into());

  let next_page_cursor: RwSignal<usize> = RwSignal::new(0);

  let loading = RwSignal::new(false);

  let intersection_element = NodeRef::<Div>::new();
  let on_scroll_element = NodeRef::<Div>::new();

  #[cfg(not(feature = "ssr"))]
  {
    use leptos_router::{NavigateOptions, location::State};
    use leptos_use::{
      UseIntersectionObserverOptions, UseIntersectionObserverReturn, UseScrollOptions, UseScrollReturn, use_intersection_observer_with_options,
      use_scroll_with_options,
    };
    use web_sys::Event;

    let on_scroll = move |_e: Event| {
      if ssr_page().len() > 1 {
        if let Some(se) = on_scroll_element.get() {
          #[cfg(not(feature = "ssr"))]
          spawn_local_scoped(async move {
            let query_params = query.get();
            if let Ok(d) = IndexedDb::new().await {
              let _ =
                d.set(&ScrollPositionKey { path: use_location().pathname.get(), query: query_params.to_query_string() }, &se.scroll_left()).await;
            }
          });
        }
      }
    };

    let UseScrollReturn { .. } = use_scroll_with_options(on_scroll_element, UseScrollOptions::default().on_scroll(on_scroll));
    let UseIntersectionObserverReturn { .. } = use_intersection_observer_with_options(
      intersection_element,
      move |intersections, _| {
        if intersections[0].is_intersecting() {
          let key = next_page_cursor.get();
          if key > 0 {
            let mut st = ssr_page();
            st.push(key);
            let mut query_params = query.get();
            query_params.insert("page", serde_json::to_string(&st).unwrap_or("[]".into()));

            let navigate = use_navigate();
            navigate(
              &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
              NavigateOptions { resolve: true, replace: false, scroll: false, state: State::default() },
            );
          }
        }
      },
      UseIntersectionObserverOptions::default(),
    );
  }

  #[cfg(not(feature = "ssr"))]
  let cancel_handle: RwSignal<Option<TimeoutHandle>> = RwSignal::new(None);

  // let search_cache: RwSignal<BTreeMap<(usize, Search, Option<String>), (i64, LemmyAppResult<SearchResponse>)>> = RwSignal::new(BTreeMap::new());
  let search_cache = expect_context::<RwSignal<BTreeMap<(usize, Search, Option<String>), (i64, LemmyAppResult<SearchResponse>)>>>();

  let search_cache_resource = Resource::new(
    move || (ssr_list(), ssr_sort(), ssr_name(), ssr_page(), ssr_term()),
    move |(_list, sort, _name, pages, term)| async move {
      let ReadAuthCookie(get_auth_cookie) = expect_context::<ReadAuthCookie>();
      let sc = search_cache.get_untracked();
      let mut new_pages: Vec<(usize, Search, i64, LemmyAppResult<SearchResponse>, Option<String>)> = Vec::new();
      let len = pages.len();
      for p in pages {
        let form = Search {
          q: term.clone(),
          type_: Some(SearchType::Posts),
          sort: Some(sort),
          community_name: None,
          community_id: None,
          page: Some(p as i64),
          limit: Some(50),
          creator_id: None,
          listing_type: None,
          post_title_only: None,
        };

        #[cfg(not(feature = "ssr"))]
        loading.set(true);

        #[cfg(not(feature = "ssr"))]
        if let Some((t, Ok(r))) = sc.get(&(p, form.clone(), get_auth_cookie.get_untracked())) {
          new_pages.push((p, form.clone(), t.clone(), Ok(r.clone()), get_auth_cookie.get_untracked()));
          continue;
          // } else {
          //   if many_pages {
          //     match load_cache(form.clone()).await {
          //       Ok(o) => {
          //         new_pages.push((
          //           p.0,
          //           form.clone(),
          //           jiff::Zoned::now().timestamp().as_millisecond(),
          //           Ok(o),
          //           get_auth_cookie.get_untracked(),
          //           do_not_render_scroll,
          //           csr_cache_render,
          //         ));
          //         continue;
          //       }
          //       _ => {}
          //     }
          //   }
        }

        let t = jiff::Zoned::now().timestamp().as_millisecond();
        let result = LemmyClient.search(form.clone()).await;
        match result {
          Ok(o) => {
            new_pages.push((p, form.clone(), t, Ok(o), get_auth_cookie.get_untracked()));
          }
          Err(e) => {
            #[cfg(not(feature = "ssr"))]
            loading.set(false);
            error!("err {:#?}", e);
            new_pages.push((p, form.clone(), t, Err(e), get_auth_cookie.get_untracked()));
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

  view! {
    <main class="flex flex-col">
      <TopNav scroll_element={on_scroll_element.into()} />
      <div class="flex flex-grow">
        <div
          on:wheel={move |e: WheelEvent| {
            let iw = window().inner_width().ok().map(|b| b.as_f64().unwrap_or(0.0)).unwrap_or(0.0);
            if iw < 768f64 {} else {
              if e.delta_x() != 0.0 {
                if e.delta_y().abs() / e.delta_x().abs() < 0.3 {} else {
                  e.prevent_default();
                  if let Some(se) = on_scroll_element.get() {
                    se.set_scroll_left(se.scroll_left() + e.delta_y() as i32);
                  }
                }
              } else {
                e.prevent_default();
                if let Some(se) = on_scroll_element.get() {
                  se.set_scroll_left(se.scroll_left() + e.delta_y() as i32);
                }
              }
            }
          }}
          node_ref={on_scroll_element}
          class="min-w-full sm:overflow-x-auto sm:overflow-y-hidden sm:absolute sm:px-4 gap-4{} sm:h-[calc(100%-4rem)] sm:columns-[23rem]"
        >
          <Transition fallback={|| {}}>
            <Title text="Search" />
            <For each={move || search_cache_resource.get().unwrap_or(vec![])} key={|r| (r.1.clone(), r.2, r.4.clone())} let:r>
              {
                match r.3 {
                  Ok(ref o) => {
                    #[cfg(not(feature = "ssr"))]
                    {
                      let result_clone = r.3.clone();
                      // let fm = p.1.clone();
                      use crate::db::csr_indexed_db::*;
                      spawn_local_scoped(async move {
                        // if p.6 {} else {
                          if let Ok(d) = IndexedDb::new().await {
                            if let Ok(_c) = d.set::<Search, Result<SearchResponse, LemmyAppError>>(&r.1, &result_clone).await {}
                          }
                          search_cache.update(move |sc| {
                            sc.insert((r.0, r.1, r.4), (r.2, result_clone));
                          });
                        // }
                      });
                      let iw = window().inner_width().ok().map(|b| b.as_f64().unwrap_or(0.0)).unwrap_or(0.0);
                      if iw < 768f64 /* || p.5 || p.6 */ {} else {
                        if let Some(c) = cancel_handle.get_untracked() {
                          c.clear();
                        }
                        cancel_handle.set(
                          set_timeout_with_handle(
                            move || {
                              if let Some(s) = on_scroll_element.get() {
                                spawn_local_scoped(async move {
                                  if let Ok(d) = IndexedDb::new().await {
                                    let l: Result<Option<i32>, Error> = d
                                      .get(
                                        &ScrollPositionKey {
                                          path: use_location().pathname.get(),
                                          query: use_query_map().get().to_query_string(),
                                        },
                                      )
                                      .await;
                                    if let Ok(Some(l)) = l {
                                      s.set_scroll_left(l);
                                    }
                                  }
                                });
                              }
                            },
                            std::time::Duration::new(0, 750_000_000),
                          ).ok(),
                        );
                      }
                      // if p.6 {
                      //   if let Some(c) = cancel_refresh_handle.get_untracked() {
                      //     c.clear();
                      //   }
                      //   cancel_refresh_handle.set(
                      //     set_timeout_with_handle(
                      //       move || {
                      //         post_list_resource.refetch();
                      //       },
                      //       std::time::Duration::new(0, 750_000_000),
                      //     ).ok(),
                      //   );
                      // }
                    }
                    #[cfg(not(feature = "ssr"))] loading.set(false);
                    next_page_cursor.set(r.0 + 1);
                    view! {
                      <Listings hide=false posts={o.posts.clone().into()} page_number={RwSignal::new(((r.0 - 1) * 50) as usize)} />
                      {
                        // let (key, _) = next_page_cursor.get();
                        // if key > 0 {
                          // let mut st = ssr_page();
                          let mut st: Vec<(usize)> = vec![next_page_cursor.get()];
                          // if let (_, Some(PaginationCursor(next_page))) = next_page_cursor.get() {
                          //   // if st.len() == 0 {
                          //   //   st.push((0usize, "".into()));
                          //   // }
                          //   // if st.iter().find(|s| s.0 == key).is_none() {
                              // st.push(next_page_cursor.get()));
                          //   // }
                          // }
                          let mut query_params = use_query_map().get();
                          query_params.remove("page");
                          query_params.insert("page", serde_json::to_string(&st).unwrap_or("[]".into()));

                          #[cfg(not(feature = "ssr"))]
                          set_timeout_with_handle(
                            move || {
                              show_next.set(false);
                            },
                            std::time::Duration::new(0, 50_000_000),
                          );

                          view! {
                            // <div class="flex justify-items-end">
                            // <div class="overflow-hidden break-inside-avoid animate-[popdown_1s_step-end_1]">
                              <div class=move || format!("py-4 px-8 flex justify-end{}", if show_next.get() { "" } else { " hidden" })>
                                <A href=format!("{}{}", use_location().pathname.get(), query_params.to_query_string()) attr:class="btn btn-soft">
                                  "Next"
                                </A>
                              </div>
                            // </div>
                          }.into_any()
                        // } else {
                        //   view! {}.into_any()
                        // }
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
                    view! { <Error description="Error loading post list"  error={e} on_retry_click={on_retry_click} /> }.into_any()
                  }
                  // _ => view! {}.into_any(),
                }
              }
            </For>
          </Transition>
          <div node_ref={intersection_element} class="block bg-transparent h-[1px]" />
          {move || {
            view! { <Loading loading={loading.get()} /> }
          }}
        </div>
      </div>
    </main>
  }
}
