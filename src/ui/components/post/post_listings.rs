use crate::{errors::LemmyAppError, ui::components::post::post_listing::PostListing};
use lemmy_api_common::{lemmy_db_views::structs::PostView, site::GetSiteResponse};
use leptos::*;

#[component]
pub fn PostListings(
  posts: MaybeSignal<Vec<PostView>>,
  ssr_site: Resource<(Option<String>, Option<String>), Result<GetSiteResponse, LemmyAppError>>,
  page_number: RwSignal<usize>,
  #[prop(into)] on_community_change: Callback<String, ()>,
) -> impl IntoView {
  let post_number = RwSignal::new(page_number.get());
  view! {
    <For each={move || posts.get()} key={|pv| pv.post.id} let:pv>
      {
        post_number.set(post_number.get() + 1);
        view! { <PostListing post_view={pv.into()} ssr_site post_number={post_number.get()} reply_show={false.into()} on_community_change /> }
      }
    </For>
  }
}
