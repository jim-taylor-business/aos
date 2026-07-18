use crate::{hero::Hero, listing::Listing};
use lemmy_api_common::lemmy_db_views::structs::PostView;
use leptos::{logging::log, prelude::*};

#[component]
pub fn Listings(posts: Signal<Vec<PostView>>, page_number: RwSignal<usize>, hide: bool) -> impl IntoView {
  let post_number = RwSignal::new(page_number.get());
  view! {
    <For each={move || posts.get()} key={|pv| pv.post.id} let:pv>
      {
        post_number.set(post_number.get() + 1);
        if post_number.get() < 2usize {
          view! {
            <Hero hide post_id={Signal::derive(move || pv.post.id)} post_number={post_number.get()} />
          }.into_any()
        } else {
          view! {
            <Listing hide post_view={pv} post_number={post_number.get()} reply_show={RwSignal::new(false)} />
          }.into_any()
        }
      }
    </For>
  }
  .into_any()
}
