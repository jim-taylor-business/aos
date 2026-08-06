use crate::{
  client::*,
  errors::{Error, LemmyAppError},
  nav::TopNav,
  // i18n::*,
};
use crate::{comment::Comment, db::csr_indexed_db::*, listing::Listing};
use lemmy_api_common::{
  lemmy_db_schema::{
    ListingType, SortType, SubscribedType,
    aggregates::structs::PostAggregates,
    newtypes::{InstanceId, PostId},
  },
  lemmy_db_views::structs::{CommentView, PostView},
  person::GetPersonDetails,
  site::GetSiteResponse,
};
use leptos::{html::Div, prelude::*, task::*, *};
use leptos_router::hooks::*;
use std::vec;
use web_sys::{MouseEvent, WheelEvent};

#[component]
pub fn User() -> impl IntoView {
  // let i18n = use_i18n();
  let _ssr_site = expect_context::<Resource<Result<GetSiteResponse, LemmyAppError>>>();
  let param = use_params_map();
  let ssr_name = move || param.get().get("name").unwrap_or("".into());

  let query = use_query_map();

  let ssr_page = move || serde_json::from_str::<Vec<u32>>(&query.get().get("page").unwrap_or("".into())).unwrap_or(vec![1u32]);

  let next_page_cursor: RwSignal<u32> = RwSignal::new(0);

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
      if let Some(se) = on_scroll_element.get() {
        #[cfg(not(feature = "ssr"))]
        spawn_local_scoped_with_cancellation(async move {
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
            st.push(key as u32);
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

  let user_resource = Resource::new(
    move || ssr_name(),
    move |name| async move {
      let form = GetPersonDetails {
        username: Some(name),
        saved_only: None,
        community_id: None,
        limit: None,
        page: None,
        person_id: None,
        sort: Some(SortType::New),
      };
      let result = match LemmyClient.get_user(form.clone()).await {
        Ok(o) => Ok(Some(o)),
        Err(e) => Err(e),
      };
      result
    },
  );

  let now_in_millis = RwSignal::new(u64::try_from(jiff::Zoned::now().timestamp().as_millisecond()).unwrap_or(0));
  // let now_in_millis = RwSignal::new({
  //   #[cfg(not(feature = "ssr"))]
  //   {
  //     chrono::offset::Utc::now().timestamp_millis() as u64
  //   }
  //   #[cfg(feature = "ssr")]
  //   {
  //     std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or(std::time::Duration::new(1000, 0)).as_millis() as u64
  //   }
  // });

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
            {move || {
              match user_resource.get() {
                Some(Err(e)) => view! { <Error error={e} on_retry_click={None::<fn(MouseEvent) -> ()>} /> }.into_any(),
                Some(Ok(Some(s))) => {
                  let t = s.clone();
                  let name = s.person_view.person.name;
                  let banner = Memo::new(move |_| s.person_view.person.banner.clone());
                  let avatar = Memo::new(move |_| s.person_view.person.avatar.clone());
                  let all_posts = RwSignal::new(
                    s
                      .posts
                      .iter()
                      .map(|p| PostWithComments {
                        post: p.clone(),
                        comments: RwSignal::new(Vec::new()),
                      })
                      .collect::<Vec<_>>(),
                  );
                  let comments = s.comments.clone();
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
                                published: std::time::SystemTime::now().into(),
                                newest_comment_time_necro: std::time::SystemTime::now().into(),
                                newest_comment_time: std::time::SystemTime::now().into(),
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
                  let bio = if let Some(bio) = s.person_view.person.bio {
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

                    <For each={move || all_posts.get()} key={|pc| pc.post.post.id} let:pc>
                      <div class="pt-4 odd:bg-base-200">
                        <Listing hide=false post_view={pc.post} post_number=0 /*reply_show={RwSignal::new(false)}*/ />
                        <For each={move || pc.comments.get()} key={|cv| cv.comment.id} let:cv>
                          <div class="pt-2 pr-4 pb-4 pl-8">
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
                  }
                    .into_any()
                }
                _ => view! {}.into_any(),
              }
            }}
          </Transition>
        </div>
      </div>
    </main>
  }
}
