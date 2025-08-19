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
// use ev::*;
use chrono::prelude::*;
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
use std::{
  collections::{BTreeMap, HashMap},
  usize, vec,
};
#[cfg(not(feature = "ssr"))]
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{js_sys::Atomics::wait_async, Event, MouseEvent, TouchEvent, WheelEvent};

#[component]
pub fn ResponsiveHomeActivity(ssr_site: Resource<Option<bool>, Result<GetSiteResponse, LemmyAppError>>) -> impl IntoView {
  let i18n = use_i18n();
  let error = expect_context::<RwSignal<Vec<Option<(LemmyAppError, Option<RwSignal<bool>>)>>>>();

  let param = use_params_map();
  let query = use_query_map();

  let ssr_name = move || param.get().get("name").cloned().unwrap_or("".into());
  let ssr_list = move || serde_json::from_str::<ListingType>(&query.get().get("list").cloned().unwrap_or("".into())).unwrap_or(ListingType::All);
  let ssr_sort = move || serde_json::from_str::<SortType>(&query.get().get("sort").cloned().unwrap_or("".into())).unwrap_or(SortType::Active);
  let ssr_page = move || serde_json::from_str::<Vec<(usize, String)>>(&query.get().get("page").cloned().unwrap_or("".into())).unwrap_or(vec![]);
  let response_cache = expect_context::<RwSignal<BTreeMap<(usize, String, ListingType, SortType, String), Option<GetPostsResponse>>>>();
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

  #[cfg(not(feature = "ssr"))]
  {
    let on_scroll = move |e: Event| {
      // if !sleep.get() {
      // let iw = window().inner_width().ok().map(|b| b.as_f64().unwrap_or(0.0)).unwrap_or(0.0);

      // if iw < 768f64 {
      // if let Ok(Some(s)) = window().local_storage() {
      //   let mut query_params = query.get();
      //   let _ = s.set_item(
      //     &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
      //     &window().scroll_y().unwrap_or(0.0).to_string(),
      //   );
      //   log!("weee {}", window().scroll_y().unwrap_or(0.0));
      // }
      // } else {
      if let Some(se) = on_scroll_element.get() {
        if let Ok(Some(s)) = window().local_storage() {
          let mut query_params = query.get();
          let _ = s.set_item(
            &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
            &se.scroll_left().to_string(),
          );
          // log!("scrolling {}", se.scroll_left());
        }
      }
      // }
      // }
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
                st.push((key, next_page));
              }
              let mut query_params = query.get();
              query_params.insert("page".into(), serde_json::to_string(&st).unwrap_or("[]".into()));
              let iw = window().inner_width().ok().map(|b| b.as_f64().unwrap_or(0.0)).unwrap_or(0.0);
              if iw < 768f64 {
                if let Ok(Some(s)) = window().local_storage() {
                  // let mut query_params = query.get();
                  let _ = s.set_item(
                    &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
                    &window().scroll_y().unwrap_or(0.0).to_string(),
                  );
                  // log!("shreee {}", window().scroll_y().unwrap_or(0.0));
                }
              } else {
                if let Some(se) = on_scroll_element.get() {
                  if let Ok(Some(s)) = window().local_storage() {
                    // let mut query_params = query.get();
                    let _ = s.set_item(
                      &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
                      &se.scroll_left().to_string(),
                    );
                    // log!("leeftling {}", se.scroll_left());
                  }
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
            } else {
            }
          } else {
          }
        } else {
        }
      },
      UseIntersectionObserverOptions::default(),
    );
  }

  let post_list_resource = Resource::new(
    move || (logged_in.get(), ssr_list(), ssr_sort(), ssr_name(), ssr_page()),
    move |(_logged_in, list, sort, name, mut pages)| async move {
      let mut rc = response_cache.get();
      let mut new_pages: Vec<(usize, String, Option<GetPostsResponse>)> = vec![];

      if pages.len() == 0 {
        #[cfg(not(feature = "ssr"))]
        refresh_base.set_untracked(chrono::Utc::now().timestamp_millis());

        // log!("empty {}", refresh_base.get_untracked());

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
          page_cursor: None,
          show_hidden: Some(true),
          show_nsfw: Some(false),
          show_read: Some(true),
        };
        let result = LemmyClient.list_posts(form.clone()).await;
        match result {
          Ok(o) => {
            new_pages.push((0, format!("{}", refresh_base.get_untracked()), Some(o.clone())));
            response_cache.update(move |rc| {
              rc.insert((0, "".into(), ListingType::All, SortType::Active, "".into()), Some(o));
            });
          }
          Err(e) => {}
        }
      } else {
        for p in pages {
          if let Some(c) = rc.get(&(p.0, p.1.clone(), list, sort, name.clone())) {
            // log!("hit {}", p.0);
            new_pages.push((
              p.0,
              if p.0 == 0usize {
                format!("{}", refresh_base.get_untracked())
              } else {
                p.1.clone()
              },
              c.clone(),
            ));
          } else {
            // log!("miss {}", p.0);
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
            let result = LemmyClient.list_posts(form.clone()).await;
            match result {
              Ok(o) => {
                new_pages.push((
                  p.0,
                  if p.0 == 0usize {
                    format!("{}", refresh_base.get_untracked())
                  } else {
                    p.1.clone()
                  },
                  Some(o.clone()),
                ));
                let moved_name = name.clone();
                response_cache.update(move |rc| {
                  rc.insert((p.0, p.1.clone(), list, sort, moved_name), Some(o));
                });
              }
              Err(e) => {}
            }
          }
        }
      }

      #[cfg(not(feature = "ssr"))]
      set_timeout(
        move || {
          if let Some(se) = on_scroll_element.get() {
            if let Ok(Some(s)) = window().local_storage() {
              let mut query_params = query.get();
              if let Ok(Some(l)) = s.get_item(&format!("{}{}", use_location().pathname.get(), query_params.to_query_string())) {
                se.set_scroll_left(l.parse().unwrap_or(0i32));
                // log!("set {}", l);
              }
            }
            scroll_element.set(Some(on_scroll_element));
          }
        },
        std::time::Duration::new(0, 1_000_000_000),
      );

      (new_pages)
    },
  );

  view! {
    <main class="flex flex-col">
      <ResponsiveTopNav ssr_site />
      <div class="flex flex-grow">
        <div on:wheel=move |e: WheelEvent| {
          if let Some(se) = on_scroll_element.get() {
            se.scroll_by_with_x_and_y(e.delta_y(), 0f64);
          }
        } node_ref=on_scroll_element class={move || {
          format!("md:h-[calc(100%-4rem)] min-w-full md:absolute md:overflow-x-auto md:overflow-y-hidden md:columns-sm md:px-4 gap-4{}", if loading.get() { " opacity-25" } else { "" })
        }}>
          <Transition fallback={|| {}}>
            <Title text="" />
            <For each={move || post_list_resource.get().unwrap_or(vec![])} key={|p| (p.0, p.1.clone())} let:p>
              <ResponsivePostListings posts={p.2.clone().unwrap().posts.into()} ssr_site page_number={p.0.into()} />
              {
                // log!("next {}", p.0 + 50usize);
                next_page_cursor.set((p.0 + 50usize, p.2.unwrap().next_page.clone()));
              }
            </For>
            <div node_ref={intersection_element} class="block bg-transparent h-[1px]" />
          </Transition>
        </div>
      </div>
    </main>
  }
}
