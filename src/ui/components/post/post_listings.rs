use crate::{errors::AosAppError, ui::components::post::post_listing::PostListing};
use lemmy_api_common::{lemmy_db_views::structs::PostView, site::GetSiteResponse};
use leptos::prelude::*;

#[component]
pub fn PostListings(
  posts: MaybeSignal<Vec<PostView>>,
  ssr_site: Resource<Result<GetSiteResponse, AosAppError>>,
  page_number: RwSignal<usize>,
) -> impl IntoView {
  let post_number = RwSignal::new(page_number.get());
  view! {
    <For each={move || posts.get()} key={|pv| pv.post.id} let:pv>
      {
        post_number.set(post_number.get() + 1);
        view! { <PostListing post_view={pv.into()} ssr_site post_number={post_number.get()} reply_show={RwSignal::new(false)} /> }
      }
    </For>
  }
}
