use crate::ReadAuthCookie;
use crate::{client::*, errors::*, nav::TopNav};
use crate::{comment::Comment, db::csr_indexed_db::*, listing::Listing};
use lemmy_api_common::{
  lemmy_db_schema::{
    ListingType, SortType, SubscribedType,
    aggregates::structs::PostAggregates,
    newtypes::{InstanceId, PostId},
  },
  lemmy_db_views::structs::{CommentView, PostView},
  person::{GetPersonDetails, GetPersonDetailsResponse},
  site::GetSiteResponse,
};
use leptos::{html::Div, logging::*, prelude::*, task::*, *};
use leptos_meta::Title;
use leptos_router::{components::*, hooks::*};
use std::{collections::BTreeMap, vec};
use web_sys::{MouseEvent, WheelEvent};

#[component]
pub fn User() -> impl IntoView {
  // let i18n = use_i18n();
  let query = use_query_map();
  let param = use_params_map();

  let ssr_name = move || param.get().get("name").unwrap_or("".into());
  let ssr_page = move || serde_json::from_str::<Vec<usize>>(&query.get().get("page").unwrap_or("".into())).unwrap_or(vec![1usize]);
  let next_page_cursor: RwSignal<usize> = RwSignal::new(0);
  let intersection_element = NodeRef::<Div>::new();
  let on_scroll_element = NodeRef::<Div>::new();
  let loading = RwSignal::new(false);

  #[cfg(not(feature = "ssr"))]
  {
    use leptos_router::{NavigateOptions, location::State};
    use leptos_use::{
      UseIntersectionObserverOptions, UseIntersectionObserverReturn, UseScrollOptions, UseScrollReturn, use_intersection_observer_with_options,
      use_scroll_with_options,
    };
    use web_sys::Event;

    let on_scroll = move |_e: Event| {
      if let Some(se) = on_scroll_element.get() {
        #[cfg(not(feature = "ssr"))]
        spawn_local_scoped(async move {
          let query_params = query.get();
          if let Ok(d) = IndexedDb::new().await {
            let _ = d.set(&ScrollPositionKey { path: use_location().pathname.get(), query: query_params.to_query_string() }, &se.scroll_left()).await;
          }
        });
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

  let user_post_cache =
    expect_context::<RwSignal<BTreeMap<(usize, GetPersonDetails, Option<String>), (i64, LemmyAppResult<GetPersonDetailsResponse>)>>>();

  let user_resource = Resource::new(
    move || (ssr_name(), ssr_page()),
    move |(name, pages)| async move {
      let ReadAuthCookie(get_auth_cookie) = expect_context::<ReadAuthCookie>();
      let sc = user_post_cache.get_untracked();
      let mut new_pages: Vec<(usize, GetPersonDetails, i64, LemmyAppResult<GetPersonDetailsResponse>, Option<String>)> = Vec::new();
      let len = pages.len();
      for p in pages {
        let form = GetPersonDetails {
          username: Some(name.clone()),
          saved_only: None,
          community_id: None,
          limit: Some(50),
          page: Some(p.try_into().unwrap_or(1i64)),
          person_id: None,
          sort: Some(SortType::New),
        };
        #[cfg(not(feature = "ssr"))]
        loading.set(true);
        #[cfg(not(feature = "ssr"))]
        if let Some((t, Ok(r))) = sc.get(&(p, form.clone(), get_auth_cookie.get_untracked())) {
          new_pages.push((p, form.clone(), t.clone(), Ok(r.clone()), get_auth_cookie.get_untracked()));
          continue;
        }
        let t = jiff::Zoned::now().timestamp().as_millisecond();
        let result = LemmyClient.get_user(form.clone()).await;
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
    user_resource.refetch();
  };

  let show_next = RwSignal::new(true);

  let now_in_millis = RwSignal::new(u64::try_from(jiff::Zoned::now().timestamp().as_millisecond()).unwrap_or(0));

  #[derive(Debug, Clone)]
  struct PostWithComments {
    post: PostView,
    comments: RwSignal<Vec<CommentView>>,
  }

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
            <Title text="User details" />
            <For each={move || user_resource.get().unwrap_or(vec![])} key={|r| (r.1.clone(), r.2, r.4.clone())} let:r>
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
                            if let Ok(_c) = d.set::<GetPersonDetails, Result<GetPersonDetailsResponse, LemmyAppError>>(&r.1, &result_clone).await {}
                          }
                          user_post_cache.update(move |sc| {
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
                    }
                    #[cfg(not(feature = "ssr"))] loading.set(false);
                    next_page_cursor.set(r.0 + 1);

                    let t = o.clone();
                    let name = t.person_view.person.name;
                    let banner = Memo::new(move |_| t.person_view.person.banner.clone());
                    let avatar = Memo::new(move |_| t.person_view.person.avatar.clone());
                    let all_posts = RwSignal::new(
                      t
                        .posts
                        .iter()
                        .map(|p| PostWithComments {
                          post: p.clone(),
                          comments: RwSignal::new(Vec::new()),
                        })
                        .collect::<Vec<_>>(),
                    );
                    let comments = o.comments.clone();
                    all_posts
                      .update(|ap| {
                        for c in comments {
                          if let Some(pc) = ap.iter_mut().find(|p| p.post.post.id == c.post.id) {
                            pc.comments.update(|comments| comments.push(c));
                          } else {
                            ap.push(PostWithComments {
                              post: PostView {
                                post: c.post.clone(),
                                creator: c.creator.clone(),
                                community: c.community.clone(),
                                creator_banned_from_community: false,
                                creator_is_moderator: false,
                                creator_is_admin: false,
                                counts: PostAggregates {
                                  post_id: PostId(0),
                                  comments: 0,
                                  score: 0,
                                  upvotes: 0,
                                  downvotes: 0,
                                  published: std::time::SystemTime::from(jiff::Timestamp::now()).into(),
                                  newest_comment_time_necro: std::time::SystemTime::from(jiff::Timestamp::now()).into(),
                                  newest_comment_time: std::time::SystemTime::from(jiff::Timestamp::now()).into(),
                                  featured_community: false,
                                  featured_local: false,
                                  hot_rank: 0f64,
                                  hot_rank_active: 0f64,
                                  community_id: c.community.id,
                                  creator_id: c.creator.id,
                                  controversy_rank: 0f64,
                                  instance_id: InstanceId(0),
                                  scaled_rank: 0f64,
                                },
                                subscribed: SubscribedType::NotSubscribed,
                                saved: false,
                                read: false,
                                creator_blocked: false,
                                my_vote: None,
                                unread_comments: 0,
                                banned_from_community: false,
                                hidden: false,
                                image_details: None,
                              },
                              comments: RwSignal::new(vec![c]),
                            });
                          }
                        }
                        ap.sort_by(|a, b| a.post.post.published.cmp(&b.post.post.published).reverse());
                      });

                    let bio = if let Some(bio) = t.person_view.person.bio {
                      let mut options = pulldown_cmark::Options::empty();
                      options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
                      options.insert(pulldown_cmark::Options::ENABLE_TABLES);
                      options.insert(pulldown_cmark::Options::ENABLE_SUPERSCRIPT);
                      options.insert(pulldown_cmark::Options::ENABLE_SUBSCRIPT);
                      options.insert(pulldown_cmark::Options::ENABLE_CONTAINER_EXTENSIONS);
                      options.insert(pulldown_cmark::Options::ENABLE_LINKIFY_LEMMY);
                      options.insert(pulldown_cmark::Options::ENABLE_LINKIFY_HTTP);
                      let parser = pulldown_cmark::Parser::new_ext(&bio, options);
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

                    view! {
                      {
                        if r.0 == 1 {
                          view! {
                            <div class="break-inside-avoid">
                              <div class="px-4 my-2">
                                <span class="overflow-y-auto text-2xl font-extrabold wrap-anywhere">{name}</span>
                              </div>
                              <div>
                                {move || {
                                  if let Some(t) = banner.get() {
                                    let thumbnail = RwSignal::new(String::from(""));
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
                                    }
                                      .into_any()
                                  } else {
                                    view! {}.into_any()
                                  }
                                }}
                              </div>
                              <div>
                                {move || {
                                  if let Some(t) = avatar.get() {
                                    let thumbnail = RwSignal::new(String::from(""));
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
                                    }
                                      .into_any()
                                  } else {
                                    view! {
                                      <div class="py-2 px-4">
                                        <div class="block">
                                          <img class="h-16" src="/lemmy.svg" />
                                        </div>
                                      </div>
                                    }
                                      .into_any()
                                  }
                                }}
                              </div>
                              <div class="px-4 my-2">
                                <div class="select-none prose" inner_html={bio} />
                              </div>
                            </div>
                          }.into_any()
                        } else {
                          view! {}.into_any()
                        }
                      }
                      <For each={move || all_posts.get()} key={|pc| pc.post.post.id} let:pc>
                        <div class="pt-4 odd:bg-base-200">
                          <Listing hide=false post_view={pc.post} post_number=0 /*reply_show={RwSignal::new(false)}*/ />
                          <For each={move || pc.comments.get()} key={|cv| cv.comment.id} let:cv>
                            <div class="pt-2 pr-4 pl-4 pb-4">
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
                        </div>
                      </For>
                      {
                        let mut st: Vec<(usize)> = vec![next_page_cursor.get()];
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
                          <Show when=move || { show_next.get() } fallback=||{}>
                            <div class="py-4 px-8 flex justify-end{}">
                              <A href=format!("{}{}", use_location().pathname.get(), query_params.to_query_string()) attr:class="btn btn-soft">
                                "Next"
                              </A>
                            </div>
                          </Show>
                        }.into_any()
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
