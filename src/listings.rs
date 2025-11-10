use crate::listing::Listing;
use lemmy_api_common::lemmy_db_views::structs::PostView;
use leptos::prelude::*;

#[component]
pub fn Listings(posts: Signal<Vec<PostView>>, page_number: RwSignal<usize>) -> impl IntoView {
  let post_number = RwSignal::new(page_number.get());
  view! {
    <For each={move || posts.get()} key={|pv| pv.post.id} let:pv>
      {
        post_number.set(post_number.get() + 1);
        view! { <Listing post_view={pv} post_number={post_number.get()} reply_show={RwSignal::new(false)} /> }
      }
    </For>
  }
  .into_any()
}
