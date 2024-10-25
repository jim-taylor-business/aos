use crate::{
  errors::{message_from_error, LemmyAppError, LemmyAppErrorType},
  i18n::*,
  lemmy_client::*,
  ui::components::{
    common::about::About,
    home::{site_summary::SiteSummary, trending::Trending},
    post::post_listings::PostListings,
  },
  OnlineSetter,
  ResourceStatus, // TitleSetter,
};
use lemmy_api_common::{
  lemmy_db_schema::{ListingType, SortType},
  lemmy_db_views::structs::PaginationCursor,
  post::{GetPosts, GetPostsResponse},
  site::GetSiteResponse,
};
use leptos::{html::*, *};
use leptos_meta::*;
use leptos_router::*;
use std::{collections::BTreeMap, usize, vec};
use web_sys::MouseEvent;

#[component]
pub fn HomeActivity(ssr_site: Resource<Option<bool>, Result<GetSiteResponse, LemmyAppError>>) -> impl IntoView {
  let i18n = use_i18n();

  let error = expect_context::<RwSignal<Vec<Option<(LemmyAppError, Option<RwSignal<bool>>)>>>>();
  // let title = expect_context::<RwSignal<Option<TitleSetter>>>();
  let online = expect_context::<RwSignal<OnlineSetter>>();

  let param = use_params_map();
  let community_name = move || param.get().get("name").cloned();

  let query = use_query_map();

  let ssr_list = move || serde_json::from_str::<ListingType>(&query.get().get("list").cloned().unwrap_or("".into())).unwrap_or(ListingType::All);
  let ssr_sort = move || serde_json::from_str::<SortType>(&query.get().get("sort").cloned().unwrap_or("".into())).unwrap_or(SortType::Active);
  let ssr_from = move || {
    serde_json::from_str::<(usize, Option<PaginationCursor>)>(&query.get().get("from").cloned().unwrap_or("".into())).unwrap_or((0usize, None))
  };
  let ssr_prev =
    move || serde_json::from_str::<Vec<(usize, Option<PaginationCursor>)>>(&query.get().get("prev").cloned().unwrap_or("".into())).unwrap_or(vec![]);
  let ssr_limit = move || query.get().get("limit").cloned().unwrap_or("".into()).parse::<usize>().unwrap_or(10usize);

  let on_sort_click = move |s: SortType| {
    move |_e: MouseEvent| {
      let r = serde_json::to_string::<SortType>(&s);
      let mut query_params = query.get();
      match r {
        Ok(o) => {
          query_params.insert("sort".into(), o);
        }
        Err(e) => {
          error.update(|es| es.push(Some((e.into(), None))));
        }
      }
      if SortType::Active == s {
        query_params.remove("sort".into());
      }
      let navigate = leptos_router::use_navigate();
      navigate(
        &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
        Default::default(),
      );
    }
  };

  let loading = RwSignal::new(false);
  let refresh = RwSignal::new(false);

  let logged_in = Signal::derive(move || {
    if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site.get() {
      Some(true)
    } else {
      Some(false)
    }
  });

  // title.set(None);

  let posts_resource = Resource::new(
    move || {
      (
        refresh.get(),
        logged_in.get(),
        ssr_list(),
        ssr_sort(),
        ssr_from(),
        ssr_limit(),
        community_name(),
      )
    },
    move |(_refresh, _logged_in, list_type, sort_type, from, limit, name)| async move {
      loading.set(true);

      let form = GetPosts {
        type_: Some(list_type),
        sort: Some(sort_type),
        community_name: name,
        community_id: None,
        page: None,
        limit: Some(i64::try_from(limit).unwrap_or(10)),
        saved_only: None,
        disliked_only: None,
        liked_only: None,
        page_cursor: from.1.clone(),
        // show_hidden: None,
      };

      if online.get().0 {
        let result = LemmyClient.list_posts(form.clone()).await;
        loading.set(false);
        // title.set(None);
        match result {
          Ok(o) => {
            #[cfg(not(feature = "ssr"))]
            if let Ok(Some(s)) = window().local_storage() {
              if let Ok(Some(_)) = s.get_item(&serde_json::to_string(&form).ok().unwrap()) {}
              let _ = s.set_item(&serde_json::to_string(&form).ok().unwrap(), &serde_json::to_string(&o).ok().unwrap());
            }
            Ok((from, o))
          }
          Err(e) => {
            error.update(|es| es.push(Some((e.clone(), None))));
            Err((e, Some(refresh)))
          }
        }
      } else {
        #[cfg(not(feature = "ssr"))]
        if let Ok(Some(s)) = window().local_storage() {
          if let Ok(Some(c)) = s.get_item(&serde_json::to_string(&form).ok().unwrap()) {
            if let Ok(o) = serde_json::from_str::<GetPostsResponse>(&c) {
              loading.set(false);
              return Ok((from, o));
            }
          }
        }

        loading.set(false);
        let e = LemmyAppError {
          error_type: LemmyAppErrorType::OfflineError,
          content: String::from(""),
        };
        error.update(|es| es.push(Some((e.clone(), None))));
        Err((e, Some(refresh)))
      }
    },
  );

  let csr_resources = expect_context::<RwSignal<BTreeMap<(usize, ResourceStatus), (Option<PaginationCursor>, Option<GetPostsResponse>)>>>();
  let csr_sort = expect_context::<RwSignal<SortType>>();
  let csr_next_page_cursor = expect_context::<RwSignal<(usize, Option<PaginationCursor>)>>();

  let on_csr_sort_click = move |s: SortType| {
    move |_e: MouseEvent| {
      csr_next_page_cursor.set((0, None));
      csr_sort.set(s);
      csr_resources.set(BTreeMap::new());
    }
  };

  let _resize_element = create_node_ref::<Main>();
  let _scroll_element = create_node_ref::<Div>();

  #[cfg(not(feature = "ssr"))]
  {
    use leptos_use::*;

    use_resize_observer(_resize_element, move |entries, _| {
      let _rect = entries[0].content_rect();
      // logging::log!("width: {:.0} height: {:.0}", rect.width(), rect.height());
      let iw = window().inner_width().ok().map(|b| b.as_f64().unwrap_or(0.0)).unwrap_or(0.0);

      let mut query_params = query.get();

      let prev_limit = if let Some(l) = query_params.get("limit".into()) {
        Some(l.clone())
      } else {
        None
      };

      let new_limit = if iw >= 2560f64 {
        query_params.insert("limit".into(), "40".to_string());
        Some("40".to_string())
      } else if iw >= 1920f64 {
        query_params.insert("limit".into(), "30".to_string());
        Some("30".to_string())
      } else if iw >= 1536f64 {
        query_params.insert("limit".into(), "20".to_string());
        Some("20".to_string())
      } else {
        query_params.remove("limit");
        None
      };

      if iw >= 640f64 {
        csr_resources.set(BTreeMap::new());
        csr_next_page_cursor.set((0, None));
      }

      if prev_limit.ne(&new_limit) {
        let navigate = leptos_router::use_navigate();
        if iw >= 640f64 {
          navigate(
            &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
            Default::default(),
          );
        } else {
          navigate("/", Default::default());
        }
      }
    });

    use_intersection_observer_with_options(
      _scroll_element,
      move |_entries, _| {
        let iw = window().inner_width().ok().map(|b| b.as_f64().unwrap_or(0.0)).unwrap_or(0.0);

        if iw < 640f64 {
          if csr_resources
            .get()
            .get(&(csr_next_page_cursor.get().0, ResourceStatus::Loading))
            .is_none()
            && csr_resources.get().get(&(csr_next_page_cursor.get().0, ResourceStatus::Ok)).is_none()
            && csr_resources.get().get(&(csr_next_page_cursor.get().0, ResourceStatus::Err)).is_none()
          {
            csr_resources.update(|h| {
              h.insert(
                (csr_next_page_cursor.get().0, ResourceStatus::Loading),
                (csr_next_page_cursor.get().1, None),
              );
            });

            let _csr_resource = create_local_resource(
              move || (),
              move |()| async move {
                let from = csr_next_page_cursor.get();

                let form = GetPosts {
                  // type_: Some(list_type),
                  type_: Some(ListingType::All),
                  sort: Some(csr_sort.get()),
                  community_name: None,
                  community_id: None,
                  page: None,
                  limit: Some(10),
                  saved_only: None,
                  disliked_only: None,
                  liked_only: None,
                  page_cursor: from.1.clone(),
                  // show_hidden: None,
                };

                let result = LemmyClient.list_posts(form).await;

                match result {
                  Ok(o) => {
                    csr_next_page_cursor.set((from.0 + ssr_limit(), o.next_page.clone()));
                    csr_resources.update(move |h| {
                      h.remove(&(from.0, ResourceStatus::Loading));
                      h.insert((from.0, ResourceStatus::Ok), (from.1.clone(), Some(o.clone())));
                    });
                    Some(())
                  }
                  Err(e) => {
                    csr_resources.update(move |h| {
                      h.remove(&(from.0, ResourceStatus::Loading));
                      h.insert((from.0, ResourceStatus::Err), (from.1, None));
                    });
                    error.update(|es| es.push(Some((e, Some(refresh)))));
                    None
                  }
                }
              },
            );
          }
        }
      },
      UseIntersectionObserverOptions::default(),
    );
  }

  let on_retry_click = move |i: (usize, ResourceStatus)| {
    move |_e: MouseEvent| {
      let _csr_resource = create_local_resource(
        move || (),
        move |()| //{
        async move {
          let from = csr_resources.get().get(&i).unwrap().0.clone();

          let form = GetPosts {
            // type_: Some(list_type),
            type_: Some(ListingType::All),
            sort: Some(csr_sort.get()),
            community_name: None,
            community_id: None,
            page: None,
            // limit: Some(i64::try_from(limit).unwrap_or(10)),
            limit: Some(10),
            saved_only: None,
            disliked_only: None,
            liked_only: None,
            page_cursor: from.clone(),
            // show_hidden: None,
          };

          let from_clone = from.clone();

          csr_resources.update(move |h| {
            h.remove(&(i.0, ResourceStatus::Err));
            h.insert((i.0, ResourceStatus::Loading), (from_clone, None));
          });

          let result = LemmyClient.list_posts(form).await;

          match result {
            Ok(o) => {
              csr_next_page_cursor.set((i.0 + ssr_limit(), o.next_page.clone()));
              csr_resources.update(move |h| {
                h.remove(&(i.0, ResourceStatus::Loading));
                h.insert((i.0, ResourceStatus::Ok), (from, Some(o.clone())));
              });
              Some(())
            }
            Err(e) => {
              csr_resources.update(move |h| {
                h.remove(&(i.0, ResourceStatus::Loading));
                h.insert((i.0, ResourceStatus::Err), (from, None));
              });
              error.update(|es| es.push(Some((e, None))));
              None
            }
          }
        },
      );
    }
  };

  view! {
    <div class="block">
      <div class="hidden mr-3 sm:inline-block join">
        <button class="btn join-item btn-active">"Posts"</button>
        <button class="btn join-item btn-disabled">"Comments"</button>
      </div>
      <div class="hidden mr-3 sm:inline-block join">
        <A
          href={move || {
            let mut query_params = query.get();
            query_params.insert("list".into(), serde_json::to_string(&ListingType::Subscribed).ok().unwrap());
            format!("{}{}", use_location().pathname.get(), query_params.to_query_string())
          }}
          class={move || {
            format!(
              "btn join-item{}{}",
              if ListingType::Subscribed == ssr_list() { " btn-active" } else { "" },
              if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site.get() { "" } else { " btn-disabled" },
            )
          }}
        >
          "Subscribed"
        </A>
        <A
          href={move || {
            let mut query_params = query.get();
            query_params.insert("list".into(), serde_json::to_string(&ListingType::Local).ok().unwrap());
            format!("{}{}", use_location().pathname.get(), query_params.to_query_string())
          }}
          class={move || format!("btn join-item{}", if ListingType::Local == ssr_list() { " btn-active" } else { "" })}
        >
          "Local"
        </A>
        <A
          href={move || {
            let mut query_params = query.get();
            query_params.remove("list".into());
            format!("{}{}", use_location().pathname.get(), query_params.to_query_string())
          }}
          class={move || format!("btn join-item{}", if ListingType::All == ssr_list() { " btn-active" } else { "" })}
        >
          "All"
        </A>
      </div>
      <div class="hidden ml-3 sm:inline-block sm:ml-0 dropdown">
        <label tabindex="0" class="btn">
          "Sort"
        </label>
        <ul tabindex="0" class="shadow menu dropdown-content z-[1] bg-base-100 rounded-box">
          <li
            class={move || { (if SortType::Active == ssr_sort() { "btn-active" } else { "" }).to_string() }}
            on:click={on_sort_click(SortType::Active)}
          >
            <span>{t!(i18n, active)}</span>
          </li>
          <li class={move || { (if SortType::Hot == ssr_sort() { "btn-active" } else { "" }).to_string() }} on:click={on_sort_click(SortType::Hot)}>
            <span>{t!(i18n, hot)}</span>
          </li>
          <li
            class={move || { (if SortType::Scaled == ssr_sort() { "btn-active" } else { "" }).to_string() }}
            on:click={on_sort_click(SortType::Scaled)}
          >
            <span>{"Scaled"}</span>
          </li>
          <li class={move || { (if SortType::New == ssr_sort() { "btn-active" } else { "" }).to_string() }} on:click={on_sort_click(SortType::New)}>
            <span>{t!(i18n, new)}</span>
          </li>
        </ul>
      </div>
      <div class="inline-block ml-3 sm:hidden sm:ml-0 dropdown">
        <label tabindex="0" class="btn">
          "Sort"
        </label>
        <ul tabindex="0" class="shadow menu dropdown-content z-[1] bg-base-100 rounded-box">
          <li
            class={move || { (if SortType::Active == csr_sort.get() { "btn-active" } else { "" }).to_string() }}
            on:click={on_csr_sort_click(SortType::Active)}
          >
            <span>{t!(i18n, active)}</span>
          </li>
          <li
            class={move || { (if SortType::Hot == csr_sort.get() { "btn-active" } else { "" }).to_string() }}
            on:click={on_csr_sort_click(SortType::Hot)}
          >
            <span>{t!(i18n, hot)}</span>
          </li>
          <li
            class={move || { (if SortType::Scaled == csr_sort.get() { "btn-active" } else { "" }).to_string() }}
            on:click={on_csr_sort_click(SortType::Scaled)}
          >
            <span>{"Scaled"}</span>
          </li>
          <li
            class={move || { (if SortType::New == csr_sort.get() { "btn-active" } else { "" }).to_string() }}
            on:click={on_csr_sort_click(SortType::New)}
          >
            <span>{t!(i18n, new)}</span>
          </li>
        </ul>
      </div>
    </div>
    <main node_ref={_resize_element} class="flex flex-col flex-grow w-full sm:flex-row">
      <div class="relative w-full sm:pr-4 lg:w-2/3 2xl:w-3/4 3xl:w-4/5 4xl:w-5/6">

        <Transition fallback={|| {}}>
          {move || {
            match posts_resource.get() {
              Some(Err(err)) => {
                view! {
                  <Title text="Error loading post list" />
                  <div class="py-4 px-8">
                    <div class="flex justify-between alert alert-error">
                      <span>{message_from_error(&err.0)} " - " {err.0.content}</span>
                      <div>
                        <Show when={move || { if let Some(_) = err.1 { true } else { false } }} fallback={|| {}}>
                          <button
                            on:click={move |_| {
                              if let Some(r) = err.1 {
                                r.set(!r.get());
                              } else {}
                            }}
                            class="btn btn-sm"
                          >
                            "Retry"
                          </button>
                        </Show>
                      </div>
                    </div>
                  </div>
                  <div class="hidden sm:block join">
                    {
                      let mut st = ssr_prev();
                      let p = st.pop();
                      let mut query_params = query.get();
                      if st.len() > 0 {
                        query_params.insert("prev".into(), serde_json::to_string(&st).unwrap_or("[]".into()));
                      } else {
                        query_params.remove("prev".into());
                      }
                      if p.ne(&Some((0, None))) {
                        query_params.insert("from".into(), serde_json::to_string(&p).unwrap_or("[0,None]".into()));
                      } else {
                        query_params.remove("from".into());
                      }
                      view! {
                        <A
                          on:click={move |_| {
                            loading.set(true);
                          }}
                          href={format!("{}{}", use_location().pathname.get(), query_params.to_query_string())}
                          class={move || format!("btn join-item{}", if !ssr_prev().is_empty() { "" } else { " btn-disabled" })}
                        >
                          "Prev"
                        </A>
                      }
                    }
                    {
                      let query_params = query.get();
                      view! {
                        <A
                          href={format!("{}{}", use_location().pathname.get(), query_params.to_query_string())}
                          // class=move || format!("btn join-item{}{}", if next_page.is_some() && !loading.get() { "" } else { " btn-disabled" }, if loading.get() { " btn-disabled" } else { "" } )
                          class="btn join-item btn-disabled"
                        >
                          "Next"
                        </A>
                      }
                    }
                  </div>
                }
              }
              Some(Ok(posts)) => {
                let next_page = Some((posts.0.0 + ssr_limit(), posts.1.next_page.clone()));
                view! {
                  <Title text={format!("Page {}", 1 + (ssr_from().0 / ssr_limit()))} />

                  // {loading
                  // .get()
                  // .then(move || {
                  // view! {
                  // <div class="overflow-hidden animate-[popdown_1s_step-end_1]">
                  // <div class="py-4 px-8">
                  // <div class="alert">
                  // <span>"Loading"</span>
                  // </div>
                  // </div>
                  // </div>
                  // }
                  // })}
                  <div class={move || {
                    format!(
                      "hidden sm:block columns-1 2xl:columns-2 3xl:columns-3 4xl:columns-4 gap-0{}",
                      if loading.get() { " opacity-25" } else { "" },
                    )
                  }}>
                    <PostListings posts={posts.1.posts.into()} ssr_site page_number={posts.0.0.into()} />
                  </div>
                  <div class="hidden sm:block join">
                    {
                      let mut st = ssr_prev();
                      let p = st.pop();
                      let mut query_params = query.get();
                      if st.len() > 0 {
                        query_params.insert("prev".into(), serde_json::to_string(&st).unwrap_or("[]".into()));
                      } else {
                        query_params.remove("prev".into());
                      }
                      if p.ne(&Some((0, None))) {
                        query_params.insert("from".into(), serde_json::to_string(&p).unwrap_or("[0,None]".into()));
                      } else {
                        query_params.remove("from".into());
                      }
                      view! {
                        <A
                          on:click={move |_| {
                            loading.set(true);
                          }}
                          href={format!("{}{}", use_location().pathname.get(), query_params.to_query_string())}
                          class={move || format!("btn join-item{}", if !ssr_prev().is_empty() { "" } else { " btn-disabled" })}
                        >
                          "Prev"
                        </A>
                      }
                    }
                    {
                      let mut st = ssr_prev();
                      st.push(ssr_from());
                      let mut query_params = query.get();
                      query_params.insert("prev".into(), serde_json::to_string(&st).unwrap_or("[]".into()));
                      query_params.insert("from".into(), serde_json::to_string(&next_page).unwrap_or("[0,None]".into()));
                      view! {
                        <A
                          on:click={move |_| {
                            loading.set(true);
                          }}
                          href={format!("{}{}", use_location().pathname.get(), query_params.to_query_string())}
                          class={move || {
                            format!(
                              "btn join-item{}{}",
                              if next_page.clone().unwrap_or((0, None)).1.is_some() && !loading.get() { "" } else { " btn-disabled" },
                              if loading.get() { " btn-disabled" } else { "" },
                            )
                          }}
                        >
                          "Next"
                        </A>
                      }
                    }
                  </div>
                }
              }
              None => {
                view! {
                  <Title text="Loading post list" />
                  <div class="overflow-hidden animate-[popdown_1s_step-end_1]">
                    <div class="py-4 px-8">
                      <div class="alert">
                        <span>"Loading"</span>
                      </div>
                    </div>
                  </div>
                  <div class="hidden" />
                }
              }
            }
          }}
        </Transition>

        <For each={move || csr_resources.get()} key={|r| r.0.clone()} let:r>
          {
            let r_copy = r.clone();
            view! {
              <Title text="" />
              <Show
                when={move || r.0.1 == ResourceStatus::Ok}
                fallback={move || {
                  match r_copy.0.1 {
                    ResourceStatus::Err => {
                      view! {
                        <div class="py-4 px-8">
                          <div class="flex justify-between alert alert-error">
                            <span class="text-lg">"Error"</span>
                            <span on:click={on_retry_click(r_copy.0)} class="btn btn-sm">
                              "Retry"
                            </span>
                          </div>
                        </div>
                      }
                    }
                    _ => {
                      view! {
                        <div class="overflow-hidden animate-[popdown_1s_step-end_1]">
                          <div class="py-4 px-8">
                            <div class="alert">
                              <span>"Loading..."</span>
                            </div>
                          </div>
                        </div>
                      }
                    }
                  }
                }}
              >
                <PostListings posts={r.1.clone().1.unwrap().posts.into()} ssr_site page_number={r.0.0.into()} />
              </Show>
            }
          }
        </For>
        <div node_ref={_scroll_element} class="block bg-transparent sm:hidden h-[1px]" />

      </div>
      <div class="hidden lg:block lg:w-1/3 2xl:w-1/4 3xl:w-1/5 4xl:w-1/6">
        <About />
        <SiteSummary ssr_site />
        <Trending />
      </div>
    </main>
  }
}
