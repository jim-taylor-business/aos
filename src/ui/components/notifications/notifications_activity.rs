use crate::{
  errors::{message_from_error, LemmyAppError, LemmyAppErrorType},
  ui::components::{comment::comment_node::CommentNode, common::about::About, post::post_listing::PostListing},
  LemmyApi,
  LemmyClient,
  NotificationsRefresh, // TitleSetter,
};
use ev::MouseEvent;
use lemmy_api_common::{
  lemmy_db_schema::{
    aggregates::structs::PostAggregates,
    newtypes::{CommentReplyId, InstanceId},
    source::{instance::Instance, person::Person},
    CommentSortType, SubscribedType,
  },
  lemmy_db_views::structs::{CommentView, PostView},
  person::{GetPersonMentions, GetReplies, MarkCommentReplyAsRead},
  private_message::GetPrivateMessages,
  site::GetSiteResponse,
};
use leptos::*;
use leptos_meta::*;

#[component]
pub fn NotificationsActivity(ssr_site: Resource<Option<bool>, Result<GetSiteResponse, LemmyAppError>>) -> impl IntoView {
  let errors = expect_context::<RwSignal<Vec<Option<(LemmyAppError, Option<RwSignal<bool>>)>>>>();
  let notifications_refresh = expect_context::<RwSignal<NotificationsRefresh>>();
  // let title = expect_context::<RwSignal<Option<TitleSetter>>>();

  // title.set(Some(TitleSetter("Notifications".into())));

  let replies_refresh = RwSignal::new(true);

  let replies = Resource::new(
    move || (replies_refresh.get()),
    move |_replies_refresh| async move {
      let form = GetReplies {
        sort: Some(CommentSortType::New),
        page: Some(1),
        limit: Some(10),
        unread_only: Some(true),
      };
      let result = LemmyClient.replies_user(form).await;
      match result {
        Ok(o) => Some(o),
        Err(e) => {
          errors.update(|es| es.push(Some((e, None))));
          None
        }
      }
    },
  );

  let mentions = Resource::new(
    move || (),
    move |()| async move {
      let form = GetPersonMentions {
        sort: Some(CommentSortType::New),
        page: Some(1),
        limit: Some(10),
        unread_only: Some(true),
      };
      let result = LemmyClient.mention_user(form).await;
      match result {
        Ok(o) => Some(o),
        Err(e) => {
          errors.update(|es| es.push(Some((e, None))));
          None
        }
      }
    },
  );

  let messages = Resource::new(
    move || (),
    move |()| async move {
      let form = GetPrivateMessages {
        page: Some(1),
        limit: Some(10),
        unread_only: Some(true),
        creator_id: None,
      };
      let result = LemmyClient.messages_user(form).await;
      match result {
        Ok(o) => Some(o),
        Err(e) => {
          errors.update(|es| es.push(Some((e, None))));
          None
        }
      }
    },
  );

  let now_in_millis = {
    #[cfg(not(feature = "ssr"))]
    {
      chrono::offset::Utc::now().timestamp_millis() as u64
    }
    #[cfg(feature = "ssr")]
    {
      std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64
    }
  };

  let on_hide_show = |_| {};

  let on_clear_reply_click = move |id: CommentReplyId| {
    move |_e: MouseEvent| {
      create_local_resource(
        move || (),
        move |()| async move {
          let form = MarkCommentReplyAsRead {
            comment_reply_id: id,
            read: true,
          };
          let result = LemmyClient.mark_comment(form).await;
          match result {
            Ok(_o) => {
              replies_refresh.update(|b| *b = !*b);
              notifications_refresh.update(|n| n.0 = !n.0);
            }
            Err(e) => {
              errors.update(|es| es.push(Some((e, None))));
            }
          }
        },
      );
    }
  };

  view! {
    <main class="mx-auto">
      <Title text="Notifications" />
      <Transition fallback={|| {}}>
        {move || {
          replies
            .get()
            .unwrap_or(None)
            .map(|g| {
              view! {
                <div class="w-full">
                  <For each={move || g.replies.clone()} key={|r| r.comment.id} let:r>
                    {
                      let c = CommentView {
                        comment: r.comment,
                        creator: r.creator.clone(),
                        post: r.post.clone(),
                        community: r.community.clone(),
                        counts: r.counts,
                        creator_banned_from_community: false,
                        creator_is_moderator: r.creator_is_moderator,
                        creator_is_admin: r.creator_is_admin,
                        subscribed: r.subscribed,
                        saved: r.saved,
                        creator_blocked: r.creator_blocked,
                        my_vote: r.my_vote,
                      };
                      let p = PostView {
                        post: r.post.clone(),
                        creator: r.creator.clone(),
                        community: r.community.clone(),
                        creator_banned_from_community: false,
                        creator_is_moderator: false,
                        creator_is_admin: false,
                        counts: PostAggregates {
                          post_id: r.post.id,
                          comments: 0,
                          score: 0,
                          upvotes: 0,
                          downvotes: 0,
                          published: chrono::offset::Utc::now(),
                          newest_comment_time_necro: chrono::offset::Utc::now(),
                          newest_comment_time: chrono::offset::Utc::now(),
                          featured_community: false,
                          featured_local: false,
                          hot_rank: 0f64,
                          hot_rank_active: 0f64,
                          community_id: r.community.id,
                          creator_id: r.creator.id,
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
                      };
                      view! {
                        <div class="mb-6">
                          <PostListing
                            post_number=0
                            reply_show={RwSignal::new(false)}
                            ssr_site={Resource::new(
                              move || { None },
                              move |_b| async move {
                                Err(LemmyAppError {
                                  error_type: LemmyAppErrorType::Unknown,
                                  content: "".to_string(),
                                })
                              },
                            )}
                            post_view={p.into()}
                          />
                          <CommentNode
                            ssr_site
                            parent_comment_id=0
                            hidden_comments={RwSignal::new(vec![])}
                            on_toggle={on_hide_show}
                            comment_view={c.into()}
                            comments={vec![].into()}
                            level=1
                            now_in_millis
                          />
                          <div class="ml-4">
                            <button class="btn btn-sm" on:click={on_clear_reply_click(r.comment_reply.id)}>
                              "Clear"
                            </button>
                          </div>
                        </div>
                      }
                    }
                  </For>
                </div>
              }
            })
        }}
      </Transition>
      <Transition fallback={|| {}}>
        {move || {
          mentions
            .get()
            .unwrap_or(None)
            .map(|m| {
              (m.mentions.len() > 0)
                .then(move || {
                  view! {
                    <div class="w-full">
                      <div class="px-8 mb-6">
                        <div class="alert">
                          <span>{m.mentions.len()} " mentions"</span>
                        </div>
                      </div>
                    </div>
                  }
                })
            })
        }}
      </Transition>
      <Transition fallback={|| {}}>
        {move || {
          messages
            .get()
            .unwrap_or(None)
            .map(|p| {
              (p.private_messages.len() > 0)
                .then(move || {
                  view! {
                    <div class="w-full">
                      <div class="px-8 mb-6">
                        <div class="alert">
                          <span>{p.private_messages.len()} " messages"</span>
                        </div>
                      </div>
                    </div>
                  }
                })
            })
        }}
      </Transition>
      {move || {
        errors
          .get()
          .into_iter()
          .enumerate()
          .map(|(i, error)| {
            error
              .map(|err| {
                view! {
                  <div class="px-8 mb-6">
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
                        <button
                          class="btn btn-sm"
                          on:click={move |_| {
                            errors
                              .update(|es| {
                                es.remove(i);
                              });
                          }}
                        >
                          "Clear"
                        </button>
                      </div>
                    </div>
                  </div>
                }
              })
          })
          .collect::<Vec<_>>()
      }}
      <div class="px-8 mb-6">
        <button
          class={move || format!("btn{}", if errors.get().len() > 0 { "" } else { " hidden" })}
          on:click={move |_| {
            errors.set(vec![]);
          }}
        >
          "Clear All Errors"
        </button>
      </div>
      <div class="mx-6 sm:mx-0">
        <About />
      </div>
    </main>
  }
}
