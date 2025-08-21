use crate::{
  errors::LemmyAppError,
  ui::components::{post::post_listing::PostListing, responsive::responsive_post_listing::ResponsivePostListing},
};
use lemmy_api_common::{lemmy_db_views::structs::PostView, site::GetSiteResponse};
use leptos::*;

#[component]
pub fn ResponsivePostListings(
  posts: MaybeSignal<Vec<PostView>>,
  ssr_site: Resource<Option<String>, Result<GetSiteResponse, LemmyAppError>>,
  page_number: RwSignal<usize>,
) -> impl IntoView {
  let post_number = RwSignal::new(page_number.get());
  view! {
    <For each={move || posts.get()} key={|pv| pv.post.id} let:pv>
      {
        post_number.set(post_number.get() + 1);
        view! { <ResponsivePostListing post_view={pv.into()} ssr_site post_number={post_number.get()} reply_show={false.into()} /> }
      }
    </For>
  }
}
