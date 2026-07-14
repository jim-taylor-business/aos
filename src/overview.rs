use crate::ReadAuthCookie;
use crate::db::csr_indexed_db::*;
use crate::errors::Offline;
use crate::{
  // i18n::*,
  client::*,
  errors::{Error, LemmyAppError, LemmyAppErrorType, LemmyAppResult, Loading},
  icon::{IconType::*, *},
  listings::Listings,
  nav::TopNav,
};
use hooks::*;
use lemmy_api_common::community::GetCommunity;
use lemmy_api_common::lemmy_db_schema::SubscribedType;
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
use send_wrapper::SendWrapper;
use std::{collections::BTreeMap, usize, vec};
use web_sys::{Event, MouseEvent, ScrollToOptions, WheelEvent};

#[component]
pub fn Overview(#[prop(optional)] ssr_name: Signal<Option<String>>) -> impl IntoView {
  // let i18n = use_i18n();

  let ssr_list = move || serde_json::from_str::<ListingType>(&use_query_map().get().get("list").unwrap_or("".into())).unwrap_or(ListingType::All);
  let ssr_sort = move || serde_json::from_str::<SortType>(&use_query_map().get().get("sort").unwrap_or("".into())).unwrap_or(SortType::Active);
  let ssr_page = move || serde_json::from_str::<Vec<(usize, String)>>(&use_query_map().get().get("page").unwrap_or("".into())).unwrap_or(vec![]);

  let response_cache = expect_context::<RwSignal<BTreeMap<(usize, GetPosts, Option<String>), (i64, LemmyAppResult<GetPostsResponse>)>>>();
  let next_page_cursor: RwSignal<(usize, Option<PaginationCursor>)> = RwSignal::new((0, None));

  let loading = RwSignal::new(false);
  let ssr_site = expect_context::<Resource<Result<GetSiteResponse, LemmyAppError>>>();

  let intersection_element = NodeRef::<Div>::new();
  let on_scroll_element = NodeRef::<Div>::new();

  let on_scroll = move |e: Event| {
    #[cfg(not(feature = "ssr"))]
    if let Some(se) = on_scroll_element.get() {
      spawn_local_scoped_with_cancellation(async move {
        if let Ok(d) = IndexedDb::new().await {
          let _ = d
            .set(&ScrollPositionKey { path: use_location().pathname.get(), query: use_query_map().get().to_query_string() }, &se.scroll_left())
            .await;
        }
      });
    }
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

            #[cfg(not(feature = "ssr"))]
            if let Some(se) = on_scroll_element.get() {
              let params = query_params.clone();
              spawn_local_scoped_with_cancellation(async move {
                if let Ok(d) = IndexedDb::new().await {
                  let _ = d.set(&ScrollPositionKey { path: use_location().pathname.get(), query: params.to_query_string() }, &se.scroll_left()).await;
                }
                use_navigate()(
                  &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
                  NavigateOptions { resolve: false, replace: true, scroll: false, state: State::default() },
                );
              });
            }
          }
        }
      },
      UseIntersectionObserverOptions::default(),
    );
  }

  #[cfg(not(feature = "ssr"))]
  fn load_cache(fc: GetPosts) -> impl std::future::Future<Output = Result<GetPostsResponse, LemmyAppError>> {
    SendWrapper::new(async move {
      if let Ok(d) = IndexedDb::new().await {
        if let Ok(c) = d.load::<GetPosts, Result<GetPostsResponse, LemmyAppError>>(&fc).await {
          if let Some(r) = c { r } else { Err(LemmyAppError { error_type: LemmyAppErrorType::Unknown, content: "".to_owned() }) }
        } else {
          Err(LemmyAppError { error_type: LemmyAppErrorType::Unknown, content: "".to_owned() })
        }
      } else {
        Err(LemmyAppError { error_type: LemmyAppErrorType::Unknown, content: "".to_owned() })
      }
    })
  }

  let post_list_resource = Resource::new(
    move || (ssr_list(), ssr_sort(), ssr_name.get(), ssr_page()),
    move |(list, sort, name, mut pages)| async move {
      let many_pages = pages.len() > 0;

      #[cfg(feature = "ssr")]
      let do_not_render_scroll = true && pages.len() == 0;
      #[cfg(not(feature = "ssr"))]
      let do_not_render_scroll = false;

      #[cfg(feature = "ssr")]
      let csr_cache_render = true && pages.len() > 0;
      #[cfg(not(feature = "ssr"))]
      let csr_cache_render = false;

      #[cfg(not(feature = "ssr"))]
      loading.set(true);

      let ReadAuthCookie(get_auth_cookie) = expect_context::<ReadAuthCookie>();
      let rc = response_cache.get_untracked();
      let mut new_pages: Vec<(usize, GetPosts, i64, LemmyAppResult<GetPostsResponse>, Option<String>, bool, bool)> = vec![];
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
        if let Some((t, Ok(r))) = rc.get(&(p.0, form.clone(), get_auth_cookie.get_untracked())) {
          new_pages.push((p.0, form.clone(), t.clone(), Ok(r.clone()), get_auth_cookie.get_untracked(), do_not_render_scroll, csr_cache_render));
          continue;
        } else {
          if many_pages {
            match load_cache(form.clone()).await {
              Ok(o) => {
                new_pages.push((
                  p.0,
                  form.clone(),
                  chrono::Utc::now().timestamp_millis(),
                  Ok(o),
                  get_auth_cookie.get_untracked(),
                  do_not_render_scroll,
                  csr_cache_render,
                ));
                continue;
              }
              _ => {}
            }
          }
        }

        let result = match LemmyClient.list_posts(form.clone()).await {
          Ok(mut o) => {
            o.posts.retain(|p| !p.banned_from_community);
            Ok(o)
          }
          Err(e) => Err(e),
        };
        new_pages.push((
          p.0,
          form.clone(),
          chrono::Utc::now().timestamp_millis(),
          result,
          get_auth_cookie.get_untracked(),
          do_not_render_scroll,
          csr_cache_render,
        ));
      }

      new_pages
    },
  );

  let details_resource = Resource::new(
    move || (ssr_name.get()),
    move |(name)| async move {
      if let Some(name) = name {
        let form = GetCommunity { id: None, name: Some(name) };
        let result = match LemmyClient.get_community(form.clone()).await {
          Ok(o) => Ok(Some(o)),
          Err(e) => Err(e),
        };
        result
      } else {
        Ok(None)
      }
    },
  );

  let on_retry_click = move |_e: MouseEvent| {
    post_list_resource.refetch();
  };

  let on_retry_site_click = move |_e: MouseEvent| {
    spawn_local_scoped_with_cancellation(async move {
      LemmyClient.get_site().await;
    });
  };

  #[cfg(not(feature = "ssr"))]
  let cancel_handle: RwSignal<Option<TimeoutHandle>> = RwSignal::new(None);
  #[cfg(not(feature = "ssr"))]
  let cancel_refresh_handle: RwSignal<Option<TimeoutHandle>> = RwSignal::new(None);

  let show_rules = RwSignal::new(false);
  let on_show_rules = move |e: MouseEvent| {
    e.prevent_default();
    show_rules.set(!show_rules.get());
  };

  view! {
      <main class="flex flex-col">
        <TopNav scroll_element=on_scroll_element.into() />
        <div class="flex flex-grow">
          <div
            on:wheel={move |e: WheelEvent| {
              let iw = window().inner_width().ok().map(|b| b.as_f64().unwrap_or(0.0)).unwrap_or(0.0);
              if iw < 768f64 {
              } else {
                if e.delta_x() != 0.0 {
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
              }
            }}
            node_ref={on_scroll_element}
            class={move || { "sm:h-[calc(100%-4rem)] min-w-full sm:absolute sm:overflow-x-auto sm:overflow-y-hidden sm:columns-[23rem] sm:px-4 gap-4" }}
          >
            <Transition fallback={|| {}}>
              {move || {
                match ssr_site.get() {
                  Some(Err(e)) => {
                    view! {
                      <Error error={e} on_retry_click={Some(on_retry_site_click)} />
                    }.into_any()
                  }
                  Some(Ok(s)) => {
                    view! {}.into_any()
                  }
                  _ => view! {}.into_any(),
                }
              }}
            </Transition>
            <Transition fallback={|| {}}>
              {move || {
                match details_resource.get() {
                  Some(Err(e)) => {
                    view! {
                      <Error error={e} on_retry_click={None::<fn(MouseEvent) -> ()>} />
                    }.into_any()
                  }
                  Some(Ok(Some(s))) => {
                    let community_title_encoded = html_escape::encode_safe(&s.community_view.community.title).to_string();

                    let description = if let Some(description) = s.community_view.community.description {
                      let mut options = pulldown_cmark::Options::empty();
                      options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
                      options.insert(pulldown_cmark::Options::ENABLE_TABLES);
                      options.insert(pulldown_cmark::Options::ENABLE_SUPERSCRIPT);
                      options.insert(pulldown_cmark::Options::ENABLE_SUBSCRIPT);
                      options.insert(pulldown_cmark::Options::ENABLE_CONTAINER_EXTENSIONS);
                      let parser = pulldown_cmark::Parser::new_ext(&description, options);
                      let custom = parser
                        .map(|event| match event {
                          pulldown_cmark::Event::Html(text) => {
                            let er = format!("<p>{}</p>", html_escape::encode_safe(&text).to_string());
                            pulldown_cmark::Event::Html(er.into())
                          }
                          pulldown_cmark::Event::InlineHtml(text) => {
                            let er = html_escape::encode_safe(&text).to_string();
                            pulldown_cmark::Event::InlineHtml(er.into())
                          }
                          _ => event,
                        });
                      let mut description_encoded = String::new();
                      pulldown_cmark::html::push_html(&mut description_encoded, custom);
                      description_encoded
                    } else {
                      String::new()
                    };

                    let thumbnail_url = Memo::new(move |_| s.community_view.community.banner.clone());
                    let thumbnail = RwSignal::new(String::from(""));
                    let follow = Memo::new(move |_| s.community_view.subscribed.clone());

                    view! {
                      <div class="break-inside-avoid mb-4">
                        <div class="py-2 px-4">
                          <span class="overflow-y-auto text-3xl font-extrabold wrap-anywhere" inner_html={community_title_encoded} />
                        </div>
                        <div>
                          {move || {
                            if let Some(t) = thumbnail_url.get() {
                              let h = t.inner().to_string();
                              thumbnail.set(h);
                              view! {
                                <div class="py-2 px-4">
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
                              }.into_any()
                            } else {
                              view! {
                                // <div class="py-2 px-4">
                                //   <div class="block">
                                //     <img class="h-16" src="/lemmy.svg" />
                                //   </div>
                                // </div>
                              }.into_any()
                            }
                          }}
                        </div>
                        <div class="px-4 break-inside-avoid">
                          <div class="flex flex-wrap gap-x-2 items-center py-2">
                          <Form action="PUT" attr:class="flex items-center">
                            <button
                              type="submit"
                              on:click={on_show_rules}
                              title="Rules"
                              class={move || {
                                format!(
                                  "{}",
                                  { if show_rules.get() { "text-accent" } else { "" } },
                                )
                              }}
                            >
                              <Icon icon={Rules} />
                            </button>
                          </Form>
                          <Form action="PUT" attr:class="flex items-center">
                            // <input type="hidden" name="post_id" value={format!("{}", post_view.get_untracked().post.id)} />
                            // <input type="hidden" name="save" value={move || format!("{}", !post_view.get().saved)} />
                            <button
                              type="submit"
                              // on:click={on_save_submit}
                              title="Subscribed"
                              class={move || {
                                format!(
                                  "{}",
                                  { if follow.get() == SubscribedType::Subscribed { "text-accent" } else { "" } },
                                )
                              }}
                              // class={move || {
                              //   // format!(
                              // //     "{}{}",
                              // //     { if post_view.get().saved { "text-accent" } else { "" } },
                              // //     { if !logged_in.get() || !online.get().0 {
                              // " text-base-content/50"
                              // // } else { " hover:text-accent/50" } },
                              // //   )
                              // }}
                              // disabled={move || true}// !logged_in.get() || !online.get().0}
                            >
                              <Icon icon={Subscribe} />
                            </button>
                          </Form>
                          </div>
                        </div>
                        <div class="py-2 px-4" style={move || { if show_rules.get() { "display: block;" } else { "display: none;" } }}>
                          <div class="prose select-none" inner_html={description} />
                        </div>
                      </div>
                    }.into_any()
                  }
                  _ => view! {}.into_any(),
                }
              }}
            </Transition>
            <Transition fallback={|| {}}>
              // <Title text="" />
              <For each={move || post_list_resource.get().unwrap_or(vec![])} key={|p| (p.1.clone(), p.2, p.4.clone())} let:p>
                {match p.3 {
                  Ok(ref o) => {
                    // log!("{} {}", !p.5, p.6);
                    #[cfg(not(feature = "ssr"))]
                    {
                      let rw = p.3.clone();
                      let fm = p.1.clone();
                      use crate::db::csr_indexed_db::*;
                      spawn_local_scoped_with_cancellation(async move {
                        if p.6 {} else {
                          if let Ok(d) = IndexedDb::new().await {
                            if let Ok(_c) = d.set::<GetPosts, Result<GetPostsResponse, LemmyAppError>>(&fm, &rw).await {}
                          }
                          response_cache
                            .update(move |rc| {
                              rc.insert((p.0, fm, p.4), (p.2, rw));
                            });
                        }
                      });
                      let iw = window().inner_width().ok().map(|b| b.as_f64().unwrap_or(0.0)).unwrap_or(0.0);
                      if iw < 768f64 || p.5 || p.6 {} else {
                        if let Some(c) = cancel_handle.get_untracked() {
                          c.clear();
                        }
                        cancel_handle.set(set_timeout_with_handle(
                          move || {
                            if let Some(s) = on_scroll_element.get() {
                              spawn_local_scoped_with_cancellation(async move {
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
                        ).ok());
                      }
                      if p.6 {
                        if let Some(c) = cancel_refresh_handle.get_untracked() {
                          c.clear();
                        }
                        cancel_refresh_handle.set(set_timeout_with_handle(
                          move || {
                            post_list_resource.refetch();
                          },
                          std::time::Duration::new(0, 750_000_000),
                        ).ok());
                      }
                    }
                    next_page_cursor.set((p.0 + o.posts.len(), o.next_page.clone()));
                    #[cfg(not(feature = "ssr"))]
                    loading.set(false);
                    view! {
                      // attr:class=format!("{}", if p.6 { "display: hidden" } else { "" })
                      <Listings hide=p.6 posts={o.posts.clone().into()} page_number={RwSignal::new(p.0)} />
                    }.into_any()
                  }
                  Err(LemmyAppError { error_type: LemmyAppErrorType::OfflineError, .. }) => {
                    #[cfg(not(feature = "ssr"))]
                    loading.set(false);
                    view! {
                      <Offline on_retry_click={Some(on_retry_click)} />
                    }
                    .into_any()
                  }
                  Err(e) => {
                    #[cfg(not(feature = "ssr"))]
                    loading.set(false);
                    error!("{:#?}", e);
                    view! {
                      <Error error={e} on_retry_click={Some(on_retry_click)} />
                    }
                    .into_any()
                  }
                }}
              </For>
            </Transition>
            <div node_ref={intersection_element} class="block bg-transparent h-[1px]" />
            {move || { view!{ <Loading loading=loading.get() /> } }}
          </div>
        </div>
      </main>
  }
}
