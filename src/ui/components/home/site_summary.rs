use crate::{errors::LemmyAppError, i18n::*};
use lemmy_api_common::site::GetSiteResponse;
use leptos::*;

#[component]
pub fn SiteSummary(ssr_site: Resource<Option<bool>, Result<GetSiteResponse, LemmyAppError>>) -> impl IntoView {
  let _i18n = use_i18n();

  view! {
    {move || {
      ssr_site
        .get()
        .map(|o| match o {
          Ok(o) => {
            Some(
              view! {
                <div class="mb-6 w-full card bg-base-300 text-base-content">
                  <figure>
                    <div class="card-body bg-neutral">
                      <h2 class="card-title text-neutral-content">{o.site_view.site.name}</h2>
                    </div>
                  </figure>
                  <div class="card-body">
                    <p>{o.site_view.site.description}</p>
                    <p>
                      <span class="inline-block whitespace-nowrap badge badge-neutral">{o.site_view.counts.users_active_day} " user / day"</span>
                      " "
                      <span class="inline-block whitespace-nowrap badge badge-neutral">{o.site_view.counts.users_active_week} " users / week"</span>
                      " "
                      <span class="inline-block whitespace-nowrap badge badge-neutral">{o.site_view.counts.users_active_month} " users / month"</span>
                      " "
                      <span class="inline-block whitespace-nowrap badge badge-neutral">
                        {o.site_view.counts.users_active_half_year} " users / 6 months"
                      </span>
                      " "
                      <span class="inline-block whitespace-nowrap badge badge-neutral">{o.site_view.counts.users} " users"</span>
                      " "
                      <span class="inline-block whitespace-nowrap badge badge-neutral">{o.site_view.counts.communities} " Communities"</span>
                      " "
                      <span class="inline-block whitespace-nowrap badge badge-neutral">{o.site_view.counts.posts} " Posts"</span>
                      " "
                      <span class="inline-block whitespace-nowrap badge badge-neutral">{o.site_view.counts.comments} " Comments"</span>
                      " "
                      <span class="inline-block whitespace-nowrap badge badge-neutral">"Modlog"</span>
                    </p>
                    <h3 class="card-title">"Admins"</h3>
                    <p>
                      <For
                        each={move || o.admins.clone()}
                        key={|admin| admin.person.id}
                        children={move |a| {
                          view! {
                            <span class="inline-block whitespace-nowrap badge badge-neutral">{a.person.name}</span>
                            " "
                          }
                        }}
                      />

                    </p>
                  </div>
                </div>
              },
            )
          }
          _ => None,
        })
    }}
  }
}
