use crate::{errors::AosAppError, i18n::*, lemmy_client::*};
use lemmy_api_common::{
  community::*,
  lemmy_db_schema::{ListingType, SortType},
  lemmy_db_views_actor::structs::CommunityView,
};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Trending() -> impl IntoView {
  let _i18n = use_i18n();

  let error = expect_context::<RwSignal<Vec<Option<(AosAppError, Option<RwSignal<bool>>)>>>>();

  let trending = Resource::new(
    move || (),
    move |()| async move {
      let form = ListCommunities {
        type_: Some(ListingType::Local),
        sort: Some(SortType::Hot),
        limit: Some(6),
        show_nsfw: None,
        page: None,
      };

      let result = LemmyClient.list_communities(form).await;

      match result {
        Ok(o) => Some(o),
        Err(e) => {
          error.update(|es| es.push(Some((e, None))));
          None
        }
      }
    },
  );

  view! {
    <Transition fallback={|| {
      view! {}.into_any()
    }}>
      {move || {
        trending
          .get()
          .map(|r| match r {
            None => view! {}.into_any(),
            Some(c) => {
              let c_signal = RwSignal::new(c.communities);
              // Some(
              view! {
                <div class="mb-0 w-full card bg-base-300 text-base-content">
                  <figure>
                    <div class="card-body bg-info">
                      <h2 class="card-title text-info-content">"Trending Communities"</h2>
                    </div>
                  </figure>
                  <div class="card-body">
                    <p>
                      <For
                        each={move || c_signal.get()}
                        key={|community| community.community.id}
                        children={move |cv: CommunityView| {
                          view! {
                            <A attr:class="block mb-1 font-bold no-underline text-l link link-accent" href={format!("/c/{}", cv.community.name)}>
                              {cv.community.title}
                            </A>
                          }
                        }}
                      />
                    </p>
                    <A attr:class="btn" href="/create_community">
                      "Create a community"
                    </A>
                    <A attr:class="btn btn-disabled" href="/communities">
                      "Explore communities"
                    </A>
                  </div>
                </div>
              }
                .into_any()
            }
          })
      }}
    </Transition>
  }
  .into_any()
}
