use crate::{errors::LemmyAppError, i18n::*, lemmy_client::*};
use lemmy_api_common::{
  community::*,
  lemmy_db_schema::{ListingType, SortType},
  lemmy_db_views::structs::PaginationCursor,
  lemmy_db_views_actor::structs::CommunityView,
  post::GetPosts,
};
use leptos::*;
use leptos_router::*;

#[component]
pub fn Page(from: (usize, Option<PaginationCursor>)) -> impl IntoView {
  let _i18n = use_i18n();

  let error = expect_context::<RwSignal<Vec<Option<(LemmyAppError, Option<RwSignal<bool>>)>>>>();

  let trending = Resource::new(
    move || (),
    move |()| async move {
      let form = GetPosts {
        type_: None,
        sort: None,
        community_name: None,
        community_id: None,
        page: None,
        limit: None,
        saved_only: None,
        disliked_only: None,
        liked_only: None,
        page_cursor: None,
        show_hidden: Some(true),
        show_nsfw: Some(false),
        show_read: Some(true),
      };

      let result = LemmyClient.list_posts(form.clone()).await;

      logging::log!("luu lu");

      result
    },
  );

  view! {
    <Transition fallback={|| { view! { <span> "Fall" </span> } }}>
    <span> { format!("tranny {:?}", from.1) } </span>
    {move || {
      // view! { <span> "move" </span> }
      match trending.get() {
        Some(Ok(o)) => {
          logging::log!("wuu wu");
          view! {
            <span> "Ok" { format!("{:#?}", o.next_page) } </span>
          }
        },
        Some(Err(e)) => {
          view! {
            <span> "Error" </span>
          }
        },
        _ => {
          view! {
            <span> "Error" </span>
          }
        }
      }
    }}
    </Transition>
  }
}
