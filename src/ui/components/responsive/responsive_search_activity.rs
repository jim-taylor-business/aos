use crate::{
  errors::{message_from_error, LemmyAppError},
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
use lemmy_api_common::{
  lemmy_db_schema::{ListingType, SearchType, SortType},
  lemmy_db_views::structs::PaginationCursor,
  post::{GetPosts, GetPostsResponse},
  site::{GetSiteResponse, Search, SearchResponse},
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
use web_sys::{js_sys::Atomics::wait_async, Event, MouseEvent, WheelEvent};

#[component]
pub fn ResponsiveSearchActivity(ssr_site: Resource<(Option<String>, Option<String>), Result<GetSiteResponse, LemmyAppError>>) -> impl IntoView {
  let i18n = use_i18n();

  let param = use_params_map();
  let ssr_name = move || param.get().get("name").cloned().unwrap_or("".into());

  let query = use_query_map();

  let ssr_list = move || serde_json::from_str::<ListingType>(&query.get().get("list").cloned().unwrap_or("".into())).unwrap_or(ListingType::All);
  let ssr_sort = move || serde_json::from_str::<SortType>(&query.get().get("sort").cloned().unwrap_or("".into())).unwrap_or(SortType::Active);
  let ssr_page = move || serde_json::from_str::<Vec<usize>>(&query.get().get("page").cloned().unwrap_or("".into())).unwrap_or(vec![0usize]);
  let ssr_term = move || query.get().get("term").cloned().unwrap_or("".into());

  let next_page_cursor: RwSignal<usize> = RwSignal::new(0);

  let loading = RwSignal::new(false);
  let refresh = RwSignal::new(false);

  let logged_in = Signal::derive(move || {
    if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site.get() {
      Some(true)
    } else {
      Some(false)
    }
  });

  let intersection_element = create_node_ref::<Div>();

  let on_scroll_element = NodeRef::<Div>::new();

  #[cfg(not(feature = "ssr"))]
  {
    let on_scroll = move |e: Event| {
      if let Some(se) = on_scroll_element.get() {
        if let Ok(Some(s)) = window().local_storage() {
          let mut query_params = query.get();
          let _ = s.set_item(
            &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
            &se.scroll_left().to_string(),
          );
        }
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

    let UseIntersectionObserverReturn {
      pause,
      resume,
      stop,
      is_active,
    } = use_intersection_observer_with_options(
      intersection_element,
      move |intersections, _| {
        if intersections[0].is_intersecting() {
          if let key = next_page_cursor.get() {
            if key > 0 {
              let mut st = ssr_page();
              st.push(key);
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
            }
          }
        }
      },
      UseIntersectionObserverOptions::default(),
    );
  }

  let search_cache_resource = Resource::new(
    move || (refresh.get(), logged_in.get(), ssr_list(), ssr_sort(), ssr_name(), ssr_page(), ssr_term()),
    move |(_refresh, _logged_in, list, sort, name, pages, term)| async move {
      let mut new_pages: HashMap<usize, Option<SearchResponse>> = HashMap::new();
      let pages_later = pages.clone();
      let pages_unit = pages_later.eq(&vec![(0usize)]);
      let mut pages_count = 1i64;
      for p in pages {
        let form = Search {
          q: term.clone(),
          type_: Some(SearchType::Posts),
          sort: Some(sort),
          community_name: None,
          community_id: None,
          page: if pages_count == 0 { None } else { Some(pages_count) },
          limit: Some(50),
          creator_id: None,
          listing_type: None,
          post_title_only: None,
        };
        let result = LemmyClient.search(form.clone()).await;
        match result {
          Ok(o) => {
            new_pages.insert(p, Some(o));
          }
          Err(e) => {}
        }
        pages_count = pages_count + 1;
      }
      (new_pages, pages_later, list, sort, name)
    },
  );

  view! {
  <main class="flex flex-col">
    <ResponsiveTopNav ssr_site />

    <div class="flex flex-grow">
      <div on:wheel=move |e: WheelEvent| {
        if let Some(se) = on_scroll_element.get() {
          // se.set_scroll_left(se.scroll_left() + e.delta_y() as i32);
          se.scroll_by_with_x_and_y(e.delta_y(), 0f64);
        }
      } node_ref=on_scroll_element class={move || {
        format!("sm:h-[calc(100%-4rem)] min-w-full sm:absolute sm:overflow-x-auto sm:overflow-y-hidden sm:columns-sm sm:px-4 gap-4{}", if loading.get() { " opacity-25" } else { "" })
      }}>

        <Transition fallback={|| {}}>
          {move || {
            match search_cache_resource.get() {
              Some(mut o) => {
                next_page_cursor.set(next_page_cursor.get() + 50usize);
                view! {
                  <div>
                    <Title text="" />
                    <For each={move || o.0.clone()} key={|r| r.0.clone()} let:r>
                      <ResponsivePostListings posts={r.1.unwrap().posts.into()} ssr_site page_number={r.0.into()} />
                    </For>
                  </div>
                }
              }
              _ => {
                view! {
                  <div>
                    <Title text="" />
                    <div class="overflow-hidden animate-[popdown_1s_step-end_1]">
                      <div class="py-4 px-8">
                        <div class="alert">
                          <span>"Loading"</span>
                        </div>
                      </div>
                    </div>
                  </div>
                }
              }
            }
          }}
          <div node_ref={intersection_element} class="block bg-transparent h-[1px]" />
        </Transition>
      </div>
    </div>
  </main>
  }
}
