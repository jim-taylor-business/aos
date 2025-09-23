use crate::{
  errors::{message_from_error, LemmyAppError, LemmyAppErrorType, LemmyAppResult},
  i18n::*,
  lemmy_client::*,
  ui::components::{
    common::about::About,
    home::{site_summary::SiteSummary, trending::Trending},
    post::post_listings::PostListings,
    responsive::{responsive_nav::ResponsiveTopNav, responsive_post_listings::ResponsivePostListings},
  },
  ResourceStatus, ResponseLoad,
};
use codee::string::FromToStringCodec;
use chrono::prelude::*;
use lemmy_api_common::{
  lemmy_db_schema::{ListingType, SortType},
  lemmy_db_views::structs::PaginationCursor,
  post::{GetPosts, GetPostsResponse},
  site::GetSiteResponse,
};
use leptos::{html::*, leptos_dom::helpers::TimeoutHandle, logging::log, svg::view, *};
use leptos_meta::*;
use leptos_router::*;
use leptos_use::*;
// #[cfg(not(feature = "ssr"))]
// use rexie::Rexie;
use std::{
  collections::{BTreeMap, HashMap},
  usize, vec,
};
#[cfg(not(feature = "ssr"))]
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{js_sys::Atomics::wait_async, Event, MouseEvent, ScrollToOptions, TouchEvent, WheelEvent};

