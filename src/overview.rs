use crate::ReadAuthCookie;
use crate::db::csr_indexed_db::*;
use crate::errors::Offline;
use crate::{
  // i18n::*,
  client::*,
  errors::{Error, LemmyAppError, LemmyAppErrorType, LemmyAppResult, Loading},
  listings::Listings,
  nav::TopNav,
};
use hooks::*;
use lemmy_api_common::site::MyUserInfo;
use lemmy_api_common::{
  lemmy_db_schema::{ListingType, SortType},
  lemmy_db_views::structs::PaginationCursor,
  post::{GetPosts, GetPostsResponse},
  site::GetSiteResponse,
};
use leptos::{
  html::Div,
  leptos_dom::helpers::TimeoutHandle,
  logging::{error, log},
  prelude::*,
  task::*,
  *,
};
use leptos_meta::*;
use leptos_router::params::ParamsMap;
use leptos_router::{components::*, location::State, *};
use leptos_use::*;
use std::{collections::BTreeMap, usize, vec};
use web_sys::{Event, MouseEvent, ScrollToOptions, WheelEvent};

#[component]
pub fn Overview(
  // param: Memo<ParamsMap>,
  // query: Memo<ParamsMap>,
  #[prop(optional)] g: Option<GetSiteResponse>,
  #[prop(optional)] ssr_name: Signal<Option<String>>,
) -> impl IntoView {
  // let i18n = use_i18n();

  // let param = use_params_map();
  // let query = use_query_map();

  // let ssr_name = move || param.get().get("name").unwrap_or("".into());
  let ssr_list = move || serde_json::from_str::<ListingType>(&use_query_map().get().get("list").unwrap_or("".into())).unwrap_or(ListingType::All);
  let ssr_sort = move || serde_json::from_str::<SortType>(&use_query_map().get().get("sort").unwrap_or("".into())).unwrap_or(SortType::Active);
  let ssr_page = move || serde_json::from_str::<Vec<(usize, String)>>(&use_query_map().get().get("page").unwrap_or("".into())).unwrap_or(vec![]);

  let response_cache = expect_context::<RwSignal<BTreeMap<(usize, GetPosts, Option<String>), (i64, LemmyAppResult<GetPostsResponse>)>>>();
  let next_page_cursor: RwSignal<(usize, Option<PaginationCursor>)> = RwSignal::new((0, None));

  // let scroll_element = expect_context::<RwSignal<Option<NodeRef<Div>>>>();

  let loading = RwSignal::new(false);
  // let scroll_save = RwSignal::new(true);

  let ssr_site = expect_context::<Resource<Result<GetSiteResponse, LemmyAppError>>>();
  // let ssr_site_signal = expect_context::<RwSignal<Option<GetSiteResponse>>>();
  // let ssr_user_signal = expect_context::<RwSignal<Option<MyUserInfo>>>();
  // let logged_in = move || {
  //   // log!("TRIGGER");
  //   false
  //   // ssr_user_signal.get().is_some()
  //   // if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site.get() { true } else { false }
  // };
  // let logged_in = RwSignal::new(false);
  // let logged_in = RwSignal::new(if g.is_some() { g.unwrap().my_user.is_some() } else { false });

  let intersection_element = NodeRef::<Div>::new();
  let on_scroll_element = NodeRef::<Div>::new();

  // #[cfg(not(feature = "ssr"))]
  // let scroll_handle: RwSignal<Option<TimeoutHandle>> = RwSignal::new(None);

  // #[cfg(not(feature = "ssr"))]
  // {
  let on_scroll = move |e: Event| {
    // #[cfg(not(feature = "ssr"))]
    // {
    //   if let Some(c) = scroll_handle.get_untracked() {
    //     c.clear();
    //   }
    //   scroll_handle.set(
    //     set_timeout_with_handle(
    //       move || {
    #[cfg(not(feature = "ssr"))]
    if let Some(se) = on_scroll_element.get() {
      spawn_local_scoped_with_cancellation(async move {
        if let Ok(d) = IndexedDb::new().await {
          // let use_query_map()_params = query.get();
          // log!("  SAVE 3 {}", scroll_save.get());
          // if scroll_save.get() {
          //   log!("  SAVE 4 {}", scroll_save.get());
          let _ = d
            .set(&ScrollPositionKey { path: use_location().pathname.get(), query: use_query_map().get().to_query_string() }, &se.scroll_left())
            .await;
          // log!("stash {} {} {}", se.scroll_left(), use_location().pathname.get(), query_params.to_query_string(),);
          // }
        }
      });
    }

    // if let Some(s) = on_scroll_element.get() {
    //   #[cfg(not(feature = "ssr"))]
    //   spawn_local_scoped_with_cancellation(async move {
    //     if let Ok(d) = IndexedDb::new().await {
    //       let query_params = query.get();
    //       let l: Result<Option<i32>, Error> =
    //         d.get(&ScrollPositionKey { path: use_location().pathname.get(), query: query_params.to_query_string() }).await;
    //       if let Ok(Some(l)) = l {
    //         log!("set {} {} {}", l, use_location().pathname.get(), query_params.to_query_string());
    //         s.set_scroll_left(l);
    //       }
    //     }
    //     log!("  SAVE 2 {}", scroll_save.get());
    //     // scroll_save.set(true);
    //   });
    //   // scroll_element.set(Some(on_scroll_element));
    // }
    //       },
    //       std::time::Duration::new(0, 750_000_000),
    //     )
    //     .ok(),
    //   );
    // }

    // if let Some(se) = on_scroll_element.get() {
    //   #[cfg(not(feature = "ssr"))]
    //   spawn_local_scoped_with_cancellation(async move {
    //     if let Ok(d) = IndexedDb::new().await {
    //       let query_params = query.get();
    //       log!("  SAVE 3 {}", scroll_save.get());
    //       if scroll_save.get() {
    //         log!("  SAVE 4 {}", scroll_save.get());
    //         let _ = d.set(&ScrollPositionKey { path: use_location().pathname.get(), query: query_params.to_query_string() }, &se.scroll_left()).await;
    //         log!("stash {} {} {}", se.scroll_left(), use_location().pathname.get(), query_params.to_query_string(),);
    //       }
    //     }
    //   });
    // }
  };

  #[cfg(not(feature = "ssr"))]
  {
    let UseScrollReturn { .. } = use_scroll_with_options(on_scroll_element, UseScrollOptions::default().on_scroll(on_scroll));

    let UseIntersectionObserverReturn { .. } = use_intersection_observer_with_options(
      intersection_element,
      move |intersections, _| {
        if intersections[0].is_intersecting() {
          let (key, _) = next_page_cursor.get();
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
            let mut query_params = use_query_map().get();
            query_params.remove("page");
            query_params.insert("page", serde_json::to_string(&st).unwrap_or("[]".into()));

            // let params = query_params.clone();

            // let iw = window().inner_width().ok().map(|b| b.as_f64().unwrap_or(0.0)).unwrap_or(0.0);
            // if iw < 768f64 {
            // } else {
            #[cfg(not(feature = "ssr"))]
            if let Some(se) = on_scroll_element.get() {
              let params = query_params.clone();
              spawn_local_scoped_with_cancellation(async move {
                if let Ok(d) = IndexedDb::new().await {
                  let _ = d.set(&ScrollPositionKey { path: use_location().pathname.get(), query: params.to_query_string() }, &se.scroll_left()).await;
                  // log!("inter {} {} {}", se.scroll_left(), use_location().pathname.get(), params.to_query_string(),);
                }
                // log!("  SAVE 1 ");
                // scroll_save.set(false);
                use_navigate()(
                  &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
                  NavigateOptions { resolve: false, replace: true, scroll: false, state: State::default() },
                );
                // if let Some(se) = on_scroll_element.get() {
                //   log!("navigate {} {} {}", se.scroll_left(), use_location().pathname.get(), params.to_query_string(),);
                // }
              });
            }
            // }
          }
        }
      },
      UseIntersectionObserverOptions::default(),
    );
  }

  let post_list_resource = Resource::new(
    move || (/*logged_in(), */ ssr_list(), ssr_sort(), ssr_name.get(), ssr_page()),
    move |(/*_logged_in, */ list, sort, name, mut pages)| async move {
      #[cfg(feature = "ssr")]
      let render_scroll = true && pages.len() == 0;
      #[cfg(not(feature = "ssr"))]
      let render_scroll = false;

      let ReadAuthCookie(get_auth_cookie) = expect_context::<ReadAuthCookie>();

      // let _logged_in = logged_in.get();
      // let _logged_in = false;
      // log!("LOAD {:?} {:?} {:?} {:?} {:?}", list, sort, name, pages, get_auth_cookie.get_untracked());
      // log!("LOAD");

      #[cfg(not(feature = "ssr"))]
      loading.set(true);
      let rc = response_cache.get_untracked();
      let mut new_pages: Vec<(usize, GetPosts, i64, LemmyAppResult<GetPostsResponse>, Option<String>, bool)> = vec![];
      if pages.len() == 0 {
        pages = vec![(0usize, "".to_owned())];
      }
      for p in pages {
        let form = GetPosts {
          type_: Some(list),
          sort: Some(sort),
          community_name: name.clone(),
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
        if let Some((t, r)) = rc.get(&(p.0, form.clone(), get_auth_cookie.get_untracked())) {
          // log!("cached");
          match r {
            Ok(_) => {
              // log!("cache");
              new_pages.push((p.0, form.clone(), t.clone(), r.clone(), get_auth_cookie.get_untracked(), render_scroll));
            }
            _ => {
              let result = match LemmyClient.list_posts(form.clone()).await {
                Ok(mut o) => {
                  o.posts.retain(|p| !p.banned_from_community);
                  Ok(o)
                }
                Err(e) => Err(e),
              };
              new_pages.push((p.0, form.clone(), chrono::Utc::now().timestamp_millis(), result, get_auth_cookie.get_untracked(), render_scroll));
            }
          }
          continue;
        }
        let result = match LemmyClient.list_posts(form.clone()).await {
          Ok(mut o) => {
            o.posts.retain(|p| !p.banned_from_community);
            Ok(o)
          }
          Err(e) => Err(e),
        };
        new_pages.push((p.0, form.clone(), chrono::Utc::now().timestamp_millis(), result, get_auth_cookie.get_untracked(), render_scroll));
      }
      new_pages
    },
  );

  let on_retry_click = move |_e: MouseEvent| {
    post_list_resource.refetch();
  };

  let on_retry_site_click = move |_e: MouseEvent| {
    spawn_local_scoped_with_cancellation(async move {
      // ssr_site.refetch();
      // ssr_site_signal.set(Some(LemmyClient.get_site().await));
      LemmyClient.get_site().await;
    });
  };

  // log!("PAGE");

  #[cfg(not(feature = "ssr"))]
  let cancel_handle: RwSignal<Option<TimeoutHandle>> = RwSignal::new(None);

  // let ssr_render_signal = RwSignal::new(false);

  // let csr_render_signal = RwSignal::new(false);

  // log!("COMPONENT");

  view! {
    // <A href="/c/technology"> "TECH" </A>
    // <A href="/"> "ROOT" </A>
    // <A href="/p/42674596"> "POST" </A>

    // <Transition fallback={|| {}}>
      <main class="flex flex-col">
      // <Transition fallback={|| {}}>
        <TopNav scroll_element=on_scroll_element.into() />
        // </Transition>
        <div class="flex flex-grow">
          <div
            on:wheel={move |e: WheelEvent| {
              // e.stop_propagation();
              if e.delta_x() != 0.0 {
                // log!("{} {} {}", e.delta_y().abs() / e.delta_x().abs() , e.delta_x(), e.delta_y());
                if e.delta_y().abs() / e.delta_x().abs() < 0.3 {
                } else {
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
            }}
            // on:scroll=on_scroll
            node_ref={on_scroll_element}
            class={move || { "md:h-[calc(100%-4rem)] min-w-full md:absolute md:overflow-x-auto md:overflow-y-hidden md:columns-sm md:px-4 gap-4" }}
          >
            <Transition fallback={|| {}}>
              {move || {
                match ssr_site.get() {
                  Some(Err(_)) => {
                    view! {
                      <div class="py-4 px-8 break-inside-avoid">
                        <div class="flex justify-between alert alert-error alert-soft">
                          <span class="text-lg">{"Site Error"}</span>
                          <span on:click={on_retry_site_click} class="btn btn-sm">
                            "Retry"
                          </span>
                        </div>
                      </div>
                    }.into_any()
                  }
                  Some(Ok(s)) => {
                    view! {
                      // <div class="py-4 px-8 break-inside-avoid">
                      //   <div class="flex justify-between alert alert-error alert-soft">
                      //     <span class="text-lg">{"Site Error"}</span>
                      //     <span on:click={on_retry_site_click} class="btn btn-sm">
                      //       "Retry"
                      //     </span>
                      //   </div>
                      // </div>
                    }.into_any()
                    // log!("adf");
                  //   logged_in.set(s.my_user.is_some());
                  //   // let logged_in = Memo::new(move |_| { s.my_user.is_some()});
                    // view! {
                  //     <For each={move || post_list_resource.get().unwrap_or(vec![])} key={|p| (p.1.clone(), p.2, p.4)} let:p>
                  //       {match p.3 {
                  //         Ok(ref o) => {
                  //           // #[cfg(feature = "ssr")]
                  //           // log!("HOMEYY ssr");
                  //           // #[cfg(feature = "ssr")]
                  //           // let ssr_render_signal = RwSignal::new(true);

                  //           // #[cfg(not(feature = "ssr"))]
                  //           // log!("HOMEYY csr");
                  //           // #[cfg(not(feature = "ssr"))]
                  //           // let csr_render_signal = RwSignal::new(true);

                  //           // log!(" SIDE  ? {} {} {}", ssr_render_signal.get(), csr_render_signal.get(), p.5);


                  //           // log!("RENDER {}", p.0);
                  //           #[cfg(not(feature = "ssr"))]
                  //           {
                  //             let rw = p.3.clone();
                  //             let fm = p.1.clone();
                  //             use crate::db::csr_indexed_db::*;
                  //             spawn_local_scoped_with_cancellation(async move {
                  //               if let Ok(d) = IndexedDb::new().await {
                  //                 if let Ok(_c) = d.set::<GetPosts, Result<GetPostsResponse, LemmyAppError>>(&fm, &rw).await {}
                  //               }
                  //               response_cache
                  //                 .update(move |rc| {
                  //                   rc.insert((p.0, fm, Some(p.4)), (p.2, rw));
                  //                 });
                  //             });
                  //             let iw = window().inner_width().ok().map(|b| b.as_f64().unwrap_or(0.0)).unwrap_or(0.0);
                  //             if iw < 768f64 || p.5 {} else {
                  //               if let Some(c) = cancel_handle.get_untracked() {
                  //                 c.clear();
                  //               }
                  //               cancel_handle.set(set_timeout_with_handle(
                  //                 move || {
                  //                   if let Some(s) = on_scroll_element.get() {
                  //                     spawn_local_scoped_with_cancellation(async move {
                  //                       if let Ok(d) = IndexedDb::new().await {
                  //                         let query_params = query.get();
                  //                         let l: Result<Option<i32>, Error> = d
                  //                           .get(
                  //                             &ScrollPositionKey {
                  //                               path: use_location().pathname.get(),
                  //                               query: query_params.to_query_string(),
                  //                             },
                  //                           )
                  //                           .await;
                  //                         if let Ok(Some(l)) = l {
                  //                           // log!("set {} {} {}", l, use_location().pathname.get(), query_params.to_query_string());
                  //                           s.set_scroll_left(l);
                  //                         }
                  //                         // scroll_save.set(true);
                  //                       }
                  //                       // log!("  SAVE 2 {}", scroll_save.get());
                  //                     });
                  //                     // scroll_element.set(Some(on_scroll_element));
                  //                   }
                  //                 },
                  //                 std::time::Duration::new(0, 750_000_000),
                  //               ).ok());
                  //             }
                  //           }
                  //           next_page_cursor.set((p.0 + 50usize, o.next_page.clone()));
                  //           loading.set(false);
                  //           view! { <Listings posts={o.posts.clone().into()} page_number={RwSignal::new(p.0)} /> }
                  //             .into_any()
                  //         }
                  //         Err(LemmyAppError { error_type: LemmyAppErrorType::OfflineError, .. }) => {
                  //           loading.set(false);
                  //           view! {
                  //             <Offline on_retry_click={Some(on_retry_click)} /*on_retry_click={None::<Option<_>>}*/ />
                  //             // <div class="py-4 px-8 break-inside-avoid">
                  //             //   <div class="flex justify-between alert alert-warning alert-soft">
                  //             //     <span class="text-lg">{"Offline"}</span>
                  //             //     <span on:click={on_retry_click} class="btn btn-sm">
                  //             //       "Retry"
                  //             //     </span>
                  //             //   </div>
                  //             // </div>
                  //           }
                  //           .into_any()
                  //         }
                  //         Err(e) => {
                  //           loading.set(false);
                  //           error!("{:#?}", e);
                  //           view! {
                  //             <Error error={e} on_retry_click={Some(on_retry_click)} /*on_retry_click={None::<Option<_>>}*/ />
                  //             // <div class="py-4 px-8 break-inside-avoid">
                  //             //   <div class="flex justify-between alert alert-error alert-soft">
                  //             //     <span class="text-lg">{"Error"}</span>
                  //             //     <span on:click={on_retry_click} class="btn btn-sm">
                  //             //       "Retry"
                  //             //     </span>
                  //             //   </div>
                  //             // </div>
                  //           }
                  //           .into_any()
                  //         }
                  //       }}
                  //     </For>
                    // }.into_any()
                  }
                  _ => view! {}.into_any(),
                }
              }}
            </Transition>
        // <Show when={move || render_signal.get()} fallback={|| {}}>
            <Transition fallback={|| {}}>
              // <Title text="" />
              <For each={move || post_list_resource.get().unwrap_or(vec![])} key={|p| (p.1.clone(), p.2, p.4.clone())} let:p>
                {match p.3 {
                  Ok(ref o) => {
                    // #[cfg(feature = "ssr")]
                    // log!("HOMEYY ssr");
                    // #[cfg(feature = "ssr")]
                    // let ssr_render_signal = RwSignal::new(true);

                    // #[cfg(not(feature = "ssr"))]
                    // log!("HOMEYY csr");
                    // #[cfg(not(feature = "ssr"))]
                    // let csr_render_signal = RwSignal::new(true);

                    // log!(" SIDE  ? {} {} {}", ssr_render_signal.get(), csr_render_signal.get(), p.5);


                    // log!("RENDER {}", p.0);
                    #[cfg(not(feature = "ssr"))]
                    {
                      let rw = p.3.clone();
                      let fm = p.1.clone();
                      use crate::db::csr_indexed_db::*;
                      spawn_local_scoped_with_cancellation(async move {
                        if let Ok(d) = IndexedDb::new().await {
                          if let Ok(_c) = d.set::<GetPosts, Result<GetPostsResponse, LemmyAppError>>(&fm, &rw).await {}
                        }
                        response_cache
                          .update(move |rc| {
                            // log!("add {:?}", fm);

                            rc.insert((p.0, fm, p.4), (p.2, rw));
                          });
                      });
                      let iw = window().inner_width().ok().map(|b| b.as_f64().unwrap_or(0.0)).unwrap_or(0.0);
                      if iw < 768f64 || p.5 {} else {
                        if let Some(c) = cancel_handle.get_untracked() {
                          c.clear();
                        }
                        cancel_handle.set(set_timeout_with_handle(
                          move || {
                            if let Some(s) = on_scroll_element.get() {
                              spawn_local_scoped_with_cancellation(async move {
                                if let Ok(d) = IndexedDb::new().await {
                                  // let query_params = query.get();
                                  let l: Result<Option<i32>, Error> = d
                                    .get(
                                      &ScrollPositionKey {
                                        path: use_location().pathname.get(),
                                        query: use_query_map().get().to_query_string(),
                                      },
                                    )
                                    .await;
                                  if let Ok(Some(l)) = l {
                                    // log!("set {} {} {}", l, use_location().pathname.get(), query_params.to_query_string());
                                    s.set_scroll_left(l);
                                  }
                                  // scroll_save.set(true);
                                }
                                // log!("  SAVE 2 {}", scroll_save.get());
                              });
                              // scroll_element.set(Some(on_scroll_element));
                            }
                          },
                          std::time::Duration::new(0, 750_000_000),
                        ).ok());
                      }
                    }
                    next_page_cursor.set((p.0 + o.posts.len(), o.next_page.clone()));
                    #[cfg(not(feature = "ssr"))]
                    loading.set(false);
                    view! { <Listings posts={o.posts.clone().into()} page_number={RwSignal::new(p.0)} /> }
                      .into_any()
                  }
                  Err(LemmyAppError { error_type: LemmyAppErrorType::OfflineError, .. }) => {
                    #[cfg(not(feature = "ssr"))]
                    loading.set(false);
                    view! {
                      <Offline on_retry_click={Some(on_retry_click)} /*on_retry_click={None::<Option<_>>}*/ />
                      // <div class="py-4 px-8 break-inside-avoid">
                      //   <div class="flex justify-between alert alert-warning alert-soft">
                      //     <span class="text-lg">{"Offline"}</span>
                      //     <span on:click={on_retry_click} class="btn btn-sm">
                      //       "Retry"
                      //     </span>
                      //   </div>
                      // </div>
                    }
                    .into_any()
                  }
                  Err(e) => {
                    #[cfg(not(feature = "ssr"))]
                    loading.set(false);
                    error!("{:#?}", e);
                    view! {
                      <Error error={e} on_retry_click={Some(on_retry_click)} /*on_retry_click={None::<Option<_>>}*/ />
                      // <div class="py-4 px-8 break-inside-avoid">
                      //   <div class="flex justify-between alert alert-error alert-soft">
                      //     <span class="text-lg">{"Error"}</span>
                      //     <span on:click={on_retry_click} class="btn btn-sm">
                      //       "Retry"
                      //     </span>
                      //   </div>
                      // </div>
                    }
                    .into_any()
                  }
                }}
              </For>
              // <Error loading=loading.get() />
              // {move || {
              //   if loading.get() {
              //     Some(
              //       view! {
              //         <div class="overflow-hidden break-inside-avoid animate-[popdown_1s_step-end_1]">
              //           <div class="py-4 px-8">
              //             <div class="alert alert-info alert-soft">
              //               <span>"Loading..."</span>
              //             </div>
              //           </div>
              //         </div>
              //       },
              //     )
              //   } else {
              //     None
              //   }
              // }}
            </Transition>
            <div node_ref={intersection_element} class="block bg-transparent h-[1px]" />
            {move || { view!{ <Loading loading=loading.get() /> } }}
        // </Show>
        // {
        //   // #[cfg(not(feature = "ssr"))]
        //   if ssr_page().len() > 0 {
        //     render_signal.set(true);
        //   }
        // }
          </div>
        </div>
      </main>
    // </Transition>
  }
}
