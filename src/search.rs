use crate::{
  client::*,
  errors::{message_from_error, LemmyAppError},
  listings::Listings,
  nav::TopNav,
  // i18n::*,
  ResourceStatus,
  ResponseLoad,
};
use lemmy_api_common::{
  lemmy_db_schema::{ListingType, SearchType, SortType},
  lemmy_db_views::structs::PaginationCursor,
  post::{GetPosts, GetPostsResponse},
  site::{GetSiteResponse, Search, SearchResponse},
};
use leptos::{
  html::Div,
  logging::{error, log},
  prelude::*,
  *,
};
use leptos_meta::Title;
use leptos_router::{hooks::*, location::State, *};
use leptos_use::*;
use std::{
  collections::{BTreeMap, HashMap},
  usize, vec,
};
use web_sys::{js_sys::Atomics::wait_async, Event, MouseEvent, WheelEvent};

#[component]
pub fn Search() -> impl IntoView {
  // let i18n = use_i18n();
  let ssr_site_signal = expect_context::<RwSignal<Option<Result<GetSiteResponse, LemmyAppError>>>>();

  let param = use_params_map();
  let ssr_name = move || param.get().get("name").unwrap_or("".into());

  let query = use_query_map();

  let ssr_list = move || serde_json::from_str::<ListingType>(&query.get().get("list").unwrap_or("".into())).unwrap_or(ListingType::All);
  let ssr_sort = move || serde_json::from_str::<SortType>(&query.get().get("sort").unwrap_or("".into())).unwrap_or(SortType::Active);
  let ssr_page = move || serde_json::from_str::<Vec<u32>>(&query.get().get("page").unwrap_or("".into())).unwrap_or(vec![1u32]);
  let ssr_term = move || query.get().get("term").unwrap_or("".into());

  let next_page_cursor: RwSignal<u32> = RwSignal::new(0);

  let loading = RwSignal::new(false);
  let refresh = RwSignal::new(false);

  let logged_in = Signal::derive(move || {
    if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site_signal.get() {
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
          if let key = next_page_cursor.get() {
            if key > 0 {
              let mut st = ssr_page();
              st.push(key as u32);
              let mut query_params = query.get();
              query_params.insert("page".to_string(), serde_json::to_string(&st).unwrap_or("[]".into()));

              let navigate = use_navigate();
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
    move || (logged_in.get(), ssr_list(), ssr_sort(), ssr_name(), ssr_page(), ssr_term()),
    move |(_logged_in, list, sort, name, pages, term)| async move {
      let mut new_pages: Vec<(u32, Option<SearchResponse>)> = Vec::new();
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
        let result = LemmyClient.search(form.clone()).await;
        match result {
          Ok(o) => {
            log!("src {}", o.posts.len());
            new_pages.push((p, Some(o)));
          }
          Err(e) => {
            error!("err {:#?}", e);
          }
        }
      }
      (new_pages)
    },
  );

  view! {
    <main class="flex flex-col">
      <TopNav />
      <div class="flex flex-grow">
        <div
          on:wheel={move |e: WheelEvent| {
            if let Some(se) = on_scroll_element.get() {
              se.scroll_by_with_x_and_y(e.delta_y(), 0f64);
            }
          }}
          node_ref={on_scroll_element}
          class={move || {
            format!(
              "sm:h-[calc(100%-4rem)] min-w-full sm:absolute sm:overflow-x-auto sm:overflow-y-hidden sm:columns-sm sm:px-4 gap-4{}",
              if loading.get() { " opacity-25" } else { "" },
            )
          }}
        >
          <Transition fallback={|| {}}>
            {move || {
              match search_cache_resource.get() {
                Some(o) => {
                  view! {
                    <div>
                      <Title text="Search" />
                      <For each={move || o.clone()} key={|r| r.0.clone()} let:r>
                        <Listings posts={r.1.unwrap().posts.into()} page_number={RwSignal::new(((r.0 - 1) * 50) as usize)} />
                        {
                          next_page_cursor.set(r.0 + 1);
                        }
                      </For>
                    </div>
                  }
                    .into_any()
                }
                _ => {
                  view! {
                    <div>
                      <Title text="" />
                      <div class="overflow-hidden animate-[popdown_1s_step-end_1]">
                        <div class="py-4 px-8">
                          <div class="alert alert-info alert-soft">
                            <span>"Loading"</span>
                          </div>
                        </div>
                      </div>
                    </div>
                  }
                    .into_any()
                }
              }
            }} <div node_ref={intersection_element} class="block bg-transparent h-[1px]" />
          </Transition>
        </div>
      </div>
    </main>
  }
}