#[component]
pub fn ResponsiveHomeActivity(ssr_site: Resource<(Option<String>, Option<String>), Result<GetSiteResponse, LemmyAppError>>) -> impl IntoView {
  let i18n = use_i18n();
  let error = expect_context::<RwSignal<Vec<Option<(LemmyAppError, Option<RwSignal<bool>>)>>>>();
  // #[cfg(not(feature = "ssr"))]
  // let idb_resource = IndexedDb::build_indexed_database();
  // let mut indexed_db = None;
  // spawn_local(async move {
  //   log!("fu");
  //   // IndexedDbApi::set(&IndexedDbApi::new("db_name", 1).await.unwrap(), &ScrollPositionKey { path: use_location().pathname.get(), query: query_params.to_query_string() }, &se.scroll_left().to_string()).await;
  //   // IndexedDb::set(&i, &ScrollPositionKey { path: use_location().pathname.get(), query: query_params.to_query_string() }, &se.scroll_left().to_string()).await;
  // });
  // log!("2");
  // #[cfg(not(feature = "ssr"))]
  // let idb_resource = expect_context::<Option<IndexedDb>>();
  // let idb_resource = expect_context::<Resource<(), IndexedDb>>();


  let param = use_params_map();
  let query = use_query_map();

  let ssr_name = move || param.get().get("name").cloned().unwrap_or("".into());
  let ssr_list = move || serde_json::from_str::<ListingType>(&query.get().get("list").cloned().unwrap_or("".into())).unwrap_or(ListingType::All);
  let ssr_sort = move || serde_json::from_str::<SortType>(&query.get().get("sort").cloned().unwrap_or("".into())).unwrap_or(SortType::Active);
  let ssr_page = move || serde_json::from_str::<Vec<(usize, String)>>(&query.get().get("page").cloned().unwrap_or("".into())).unwrap_or(vec![]);

  let response_cache = expect_context::<RwSignal<BTreeMap<(usize, GetPosts), (i64, LemmyAppResult<GetPostsResponse>)>>>();
  let next_page_cursor: RwSignal<(usize, Option<PaginationCursor>)> = RwSignal::new((0, None));

  let scroll_element = expect_context::<RwSignal<Option<NodeRef<Div>>>>();

  let loading = RwSignal::new(false);
  let refresh = RwSignal::new(false);

  let logged_in = Signal::derive(move || {
    if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site.get() {
      Some(true)
    } else {
      Some(false)
    }
  });

  let sleep = RwSignal::new(false);
  let intersection_element = create_node_ref::<Div>();
  let on_scroll_element = NodeRef::<Div>::new();
  let refresh_base = RwSignal::new(0);

  // spawn_local(async move {
  //   log!("fu");
  //   indexed_db = Some(IndexedDb::build_indexed_database().await.unwrap());

  //   let mut query_params = query.get();
  //   if let Some(se) = on_scroll_element.get() {
  //   // IndexedDbApi::set(&IndexedDbApi::new("db_name", 1).await.unwrap(), &ScrollPositionKey { path: use_location().pathname.get(), query: query_params.to_query_string() }, &se.scroll_left().to_string()).await;
  //   IndexedDb::set(&indexed_db.unwrap(), &ScrollPositionKey { path: use_location().pathname.get(), query: query_params.to_query_string() }, &se.scroll_left().to_string()).await;
  //   }
  // });

  #[cfg(not(feature = "ssr"))]
  let mut idb_resource = expect_context::<RwSignal<Option<IndexedDb>>>();
  // #[cfg(not(feature = "ssr"))]
  // let mut rdb_resource = expect_context::<RwSignal<Option<Rexie>>>();

  #[cfg(not(feature = "ssr"))]
  {
    let on_scroll = move |e: Event| {
      if let Some(se) = on_scroll_element.get() {
        // if ssr_page().len() > 0 {
          // let i = &idb_resource;
          // if let Some(i) = idb_resource.get() {
            spawn_local(async move {
              // log!("fu");
              let mut query_params = query.get();
              // // idb_resource.build();
              // rdb_resource.get();
              if let Some(i) = idb_resource.get() {
                i.set(&ScrollPositionKey { path: use_location().pathname.get(), query: query_params.to_query_string() }, &se.scroll_left()).await;
              }
              // // IndexedDb::set(&i, &ScrollPositionKey { path: use_location().pathname.get(), query: query_params.to_query_string() }, &se.scroll_left().to_string()).await;
            });

          // }
            // spawn_local(async move {
            //   log!("fu");
            //   let indexed_db = IndexedDb::new().await.unwrap();

            //   let mut query_params = query.get();
            //   if let Some(se) = on_scroll_element.get() {
            //   // IndexedDbApi::set(&IndexedDbApi::new("db_name", 1).await.unwrap(), &ScrollPositionKey { path: use_location().pathname.get(), query: query_params.to_query_string() }, &se.scroll_left().to_string()).await;
            //   IndexedDb::set(&indexed_db, &ScrollPositionKey { path: use_location().pathname.get(), query: query_params.to_query_string() }, &se.scroll_left().to_string()).await;
            //   }
            // });

          // if let Ok(Some(s)) = window().local_storage() {
          //   let mut query_params = query.get();
          //   let _ = s.set_item(
          //     &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
          //     &se.scroll_left().to_string(),
          //   );
          // }
        // }
      }
    };

    // let _scroll_handle = window_event_listener_untyped("scroll", on_scroll);

    let UseScrollReturn { .. } = use_scroll_with_options(on_scroll_element, UseScrollOptions::default().on_scroll(on_scroll));

    let UseIntersectionObserverReturn {
      pause,
      resume,
      stop,
      is_active,
    } = use_intersection_observer_with_options(
      intersection_element,
      move |intersections, _| {
        if intersections[0].is_intersecting() {
          if let (key, _) = next_page_cursor.get() {
            if key > 0 {
              let mut st = ssr_page();
              if let (_, Some(PaginationCursor(next_page))) = next_page_cursor.get() {
                if st.len() == 0 {
                  st.push((0usize, "".into()));
                }
                if st.iter().find(|s| s.0 == key).is_none() {
                  st.push((key, next_page));
                }
              }
              let mut query_params = query.get();
              query_params.insert("page".into(), serde_json::to_string(&st).unwrap_or("[]".into()));
              let iw = window().inner_width().ok().map(|b| b.as_f64().unwrap_or(0.0)).unwrap_or(0.0);
              if iw < 768f64 {
              //   if let Ok(Some(s)) = window().local_storage() {
              //     let _ = s.set_item(
              //       &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
              //       &window().scroll_y().unwrap_or(0.0).to_string(),
              //     );
              //   }
              } else {
                if let Some(se) = on_scroll_element.get() {
                  spawn_local(async move {
                    let mut query_params = query.get();
                    if let Some(i) = idb_resource.get() {
                      if let Err(e) = i.set(&ScrollPositionKey { path: use_location().pathname.get(), query: query_params.to_query_string() }, &se.scroll_left()).await {
                        // log!("puu");
                      }
                    }
                  });

                  // if let Ok(Some(s)) = window().local_storage() {
                  //   let _ = s.set_item(
                  //     &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
                  //     &se.scroll_left().to_string(),
                  //   );
                  // }
                }
              }
              sleep.set(true);
              let navigate = leptos_router::use_navigate();
              navigate(
                &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
                NavigateOptions {
                  resolve: false,
                  replace: true,
                  scroll: false,
                  state: State::default(),
                },
              );
            }
          }
        }
      },
      UseIntersectionObserverOptions::default(),
    );
  }

  #[cfg(not(feature = "ssr"))]
  use crate::indexed_db::csr_indexed_db::*;

  let post_list_resource = Resource::new(
    move || (logged_in.get(), ssr_list(), ssr_sort(), ssr_name(), ssr_page()),
    move |(_logged_in, list, sort, name, mut pages)| async move {
      loading.set(true);
      let mut rc = response_cache.get_untracked();
      let mut new_pages: Vec<(usize, GetPosts, i64, LemmyAppResult<GetPostsResponse>)> = vec![];

      if pages.len() == 0 {
        pages = vec![(0usize, "".to_string())];
      }

      for p in pages {
        let form = GetPosts {
          type_: Some(list),
          sort: Some(sort),
          community_name: if name.clone().len() == 0usize { None } else { Some(name.clone()) },
          community_id: None,
          page: None,
          limit: Some(50),
          saved_only: None,
          disliked_only: None,
          liked_only: None,
          page_cursor: if p.0 == 0usize { None } else { Some(PaginationCursor(p.1.clone())) },
          show_hidden: Some(true),
          show_nsfw: Some(false),
          show_read: Some(true),
        };
        #[cfg(not(feature = "ssr"))]
        if let Some((t, r)) = rc.get(&(p.0, form.clone())) {
          // log!("hit {:?}", form);
          match r {
            Ok(_) => {
              new_pages.push((p.0, form.clone(), t.clone(), r.clone()));
            }
            _ => {
              let result = LemmyClient.list_posts(form.clone()).await;
              new_pages.push((p.0, form.clone(), chrono::Utc::now().timestamp_millis(), result));
            }
          }
          continue;
        }

        let result = LemmyClient.list_posts(form.clone()).await;
        new_pages.push((p.0, form.clone(), chrono::Utc::now().timestamp_millis(), result));
      }

      (new_pages)
    },
  );

  let on_retry_click = move |_e: MouseEvent| {
    post_list_resource.refetch();
  };

  let on_retry_site_click = move |_e: MouseEvent| {
    ssr_site.refetch();
  };

  #[cfg(not(feature = "ssr"))]
  let mut cancel_handle: RwSignal<Option<Result<TimeoutHandle, JsValue>>> = RwSignal::new(None);

  view! {
    <main class="flex flex-col">
      <ResponsiveTopNav ssr_site />
      <div class="flex flex-grow">
        <div
          on:wheel={move |e: WheelEvent| {
            e.stop_propagation();
            if let Some(se) = on_scroll_element.get() {
              let mut o = ScrollToOptions::new();
              o.set_left(e.delta_y());
              o.set_behavior(web_sys::ScrollBehavior::Smooth);
              se.scroll_by_with_scroll_to_options(&o);
            }
          }}
          node_ref={on_scroll_element}
          class={move || { "md:h-[calc(100%-4rem)] min-w-full md:absolute md:overflow-x-auto md:overflow-y-hidden md:columns-sm md:px-4 gap-4" }}
        >
          <Transition fallback={|| {}}>
            {move || {
              match ssr_site.get() {
                Some(Err(_)) => {
                  view! {
                    <div class="py-4 px-8 break-inside-avoid">
                      <div class="flex justify-between alert alert-error">
                        <span class="text-lg">{"Site Error"}</span>
                        <span on:click={on_retry_site_click} class="btn btn-sm">
                          "Retry"
                        </span>
                      </div>
                    </div>
                  }
                    .into_view()
                }
                _ => view! {}.into_view(),
              }
            }}
          </Transition>
          <Transition fallback={|| {}}>
            <Title text="" />
            <For each={move || post_list_resource.get().unwrap_or(vec![])} key={|p| (p.1.clone(), p.2)} let:p>
              {match p.3 {
                Ok(ref o) => {
                  #[cfg(not(feature = "ssr"))]
                  {
                    let rw = p.3.clone();
                    let fm = p.1.clone();
                    use crate::indexed_db::csr_indexed_db::*;
                    spawn_local(async move {
                      if let Ok(d) = build_indexed_database().await {
                        if let Ok(c) = set_query_get_cache::<GetPosts, Result<GetPostsResponse, LemmyAppError>>(&d, &fm, &rw).await {}
                      }
                      response_cache
                        .update(move |rc| {
                          rc.insert((p.0, fm), (p.2, rw));
                        });
                    });
                    let iw = window().inner_width().ok().map(|b| b.as_f64().unwrap_or(0.0)).unwrap_or(0.0);
                    if iw < 768f64 {
                    } else {
                      if let Some(Ok(c)) = cancel_handle.get_untracked() {
                        c.clear();
                      }
                      cancel_handle.set(
                        Some(
                          set_timeout_with_handle(
                            move || {
                              if let Some(se) = on_scroll_element.get() {
                                spawn_local(async move {
                                  if let Some(i) = idb_resource.get() {
                                    let mut query_params = query.get();
                                    let l: Result<Option<i32>, Error> = i.get(&ScrollPositionKey { path: use_location().pathname.get(), query: query_params.to_query_string() }).await;
                                    if let Ok(Some(l)) = l {
                                      se.set_scroll_left(l);
                                    };
                                  }
                                });

                                // if let Ok(Some(s)) = window().local_storage() {
                                //   let mut query_params = query.get();
                                //   if let Ok(Some(l)) = s.get_item(&format!("{}{}", use_location().pathname.get(), query_params.to_query_string())) {
                                //     se.set_scroll_left(l.parse().unwrap_or(0i32));
                                //   }
                                // }
                                scroll_element.set(Some(on_scroll_element));
                              }
                            },
                            std::time::Duration::new(0, 750_000_000),
                          ),
                        ),
                      );
                    }
                  }
                  next_page_cursor.set((p.0 + 50usize, o.next_page.clone()));
                  loading.set(false);

                  // #[cfg(not(feature = "ssr"))]
                  // log!("store");
                  // rc.insert((p.0, if let Some(pc) = fm.page_cursor { pc.0 } else { "".into() } /*if p.0 == 0 { "".into() } else { p.1.clone() }*/, fm.type_.unwrap_or(ListingType::All), fm.sort.unwrap_or(SortType::Active), fm.community_name.unwrap_or("".into())), rw);

                  // log!("set {}", l);

                  // log!("next {} {:?}", p.0 + 50usize, o.next_page.clone());

                  view! { <ResponsivePostListings posts={o.posts.clone().into()} ssr_site page_number={p.0.into()} /> }
                    .into_view()
                }
                Err(LemmyAppError { error_type: LemmyAppErrorType::OfflineError, .. }) => {
                  loading.set(false);
                  view! {
                    <div class="py-4 px-8 break-inside-avoid">
                      <div class="flex justify-between alert alert-warning">
                        <span class="text-lg">{"Offline"}</span>
                        <span on:click={on_retry_click} class="btn btn-sm">
                          "Retry"
                        </span>
                      </div>
                    </div>
                  }
                    .into_view()
                }
                _ => {
                  loading.set(false);
                  view! {
                    <div class="py-4 px-8 break-inside-avoid">
                      <div class="flex justify-between alert alert-error">
                        <span class="text-lg">{"Error"}</span>
                        <span on:click={on_retry_click} class="btn btn-sm">
                          "Retry"
                        </span>
                      </div>
                    </div>
                  }
                    .into_view()
                }
              }}
            </For>
            <div node_ref={intersection_element} class="block bg-transparent h-[1px]" />
            {move || {
              if loading.get() {
                Some(
                  view! {
                    <div class="overflow-hidden break-inside-avoid animate-[popdown_1s_step-end_1]">
                      <div class="py-4 px-8">
                        <div class="alert">
                          <span>"Loading..."</span>
                        </div>
                      </div>
                    </div>
                  },
                )
              } else {
                None
              }
            }}
          </Transition>
        </div>
      </div>
    </main>
  }
}
