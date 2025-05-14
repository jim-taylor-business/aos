use crate::{
  errors::{message_from_error, LemmyAppError},
  i18n::*,
  lemmy_client::*,
  ui::components::{
    common::{about::About, responsive_nav::ResponsiveTopNav},
    home::{site_summary::SiteSummary, trending::Trending},
    post::{post_listings::PostListings, responsive_post_listings::ResponsivePostListings},
  },
  ResourceStatus, ResponseLoad,
};
use codee::string::FromToStringCodec;
use lemmy_api_common::{
  lemmy_db_schema::{ListingType, SortType},
  lemmy_db_views::structs::PaginationCursor,
  post::{GetPosts, GetPostsResponse},
  site::GetSiteResponse,
};
use leptos::{html::*, logging::log, *};
use leptos_meta::*;
use leptos_router::*;
use leptos_use::*;
#[cfg(not(feature = "ssr"))]
use wasm_bindgen::{prelude::Closure, JsCast};
// use serde::serde_as;
use std::{
  collections::{BTreeMap, HashMap},
  usize, vec,
};
use web_sys::{js_sys::Atomics::wait_async, Event, MouseEvent, WheelEvent};

#[component]
pub fn ResponsiveHomeActivity(ssr_site: Resource<Option<bool>, Result<GetSiteResponse, LemmyAppError>>) -> impl IntoView {
  let i18n = use_i18n();

  let error = expect_context::<RwSignal<Vec<Option<(LemmyAppError, Option<RwSignal<bool>>)>>>>();

  let param = use_params_map();
  let ssr_name = move || param.get().get("name").cloned().unwrap_or("".into());

  let query = use_query_map();

  let ssr_list = move || serde_json::from_str::<ListingType>(&query.get().get("list").cloned().unwrap_or("".into())).unwrap_or(ListingType::All);
  let ssr_sort = move || serde_json::from_str::<SortType>(&query.get().get("sort").cloned().unwrap_or("".into())).unwrap_or(SortType::Active);
  // let ssr_from = move || {
  //   serde_json::from_str::<(usize, Option<PaginationCursor>)>(&query.get().get("from").cloned().unwrap_or("".into())).unwrap_or((0usize, None))
  // };
  // let ssr_prev =
  //   move || serde_json::from_str::<Vec<(usize, Option<PaginationCursor>)>>(&query.get().get("prev").cloned().unwrap_or("".into())).unwrap_or(vec![]);
  let ssr_page =
    move || serde_json::from_str::<Vec<(usize, String)>>(&query.get().get("page").cloned().unwrap_or("".into())).unwrap_or(vec![(0usize, "".into())]);
  // let ssr_limit = move || query.get().get("limit").cloned().unwrap_or("".into()).parse::<usize>().unwrap_or(10usize);

  // let csr_resources = expect_context::<RwSignal<BTreeMap<(usize, ResourceStatus), (Option<PaginationCursor>, Option<GetPostsResponse>)>>>();
  // let csr_next_page_cursor = expect_context::<RwSignal<(usize, Option<PaginationCursor>)>>();
  let response_cache = expect_context::<RwSignal<BTreeMap<(usize, String, ListingType, SortType, String), Option<GetPostsResponse>>>>();
  // let response_load = expect_context::<RwSignal<ResponseLoad>>();
  let next_page_cursor: RwSignal<(usize, Option<PaginationCursor>)> = RwSignal::new((0, None));

  // let on_sort_click = move |s: SortType| {
  //   move |_e: MouseEvent| {
  //     // csr_resources.set(BTreeMap::new());
  //     // csr_next_page_cursor.set((0, None));

  //     let r = serde_json::to_string::<SortType>(&s);
  //     let mut query_params = query.get();
  //     match r {
  //       Ok(o) => {
  //         query_params.insert("sort".into(), o);
  //       }
  //       Err(e) => {
  //         error.update(|es| es.push(Some((e.into(), None))));
  //       }
  //     }
  //     if SortType::Active == s {
  //       query_params.remove("sort".into());
  //     }
  //     query_params.remove("from".into());
  //     query_params.remove("prev".into());
  //     let navigate = leptos_router::use_navigate();
  //     navigate(
  //       &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
  //       Default::default(),
  //     );
  //   }
  // };

  let loading = RwSignal::new(false);
  let refresh = RwSignal::new(false);

  let logged_in = Signal::derive(move || {
    if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site.get() {
      Some(true)
    } else {
      Some(false)
    }
  });

  // let posts_resource = Resource::new(
  //   move || {
  //     (
  //       refresh.get(),
  //       logged_in.get(),
  //       ssr_list(),
  //       ssr_sort(),
  //       ssr_from(),
  //       ssr_limit(),
  //       community_name(),
  //     )
  //   },
  //   move |(_refresh, _logged_in, list_type, sort_type, from, limit, name)| async move {
  //     logging::log!("ssr");
  //     loading.set(true);

  //     let form = GetPosts {
  //       type_: Some(list_type),
  //       sort: Some(sort_type),
  //       community_name: name,
  //       community_id: None,
  //       page: None,
  //       limit: Some(i64::try_from(limit).unwrap_or(10)),
  //       saved_only: None,
  //       disliked_only: None,
  //       liked_only: None,
  //       page_cursor: from.1.clone(),
  //       show_hidden: Some(true),
  //       show_nsfw: Some(false),
  //       show_read: Some(true),
  //     };

  //     let result = LemmyClient.list_posts(form.clone()).await;
  //     loading.set(false);
  //     match result {
  //       Ok(o) => {
  //         #[cfg(not(feature = "ssr"))]
  //         if let Ok(Some(s)) = window().local_storage() {
  //           if let Ok(Some(_)) = s.get_item(&serde_json::to_string(&form).ok().unwrap()) {}
  //           let _ = s.set_item(&serde_json::to_string(&form).ok().unwrap(), &serde_json::to_string(&o).ok().unwrap());
  //         }
  //         Ok((from, o))
  //       }
  //       Err(e) => {
  //         error.update(|es| es.push(Some((e.clone(), None))));
  //         Err((e, Some(refresh)))
  //       }
  //     }
  //   },
  // );

  // let on_csr_filter_click = move |l: ListingType| {
  //   move |_e: MouseEvent| {
  //     let mut query_params = query.get();
  //     // query_params.remove("sort".into());
  //     query_params.remove("from".into());
  //     query_params.remove("prev".into());
  //     let navigate = leptos_router::use_navigate();
  //     if l == ListingType::All {
  //       query_params.remove("list".into());
  //     } else {
  //       query_params.insert("list".into(), serde_json::to_string(&l).ok().unwrap());
  //     }
  //     navigate(
  //       &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
  //       Default::default(),
  //     );
  //   }
  // };

  // let highlight_csr_filter = move |l: ListingType| {
  //   if l == ssr_list() {
  //     "btn-active"
  //   } else {
  //     ""
  //   }
  // };

  let intersection_element = create_node_ref::<Div>();

  let on_scroll_element = NodeRef::<Div>::new();

  #[cfg(not(feature = "ssr"))]
  let (get_scroll_cookie, set_scroll_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
    "scroll",
    UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax),
  );

  #[cfg(not(feature = "ssr"))]
  {
    // if let Some(se) = on_scroll_element.get() {
    //   // log!("scrolling {}", se.scroll_left());
    //   if let Some(s) = get_scroll_cookie.get() {
    //     log!("set");
    //     se.set_scroll_left(s.parse().unwrap_or(0i32));
    //   } else {
    //     log!("ignore");
    //   }
    //   // se.set_scroll_left(se.scroll_left() + e.delta_y() as i32);
    // } else {
    //   log!("ignore");
    // }

    let on_scroll = move |e: Event| {
      if let Some(se) = on_scroll_element.get() {
        set_scroll_cookie.set(Some(se.scroll_left().to_string()));
        // log!("scrolling {}", se.scroll_left());
        // se.set_scroll_left(se.scroll_left() + e.delta_y() as i32);
        // } else {
        //   log!("ignore");
      }
    };

    let UseScrollReturn {
      x,
      y,
      set_x,
      set_y,
      is_scrolling,
      arrived_state,
      directions,
      ..
    } = use_scroll_with_options(on_scroll_element, UseScrollOptions::default().on_scroll(on_scroll));

    // let on_online = move |b: bool| {
    //   move |_| {
    //     online.set(OnlineSetter(b));
    //   }
    // };

    // let _offline_handle = window_event_listener_untyped("offline", on_online(false));
    // let _online_handle = window_event_listener_untyped("online", on_online(true));
    //
    // canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
    //
    // set_timeout(
    //   move || {
    //     if let Some(se) = on_scroll_element.get() {
    //       // log!("scrolling {}", se.scroll_left());
    //       if let Some(s) = get_scroll_cookie.get() {
    //         log!("set");
    //         se.set_scroll_left(s.parse().unwrap_or(0i32));
    //       } else {
    //         log!("ignore");
    //       }
    //       // se.set_scroll_left(se.scroll_left() + e.delta_y() as i32);
    //     } else {
    //       log!("ignore");
    //     }
    //   },
    //   std::time::Duration::new(0, 500_000_000),
    // );
    //
    // pub fn window_event_listener_untyped(
    //     event_name: &str,
    //     cb: impl Fn(web_sys::Event) + 'static,
    // ) -> WindowListenerHandle {
    //     cfg_if::cfg_if! {
    //       if #[cfg(debug_assertions)] {
    //         let span = ::tracing::Span::current();
    //         let cb = move |e| {
    //           let prev = leptos_reactive::SpecialNonReactiveZone::enter();
    //           let _guard = span.enter();
    //           cb(e);
    //           leptos_reactive::SpecialNonReactiveZone::exit(prev);
    //         };
    //       }
    //     }

    //     if !is_server() {
    //         #[inline(never)]
    //         fn wel(
    //             cb: Box<dyn FnMut(web_sys::Event)>,
    //             event_name: &str,
    //         ) -> WindowListenerHandle {
    //             let cb = Closure::wrap(cb).into_js_value();
    //             _ = window().add_event_listener_with_callback(
    //                 event_name,
    //                 cb.unchecked_ref(),
    //             );
    //             let event_name = event_name.to_string();
    //             WindowListenerHandle(Box::new(move || {
    //                 _ = window().remove_event_listener_with_callback(
    //                     &event_name,
    //                     cb.unchecked_ref(),
    //                 );
    //             }))
    //         }

    //         wel(Box::new(cb), event_name)
    //     } else {
    //         WindowListenerHandle(Box::new(|| ()))
    //     }
    // }

    // let on_rsc = move |e: Event| {
    //   log!("state! {:#?}", e);
    // };
    // let _handle = window_event_listener_untyped("offline", on_rsc);
    // documen
    // let _handle = window_event_listener_untyped("readystatechange", on_rsc);
    // let _handle = window_event_listener_untyped("load", on_rsc);

    let on_rsc = move |e: Event| {
      log!("rsc state! {:#?}", e);
    };
    // // let _handle = window_event_listener_untyped("readystatechange", on_rsc);
    let _handle = window_event_listener_untyped("load", on_rsc);

    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
      log!("load state! {:#?}", event);
    });
    // if let Some(d) = window().document() {
    window()
      // .document()
      // .unwrap()
      .add_event_listener_with_callback("load", closure.as_ref().unchecked_ref());
    // } else {
    //   log!("NOOO state!");
    // }
    //   .unwrap()

    // window()
    //   .document().unwrap().ready_state()

    closure.forget();

    // .onreadystatechange() {//.readyState == 'complete' {
    //     highlighter();
    // } else {
    //     document.onreadystatechange = function () {
    //         if (document.readyState === "complete") {
    //             highlighter();
    //         }
    //     }
    // }

    let UseIntersectionObserverReturn {
      pause,
      resume,
      stop,
      is_active,
    } = use_intersection_observer_with_options(
      intersection_element,
      move |intersections, _| {
        // if let Some(e) = response_cache.get().last_entry() {
        //   logging::log!("trigger");

        //   // logging::log!("{:#?}", e.key());
        //   // logging::log!("{:#?}", e.get().as_ref().unwrap().next_page);
        //   let mut st = ssr_page();
        //   if let Some(PaginationCursor(next_page)) = e.get().as_ref().unwrap().next_page.clone() {
        //     // let next_page = (e.key().0 + 50, e.get().as_ref().unwrap().next_page.clone());
        //     // let next_page = Some((e.key().0 + 50, e.get().as_ref().unwrap().next_page.clone()));
        //     // let next_page = e.get().as_ref().unwrap().next_page.clone();

        //     // let mut st = ssr_prev();
        //     st.push((e.key().0 + 50, next_page));
        //   }
        //   let mut query_params = query.get();
        //   query_params.insert("page".into(), serde_json::to_string(&st).unwrap_or("[]".into()));
        //   // query_params.insert("prev".into(), serde_json::to_string(&st).unwrap_or("[]".into()));
        //   // query_params.insert("from".into(), serde_json::to_string(&next_page).unwrap_or("[0,None]".into()));

        //   let navigate = leptos_router::use_navigate();
        //   navigate(
        //     &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
        //     NavigateOptions {
        //       resolve: true,
        //       replace: false,
        //       scroll: false,
        //       state: State::default(),
        //     },
        //   );
        // } else {
        //   logging::log!("trigger ignore");
        // }
        //
        // log!("{:#?}", intersections[0].is_intersecting());

        if intersections[0].is_intersecting() {
          if let (key, _) = next_page_cursor.get() {
            if key > 0 {
              // log!("trigger {}", key);

              let mut st = ssr_page();
              if let (_, Some(PaginationCursor(next_page))) = next_page_cursor.get() {
                st.push((key, next_page));
              }
              let mut query_params = query.get();
              query_params.insert("page".into(), serde_json::to_string(&st).unwrap_or("[]".into()));

              let navigate = leptos_router::use_navigate();
              navigate(
                &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
                NavigateOptions {
                  resolve: true,
                  replace: false,
                  scroll: false,
                  state: State::default(),
                },
              );
            } else {
              // log!("trigger ignore");
            }
          } else {
            // log!("trigger ignore");
          }
        } else {
          // log!("trigger ignore");
        }

        // let iw = window().inner_width().ok().map(|b| b.as_f64().unwrap_or(0.0)).unwrap_or(0.0);

        // if iw < 640f64 {
        // logging::log!("inter");

        // if csr_next_page_cursor.get().1.is_some()
        //   && csr_resources
        //     .get()
        //     .get(&(csr_next_page_cursor.get().0, ResourceStatus::Loading))
        //     .is_none()
        //   && csr_resources.get().get(&(csr_next_page_cursor.get().0, ResourceStatus::Ok)).is_none()
        //   && csr_resources.get().get(&(csr_next_page_cursor.get().0, ResourceStatus::Err)).is_none()
        // {
        //   csr_resources.update(|h| {
        //     h.insert(
        //       (csr_next_page_cursor.get().0, ResourceStatus::Loading),
        //       (csr_next_page_cursor.get().1, None),
        //     );
        //   });

        //   let _csr_resource = create_local_resource(
        //     move || (),
        //     move |()| async move {
        //       let from = csr_next_page_cursor.get();

        //       let form = GetPosts {
        //         type_: Some(ssr_list()),
        //         sort: Some(ssr_sort()),
        //         community_name: community_name(),
        //         community_id: None,
        //         page: None,
        //         limit: Some(50),
        //         saved_only: None,
        //         disliked_only: None,
        //         liked_only: None,
        //         page_cursor: from.1.clone(),
        //         show_hidden: Some(true),
        //         show_nsfw: Some(false),
        //         show_read: Some(true),
        //       };

        //       let result = LemmyClient.list_posts(form).await;

        //       match result {
        //         Ok(o) => {
        //           csr_next_page_cursor.set((from.0 + 50, o.next_page.clone()));
        //           csr_resources.update(move |h| {
        //             h.remove(&(from.0, ResourceStatus::Loading));
        //             h.insert((from.0, ResourceStatus::Ok), (from.1.clone(), Some(o.clone())));
        //           });
        //           Some(())
        //         }
        //         Err(e) => {
        //           csr_resources.update(move |h| {
        //             h.remove(&(from.0, ResourceStatus::Loading));
        //             h.insert((from.0, ResourceStatus::Err), (from.1, None));
        //           });
        //           error.update(|es| es.push(Some((e, Some(refresh)))));
        //           None
        //         }
        //       }
        //     },
        //   );
        // }

        // }
      },
      UseIntersectionObserverOptions::default(),
    );

    // pause();
    // resume();

    // let _a_effect = Effect::new(move |_| match is_active.get() {
    //   true => {
    //     logging::log!("a");
    //   }
    //   _ => {
    //     logging::log!("n");
    //   }
    // });
  }

  // let on_retry_click = move |i: (usize, ResourceStatus)| {
  //   move |_e: MouseEvent| {
  //     let _csr_resource = create_local_resource(
  //       move || (),
  //       move |()| //{
  //       async move {
  //         let from = csr_resources.get().get(&i).unwrap().0.clone();
  //         let form = GetPosts {
  //           type_: Some(ssr_list()),
  //           sort: Some(ssr_sort()),
  //           community_name: community_name(),
  //           community_id: None,
  //           page: None,
  //           limit: Some(10),
  //           saved_only: None,
  //           disliked_only: None,
  //           liked_only: None,
  //           page_cursor: from.clone(),
  //           show_hidden: Some(true),
  //           show_nsfw: Some(false),
  //           show_read: Some(true),
  //         };

  //         let from_clone = from.clone();
  //         csr_resources.update(move |h| {
  //           h.remove(&(i.0, ResourceStatus::Err));
  //           h.insert((i.0, ResourceStatus::Loading), (from_clone, None));
  //         });

  //         let result = LemmyClient.list_posts(form).await;

  //         match result {
  //           Ok(o) => {
  //             csr_next_page_cursor.set((i.0 + ssr_limit(), o.next_page.clone()));
  //             csr_resources.update(move |h| {
  //               h.remove(&(i.0, ResourceStatus::Loading));
  //               h.insert((i.0, ResourceStatus::Ok), (from, Some(o.clone())));
  //             });
  //             Some(())
  //           }
  //           Err(e) => {
  //             csr_resources.update(move |h| {
  //               h.remove(&(i.0, ResourceStatus::Loading));
  //               h.insert((i.0, ResourceStatus::Err), (from, None));
  //             });
  //             error.update(|es| es.push(Some((e, None))));
  //             None
  //           }
  //         }
  //       },
  //     );
  //   }
  // };

  let responsive_cache_resourcs = Resource::new(
    move || (refresh.get(), logged_in.get(), ssr_list(), ssr_sort(), ssr_name(), ssr_page()),
    move |(_refresh, _logged_in, list, sort, name, pages)| async move {
      let mut rc = response_cache.get();
      let mut new_pages: HashMap<usize, Option<GetPostsResponse>> = HashMap::new();

      let pages_later = pages.clone();

      log!("keys {:#?}", rc.keys());

      for p in pages {
        if rc.get(&(p.0, p.1.clone(), list, sort, name.clone())).is_none() {
          //} && new_pages.get(&p.1.clone()).is_none() {
          // // if rc.get(&(p.0, p.1.clone(), list, sort, name.clone())).is_none() && new_pages.get(&(p.0, p.1.clone(), list, sort, name.clone())).is_none() {
          let form = GetPosts {
            type_: Some(list),
            sort: Some(sort),
            // community_name: name.clone(),
            community_name: if name.clone().len() == 0usize { None } else { Some(name.clone()) },
            community_id: None,
            page: None,
            limit: Some(50),
            saved_only: None,
            disliked_only: None,
            liked_only: None,
            // page_cursor: Some(p.1.clone()),
            page_cursor: if p.0 == 0usize { None } else { Some(PaginationCursor(p.1.clone())) },
            show_hidden: Some(true),
            show_nsfw: Some(false),
            show_read: Some(true),
          };
          let result = LemmyClient.list_posts(form.clone()).await;
          match result {
            Ok(o) => {
              logging::log!("load");
              // new_pages.insert((p.0, p.1, list, sort, name.clone()), Some(o));
              new_pages.insert(p.0, Some(o));
              // rc.insert(p, Some(o));
            }
            Err(e) => {}
          }
        } else {
          logging::log!("ignore");
        }
      }

      // response_load.set(ResponseLoad(true));
      //
      #[cfg(not(feature = "ssr"))]
      set_timeout(
        move || {
          if let Some(se) = on_scroll_element.get() {
            // log!("scrolling {}", se.scroll_left());
            if let Some(s) = get_scroll_cookie.get() {
              log!("set");
              se.set_scroll_left(s.parse().unwrap_or(0i32));
            } else {
              log!("ignore");
            }
            // se.set_scroll_left(se.scroll_left() + e.delta_y() as i32);
          } else {
            log!("ignore");
          }
        },
        std::time::Duration::new(0, 500_000_000),
      );

      (new_pages, pages_later, list, sort, name)

      // let mut counter = if let Some(e) = rc.last_entry() {
      //   // let mut counter = if let Some(e) = response_cache.get().last_entry() {
      //   logging::log!("render {}", e.key().0 + 50usize);
      //   e.key().0 + 50usize
      // } else {
      //   logging::log!("render empty");
      //   0usize
      // };

      // rc.retain(|p, q| pages_later.contains(&p));

      // for n in new_pages {
      //   // rc.update(move |rc| {
      //   if rc.get(&(counter, n.0.clone())).is_none() {
      //     logging::log!("add");
      //     rc.insert((counter, n.0), n.1);
      //   }
      //   // });
      //   counter = counter + 50usize;
      // }

      // (rc)

      // if rc.get(&from.clone()).is_some() {
      //   Ok((from, None, rc))
      // } else {
      //   let form = GetPosts {
      //     type_: Some(list_type),
      //     sort: Some(sort_type),
      //     community_name: name,
      //     community_id: None,
      //     page: None,
      //     limit: Some(50),
      //     saved_only: None,
      //     disliked_only: None,
      //     liked_only: None,
      //     page_cursor: from.1.clone(),
      //     show_hidden: Some(true),
      //     show_nsfw: Some(false),
      //     show_read: Some(true),
      //   };
      //   let result = LemmyClient.list_posts(form.clone()).await;
      //   match result {
      //     Ok(o) => Ok((from, Some(o), rc)),
      //     Err(e) => {
      //       error.update(|es| es.push(Some((e.clone(), None))));
      //       Err((e, Some(refresh)))
      //     }
      //   }
      // }
    },
  );

  // let responsive_cache_resourcs = Resource::new(
  //   move || (ssr_list(), ssr_sort(), ssr_name(), ssr_page()),
  //   move |(list, sort, name, pages)| async move {
  //     let mut new_pages: RwSignal<BTreeMap<usize, Option<GetPostsResponse>>> = RwSignal::new(BTreeMap::new());
  //     let mut counter = 0usize;
  //     for p in pages {
  //       let form = GetPosts {
  //         type_: Some(list),
  //         sort: Some(sort),
  //         community_name: if name.clone().len() == 0usize { None } else { Some(name.clone()) },
  //         community_id: None,
  //         page: None,
  //         limit: Some(50),
  //         saved_only: None,
  //         disliked_only: None,
  //         liked_only: None,
  //         page_cursor: if p.0 == 0usize { None } else { Some(PaginationCursor(p.1.clone())) },
  //         show_hidden: Some(true),
  //         show_nsfw: Some(false),
  //         show_read: Some(true),
  //       };
  //       let result = LemmyClient.list_posts(form.clone()).await;
  //       match result {
  //         Ok(o) => {
  //           new_pages.update(|np| {
  //             np.insert(counter, Some(o));
  //           });
  //           counter = counter + 50usize;
  //         }
  //         Err(e) => {}
  //       }
  //     }
  //     new_pages
  //   },
  // );

  view! {
  <main class="flex flex-col">
    <ResponsiveTopNav ssr_site />

    // <div class="flex flex-shrink">
    //   <div class="hidden mr-3 sm:inline-block join">
    //     <button class="btn join-item btn-active">"Posts"</button>
    //     <button class="btn join-item btn-disabled">"Comments"</button>
    //   </div>
    //   <div class="hidden mr-3 sm:inline-block join">
    //     <A
    //       href={move || {
    //         let mut query_params = query.get();
    //         query_params.insert("list".into(), serde_json::to_string(&ListingType::Subscribed).ok().unwrap());
    //         query_params.remove("from".into());
    //         query_params.remove("prev".into());
    //         format!("{}{}", use_location().pathname.get(), query_params.to_query_string())
    //       }}
    //       class={move || {
    //         format!(
    //           "btn join-item{}{}",
    //           if ListingType::Subscribed == ssr_list() { " btn-active" } else { "" },
    //           if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site.get() { "" } else { " btn-disabled" },
    //         )
    //       }}
    //     >
    //       "Subscribed"
    //     </A>
    //     <A
    //       href={move || {
    //         let mut query_params = query.get();
    //         query_params.insert("list".into(), serde_json::to_string(&ListingType::Local).ok().unwrap());
    //         query_params.remove("from".into());
    //         query_params.remove("prev".into());
    //         format!("{}{}", use_location().pathname.get(), query_params.to_query_string())
    //       }}
    //       class={move || format!("btn join-item{}", if ListingType::Local == ssr_list() { " btn-active" } else { "" })}
    //     >
    //       "Local"
    //     </A>
    //     <A
    //       href={move || {
    //         let mut query_params = query.get();
    //         query_params.remove("list".into());
    //         query_params.remove("from".into());
    //         query_params.remove("prev".into());
    //         format!("{}{}", use_location().pathname.get(), query_params.to_query_string())
    //       }}
    //       class={move || format!("btn join-item{}", if ListingType::All == ssr_list() { " btn-active" } else { "" })}
    //     >
    //       "All"
    //     </A>
    //   </div>
    //   <div class="ml-3 sm:inline-block sm:ml-0 dropdown">
    //     <label tabindex="0" class="btn">
    //       "Sort"
    //     </label>
    //     <ul tabindex="0" class="shadow menu dropdown-content z-[1] bg-base-100 rounded-box">
    //       <li
    //         class={move || { (if SortType::Active == ssr_sort() { "btn-active" } else { "" }).to_string() }}
    //         on:click={on_sort_click(SortType::Active)}
    //       >
    //         <span>{t!(i18n, active)}</span>
    //       </li>
    //       <li class={move || { (if SortType::Hot == ssr_sort() { "btn-active" } else { "" }).to_string() }} on:click={on_sort_click(SortType::Hot)}>
    //         <span>{t!(i18n, hot)}</span>
    //       </li>
    //       <li
    //         class={move || { (if SortType::Scaled == ssr_sort() { "btn-active" } else { "" }).to_string() }}
    //         on:click={on_sort_click(SortType::Scaled)}
    //       >
    //         <span>{"Scaled"}</span>
    //       </li>
    //       <li class={move || { (if SortType::New == ssr_sort() { "btn-active" } else { "" }).to_string() }} on:click={on_sort_click(SortType::New)}>
    //         <span>{t!(i18n, new)}</span>
    //       </li>
    //     </ul>
    //   </div>
    //   <div class="inline-block ml-3 sm:hidden sm:ml-0 dropdown">
    //     <label tabindex="0" class="btn">
    //       "List"
    //     </label>
    //     <ul tabindex="0" class="shadow menu dropdown-content z-[1] bg-base-100 rounded-box">
    //       <li class={move || highlight_csr_filter(ListingType::Subscribed)} on:click={on_csr_filter_click(ListingType::Subscribed)}>
    //         <span>"Subscribed"</span>
    //       </li>
    //       <li class={move || highlight_csr_filter(ListingType::All)} on:click={on_csr_filter_click(ListingType::All)}>
    //         <span>"All"</span>
    //       </li>
    //     </ul>
    //   </div>
    // </div>




    // <main class="flex flex-col flex-grow w-full sm:flex-row">

    // <main class="">
      // <div class="relative w-full sm:pr-4 lg:w-2/3 2xl:w-3/4 3xl:w-4/5 4xl:w-5/6">
    // <script>
    // document.addEventListener("readystatechange", (event) => {
    //   console.log(document.readyState);
    // });
    // </script>
    <div class="flex flex-grow">
      <div on:wheel=move |e: WheelEvent| {
        if let Some(se) = on_scroll_element.get() {
          se.set_scroll_left(se.scroll_left() + e.delta_y() as i32);
        }
      } node_ref=on_scroll_element class={move || {
        format!("sm:h-[calc(100%-6rem)] min-w-full absolute sm:overflow-x-auto sm:overflow-y-hidden sm:columns-sm px-4 gap-4{}", if loading.get() { " opacity-25" } else { "" })
        // format!("sm:container sm:h-[calc(100%-12rem)] absolute sm:overflow-x-auto sm:overflow-y-hidden sm:columns-[50ch] gap-0{}", if loading.get() { " opacity-25" } else { "" })
      }}>

        // <Transition fallback={|| {}}>
        //   {move || {
        //     match posts_resource.get() {
        //       Some(Err(err)) => {
        //         view! {
        //           <Title text="Error loading post list" />
        //           <div class="py-4 px-8">
        //             <div class="flex justify-between alert alert-error">
        //               <span>{message_from_error(&err.0)} " - " {err.0.content}</span>
        //               <div>
        //                 <Show when={move || { if let Some(_) = err.1 { true } else { false } }} fallback={|| {}}>
        //                   <button
        //                     on:click={move |_| {
        //                       if let Some(r) = err.1 {
        //                         r.set(!r.get());
        //                       } else {}
        //                     }}
        //                     class="btn btn-sm"
        //                   >
        //                     "Retry"
        //                   </button>
        //                 </Show>
        //               </div>
        //             </div>
        //           </div>
        //         }
        //       }
        //       Some(Ok(posts)) => {
        //         let next_page = Some((posts.0.0 + ssr_limit(), posts.1.next_page.clone()));
        //         // csr_next_page_cursor.set(next_page.clone().unwrap());
        //         view! {
        //           // <Title text={format!("Page {}", 1 + (ssr_from().0 / ssr_limit()))} />
        //           <Title text="" />
        //           <ResponsivePostListings posts={posts.1.posts.into()} ssr_site page_number={posts.0.0.into()} />
        //           // <div class="hidden sm:block join">
        //           //   {
        //           //     let mut st = ssr_prev();
        //           //     let p = st.pop();
        //           //     let mut query_params = query.get();
        //           //     if st.len() > 0 {
        //           //       query_params.insert("prev".into(), serde_json::to_string(&st).unwrap_or("[]".into()));
        //           //     } else {
        //           //       query_params.remove("prev".into());
        //           //     }
        //           //     if p.ne(&Some((0, None))) {
        //           //       query_params.insert("from".into(), serde_json::to_string(&p).unwrap_or("[0,None]".into()));
        //           //     } else {
        //           //       query_params.remove("from".into());
        //           //     }
        //           //     view! {
        //           //       <A
        //           //         on:click={move |_| {
        //           //           loading.set(true);
        //           //         }}
        //           //         href={format!("{}{}", use_location().pathname.get(), query_params.to_query_string())}
        //           //         class={move || format!("btn join-item{}", if !ssr_prev().is_empty() { "" } else { " btn-disabled" })}
        //           //       >
        //           //         "Prev"
        //           //       </A>
        //           //     }
        //           //   }
        //           //   {
        //           //     let mut st = ssr_prev();
        //           //     st.push(ssr_from());
        //           //     let mut query_params = query.get();
        //           //     query_params.insert("prev".into(), serde_json::to_string(&st).unwrap_or("[]".into()));
        //           //     query_params.insert("from".into(), serde_json::to_string(&next_page).unwrap_or("[0,None]".into()));
        //           //     view! {
        //           //       <A
        //           //         on:click={move |_| {
        //           //           loading.set(true);
        //           //         }}
        //           //         href={format!("{}{}", use_location().pathname.get(), query_params.to_query_string())}
        //           //         class={move || {
        //           //           format!(
        //           //             "btn join-item{}{}",
        //           //             if next_page.clone().unwrap_or((0, None)).1.is_some() && !loading.get() { "" } else { " btn-disabled" },
        //           //             if loading.get() { " btn-disabled" } else { "" },
        //           //           )
        //           //         }}
        //           //       >
        //           //         "Next"
        //           //       </A>
        //           //     }
        //           //   }
        //           // </div>
        //         }
        //       }
        //       None => {
        //         view! {
        //           <Title text="Loading post list" />
        //           {loading
        //             .get()
        //             .then(move || {
        //               view! {
        //                 <div class="overflow-hidden animate-[popdown_1s_step-end_1]">
        //                   <div class="py-4 px-8">
        //                     <div class="alert">
        //                       <span>"Loading"</span>
        //                     </div>
        //                   </div>
        //                 </div>
        //               }
        //             })}
        //           <div class="hidden" />
        //         }
        //       }
        //     }
        //   }}
        // </Transition>

        // <For each={move || csr_resources.get()} key={|r| r.0.clone()} let:r>
        //   {
        //     let r_copy = r.clone();
        //     view! {
        //       <Title text="" />
        //       <Show
        //         when={move || r.0.1 == ResourceStatus::Ok}
        //         fallback={move || {
        //           match r_copy.0.1 {
        //             ResourceStatus::Err => {
        //               view! {
        //                 <div class="py-4 px-8">
        //                   <div class="flex justify-between alert alert-error">
        //                     <span class="text-lg">"Error"</span>
        //                     // <span on:click={on_retry_click(r_copy.0)} class="btn btn-sm">
        //                     //   "Retry"
        //                     // </span>
        //                   </div>
        //                 </div>
        //               }
        //             }
        //             _rs => {
        //               view! {
        //                 <div class="overflow-hidden animate-[popdown_1s_step-end_1]">
        //                   <div class="py-4 px-8">
        //                     <div class="alert">
        //                       <span>"Loading..."</span>
        //                     </div>
        //                   </div>
        //                 </div>
        //               }
        //             }
        //           }
        //         }}
        //       >
        //         <ResponsivePostListings posts={r.1.clone().1.unwrap().posts.into()} ssr_site page_number={r.0.0.into()} />
        //       </Show>
        //     }
        //   }
        // </For>
        <Transition fallback={|| {}}>
          {move || {
            match responsive_cache_resourcs.get() {
              Some(mut o) => {
                // if let Some(e) = o.get().last_entry() {
                //   // log!("next {} ", e.key());
                //   next_page_cursor.set((e.key() + 50usize, e.get().as_ref().unwrap().next_page.clone()));
                // }


                // let ohye = o.clone();
                // let woop = o.clone();
                // let goop = o.clone();

                // if let Some(x) = ohye.1 {
                //
                //
                // response_cache.set(o.1);


                // response_cache.update(move |rc| {
                //   // rc.retain(|p, q| o.2.contains(&p.1));
                //   rc.retain(|p, q| o.2.contains(&(p.0, p.1.clone())) && p.2.eq(&o.3) && p.3.eq(&o.4) && p.4.eq(&o.5.clone()));
                // // });

                // let mut counter = if let Some(e) = o.1.last_entry() {
                // // let mut counter = if let Some(e) = response_cache.get().last_entry() {
                //   logging::log!("counter {}", e.key().0 + 50usize);
                //   e.key().0 + 50usize
                // } else {
                //   logging::log!("counter empty");
                //   0usize
                // };

                response_cache.update(move |rc| {
                  // rc.clear();
                  rc.retain(|t, u| o.1.contains(&(t.0, t.1.clone())) && t.2.eq(&o.2) && t.3.eq(&o.3) && t.4.eq(&o.4.clone()));
                  let mut counter = 0usize;
                  // rc.retain(|p, q| o.2.contains(&p.1));
                  for n in o.1 {
                    if rc.get(&(n.0, n.1.clone(), o.2, o.3, o.4.clone())).is_none() {
                      logging::log!("add");
                      if let Some(q) = o.0.remove(&n.0) {
                        rc.insert((n.0, n.1.clone(), o.2, o.3, o.4.clone()), q);
                      }
                    }
                    counter = counter + 1usize;
                  }
                  if let Some(e) = rc.last_entry() {
                    next_page_cursor.set((e.key().0 + 50usize, e.get().as_ref().unwrap().next_page.clone()));
                  }

                  log!("keys ater {:#?}", rc.keys());

                });





                // counter = counter + 50usize;

                // #[cfg(not(feature = "ssr"))]
                // if let Some(se) = on_scroll_element.get() {
                //   // log!("scrolling {}", se.scroll_left());
                //   if let Some(s) = get_scroll_cookie.get() {
                //     log!("set");
                //     se.set_scroll_left(s.parse().unwrap_or(0i32));
                //   } else {
                //     log!("ignore");
                //   }
                //   // se.set_scroll_left(se.scroll_left() + e.delta_y() as i32);
                // } else {
                //   log!("ignore");
                // }


                // } else {
                //   view! {
                //     // <For each={move || lot_resources.get()} key={|r| r.0 /* .1.0*/} let:r>
                //     // </For>
                //     // <span> "Ok" </span>
                //     <div class="hidden sm:block join">
                //     </div>
                //     <div class={move || {
                //       format!("sm:block columns-1 2xl:columns-2 3xl:columns-3 4xl:columns-4 gap-0{}", if loading.get() { " opacity-25" } else { "" })
                //     }}>
                //       // <PostListings posts={ohye.1.posts.into()} ssr_site page_number={ohye.0.0.into()} on_community_change={move |s| {}} />
                //     </div>

                //   }
                // }

                // let next_page = Some((goop.0.0 + ssr_limit(), goop.1.unwrap().next_page.clone()));
                view! {
                  // <div class="hidden sm:block join">
                  //   {
                  //     let mut st = ssr_prev();
                  //     let p = st.pop();
                  //     let mut query_params = query.get();
                  //     if st.len() > 0 {
                  //       query_params.insert("prev".into(), serde_json::to_string(&st).unwrap_or("[]".into()));
                  //     } else {
                  //       query_params.remove("prev".into());
                  //     }
                  //     if p.ne(&Some((0, None))) {
                  //       query_params.insert("from".into(), serde_json::to_string(&p).unwrap_or("[0,None]".into()));
                  //     } else {
                  //       query_params.remove("from".into());
                  //     }
                  //     view! {
                  //       <A
                  //         // on:click={move |_| {
                  //         //   loading.set(true);
                  //         // }}
                  //         href={format!("{}{}", use_location().pathname.get(), query_params.to_query_string())}
                  //         class={move || format!("btn join-item{}", if !ssr_prev().is_empty() { "" } else { " btn-disabled" })}
                  //       >
                  //         "Prev"
                  //       </A>
                  //     }
                  //   }
                  //   {
                  //     let mut st = ssr_prev();
                  //     st.push(ssr_from());
                  //     let mut query_params = query.get();
                  //     query_params.insert("prev".into(), serde_json::to_string(&st).unwrap_or("[]".into()));
                  //     query_params.insert("from".into(), serde_json::to_string(&next_page).unwrap_or("[0,None]".into()));
                  //     view! {
                  //       <A
                  //         // on:click={move |_| {
                  //         //   loading.set(true);
                  //         // }}
                  //         href={format!("{}{}", use_location().pathname.get(), query_params.to_query_string())}
                  //         class={move || {
                  //           format!(
                  //             "btn join-item{}{}",
                  //             if next_page.clone().unwrap_or((0, None)).1.is_some() && !loading.get() { "" } else { " btn-disabled" },
                  //             if loading.get() { " btn-disabled" } else { "" },
                  //           )
                  //         }}
                  //       >
                  //         "Next"
                  //       </A>
                  //     }
                  //   }
                  // </div>
                  // <div class={move || {
                  //   format!("sm:block columns-1 2xl:columns-2 3xl:columns-3 4xl:columns-4 gap-0{}", if loading.get() { " opacity-25" } else { "" })
                  // }}>

                  <div>
                    <For each={move || response_cache.get()} key={|r| r.0.clone()} let:r>
                      <ResponsivePostListings posts={r.1.unwrap().posts.into()} ssr_site page_number={r.0.0.into()} />
                    </For>


                    // <For each={move || o.get()} key={|r| r.0} let:r>
                    //   <ResponsivePostListings posts={r.1.unwrap().posts.into()} ssr_site page_number={r.0.into()} />
                    // </For>

                  </div>

                }


              }
              _ => {
                view! {
                  <div>
                    <Title text="" />
                    // {loading
                    //   .get()
                    //   .then(move || {
                    //     view! {
                          <div class="overflow-hidden animate-[popdown_1s_step-end_1]">
                            <div class="py-4 px-8">
                              <div class="alert">
                                <span>"Loading"</span>
                              </div>
                            </div>
                          </div>
                    //     }
                    // })}
                  // <div class="hidden" />
                  </div>
                }

                // view! {
                //   // <div class="hidden sm:block join">
                //   // </div>
                //   <div>
                //     <span> "None" </span>
                //   </div>
                // }
              }
            }
          }}
          <div node_ref={intersection_element} class="block bg-transparent h-[1px]" />
          // <div node_ref={intersection_element} class=move || format!("{} bg-white h-[5px]", if response_load.get().eq(&ResponseLoad(true)) { "block"} else { "hidden" }) />
        </Transition>
        // <Transition fallback={|| {}}>
        //   <For each={move || responsive_cache_resourcs.get().unwrap()} key={|r| r.0} let:r>
        //     <ResponsivePostListings posts={r.1.unwrap().posts.into()} ssr_site page_number={r.0.into()} />
        //   </For>
        //   // <div node_ref={_scroll_element} class=move || format!("{} bg-white h-[5px]", if response_load.get().eq(&ResponseLoad(true)) { "block"} else { "hidden" }) />
        // </Transition>
        // <div node_ref={_scroll_element} class="block bg-white h-[5px]" />

      // </div>
      // // <div class="hidden lg:block lg:w-1/3 2xl:w-1/4 3xl:w-1/5 4xl:w-1/6">
      // //   <About />
      // //   <SiteSummary ssr_site />
      // //   <Trending />
      </div>
    // </div>
    // <div class="flex flex-shrink">
    // "ohye"
    </div>
  </main>
  }
}
