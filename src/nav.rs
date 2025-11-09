use crate::{
  client::*,
  db::csr_indexed_db::*,
  errors::{message_from_error, LemmyAppError, LemmyAppResult},
  icon::{IconType::*, *},
  NotificationsRefresh,
  OnlineSetter,
  ReadInstanceCookie,
  ResourceStatus,
  ResponseLoad,
  WriteAuthCookie,
  WriteInstanceCookie,
  WriteThemeCookie,
};
use lemmy_api_common::{
  lemmy_db_schema::{source::site::Site, ListingType, SortType},
  lemmy_db_views::structs::{PaginationCursor, SiteView},
  person::GetUnreadCountResponse,
  post::{GetPostResponse, GetPosts, GetPostsResponse},
  site::GetSiteResponse,
};
use leptos::{html::Div, logging::log, prelude::*, server::codee::string::FromToStringCodec, task::spawn_local_scoped_with_cancellation, *};
use leptos_router::{components::*, hooks::*, *};
use leptos_use::{use_cookie_with_options, SameSite, UseCookieOptions, *};
use std::collections::BTreeMap;
use web_sys::{KeyboardEvent, MouseEvent, SubmitEvent, VisibilityState};

#[server(LogoutFn, "/serverfn")]
pub async fn logout() -> Result<(), ServerFnError> {
  use leptos_axum::redirect;
  let result = LemmyClient.logout().await;
  match result {
    Ok(_o) => {
      let WriteAuthCookie(set_auth_cookie) = expect_context::<WriteAuthCookie>();
      set_auth_cookie.set(None);
      Ok(())
    }
    Err(e) => {
      redirect(&format!("/login?error={}", serde_json::to_string(&e)?)[..]);
      Ok(())
    }
  }
}

#[server(SearchFn, "/serverfn")]
pub async fn search(term: String) -> Result<(), ServerFnError> {
  use leptos_axum::redirect;
  redirect(&format!("/s?term={}", &term));
  Ok(())
}

#[server(InstanceFn, "/serverfn")]
pub async fn instance(instance: String) -> Result<(), ServerFnError> {
  let WriteInstanceCookie(set_instance_cookie) = expect_context::<WriteInstanceCookie>();
  if instance.len() > 0 {
    set_instance_cookie.set(Some(instance));
  } else {
    set_instance_cookie.set(Some("lemmy.world".to_string()));
  }
  Ok(())
}

#[server(ChangeLangFn, "/serverfn")]
pub async fn change_lang(lang: String) -> Result<(), ServerFnError> {
  let (_, set_locale_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
    "i18n_pref_locale",
    UseCookieOptions::default().max_age(691200000).path("/").same_site(SameSite::Lax),
  );
  set_locale_cookie.set(Some(lang.to_lowercase()));
  Ok(())
}

#[server]
pub async fn change_theme(theme: String) -> Result<(), ServerFnError> {
  let (_, set_theme_cookie) =
    use_cookie_with_options::<String, FromToStringCodec>("theme", UseCookieOptions::default().max_age(691200000).path("/").same_site(SameSite::Lax));
  log!("{}", theme);
  set_theme_cookie.set(Some(theme));
  Ok(())
}

#[component]
pub fn TopNav(
  #[prop(optional)] default_sort: MaybeProp<SortType>,
  #[prop(optional)] post_view: MaybeSignal<Option<GetPostResponse>>,
) -> impl IntoView {
  // let i18n = use_i18n();

  let ssr_site_signal = expect_context::<RwSignal<Option<Result<GetSiteResponse, LemmyAppError>>>>();
  let WriteThemeCookie(set_theme_cookie) = expect_context::<WriteThemeCookie>();
  let online = expect_context::<RwSignal<OnlineSetter>>();
  let response_cache = expect_context::<RwSignal<BTreeMap<(usize, GetPosts, Option<bool>), (i64, LemmyAppResult<GetPostsResponse>)>>>();
  let scroll_element = expect_context::<RwSignal<Option<NodeRef<Div>>>>();
  let query = use_query_map();
  let ssr_query_error = move || {
    serde_json::from_str::<LemmyAppError>(&query.get().get("error").unwrap_or("".into()))
      .ok()
      .map(|e| (e, None::<Option<RwSignal<bool>>>))
  };
  let ssr_list = move || serde_json::from_str::<ListingType>(&query.get().get("list").unwrap_or("".into())).unwrap_or(ListingType::All);
  let ssr_sort =
    move || serde_json::from_str::<SortType>(&query.get().get("sort").unwrap_or("".into())).unwrap_or(default_sort.get().unwrap_or(SortType::Active));
  let logged_in = Signal::derive(move || {
    if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site_signal.get() {
      Some(true)
    } else {
      Some(false)
    }
  });

  let on_sort_click = move |s: SortType| {
    move |_e: MouseEvent| {
      let r = serde_json::to_string::<SortType>(&s);
      let mut query_params = query.get();
      match r {
        Ok(o) => {
          query_params.insert("sort".to_string(), o);
        }
        Err(e) => {}
      }
      if default_sort.get().unwrap_or(SortType::Active) == s {
        query_params.remove("sort".into());
      }
      query_params.remove("page".into());
      let navigate = use_navigate();
      if let Ok(Some(s)) = window().local_storage() {
        let _ = s.set_item(&format!("{}{}", use_location().pathname.get(), query_params.to_query_string()), "0");
      }

      response_cache.update(move |rc| {
        rc.remove(&(
          0usize,
          GetPosts {
            type_: Some(ssr_list()),
            sort: Some(s),
            page: None,
            limit: Some(50),
            community_id: None,
            community_name: None,
            saved_only: None,
            liked_only: None,
            disliked_only: None,
            show_hidden: Some(true),
            show_read: Some(true),
            show_nsfw: Some(false),
            page_cursor: None,
          },
          logged_in.get(),
        ));
      });

      navigate(
        &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
        Default::default(),
      );
    }
  };

  let on_csr_filter_click = move |l: ListingType| {
    move |_e: MouseEvent| {
      let mut query_params = query.get();
      query_params.remove("page".into());
      let navigate = use_navigate();
      if l == ListingType::All {
        query_params.remove("list".into());
      } else {
        query_params.insert("list".to_string(), serde_json::to_string(&l).ok().unwrap());
      }
      if let Ok(Some(s)) = window().local_storage() {
        let _ = s.set_item(&format!("{}{}", use_location().pathname.get(), query_params.to_query_string()), "0");
      }
      response_cache.update(move |rc| {
        rc.remove(&(
          0usize,
          GetPosts {
            type_: Some(l),
            sort: Some(ssr_sort()),
            page: None,
            limit: Some(50),
            community_id: None,
            community_name: None,
            saved_only: None,
            liked_only: None,
            disliked_only: None,
            show_hidden: Some(true),
            show_read: Some(true),
            show_nsfw: Some(false),
            page_cursor: None,
          },
          logged_in.get(),
        ));
      });

      navigate(
        &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
        Default::default(),
      );
    }
  };

  let highlight_csr_filter = move |l: ListingType| {
    if l == ssr_list() {
      "btn-active"
    } else {
      ""
    }
  };

  let notifications_refresh = expect_context::<RwSignal<NotificationsRefresh>>();

  let logout_action = ServerAction::<LogoutFn>::new();

  let search_show = RwSignal::new(false);
  let still_pressed = create_rw_signal(false);

  #[cfg(not(feature = "ssr"))]
  let visibility = expect_context::<Signal<VisibilityState>>();

  #[cfg(not(feature = "ssr"))]
  let _visibility_effect = Effect::new(move |_| match visibility.get() {
    VisibilityState::Visible => {}
    _ => {}
  });

  let on_logout_submit = move |e: MouseEvent| {
    e.prevent_default();
    spawn_local_scoped_with_cancellation(async move {
      let result = LemmyClient.logout().await;
      match result {
        Ok(_o) => {
          let WriteAuthCookie(set_auth_cookie) = expect_context::<WriteAuthCookie>();
          set_auth_cookie.set(None);
          ssr_site_signal.update(|s| {
            if let Some(Ok(t)) = s {
              t.my_user = None;
            }
          });
        }
        Err(e) => {}
      }
    });
  };

  let search_action = ServerAction::<SearchFn>::new();
  let search_term = RwSignal::new("".to_string());

  let display_title = Signal::derive(move || {
    let s = if let Some(pv) = post_view.get() {
      let community_title = if pv.post_view.community.local {
        format!("{}", pv.post_view.community.name)
      } else {
        format!(
          "{}@{}",
          pv.post_view.community.name,
          pv.post_view.community.actor_id.inner().host().unwrap().to_string()
        )
      };
      format!(
        "{} by {} in {}",
        pv.post_view.post.name,
        pv.post_view.creator.actor_id.to_string()[8..].to_string(),
        community_title
      )
    } else {
      "".to_string()
    };
    search_term.set(s.clone());
    s
  });

  let on_search_submit = move |e: SubmitEvent| {
    e.prevent_default();
    use_navigate()(&format!("/s?term={}", search_term.get()), NavigateOptions::default());
  };

  let ReadInstanceCookie(get_instance_cookie) = expect_context::<ReadInstanceCookie>();
  let WriteInstanceCookie(set_instance_cookie) = expect_context::<WriteInstanceCookie>();

  let instance_term = RwSignal::new(get_instance_cookie.get());
  let instance_action = ServerAction::<InstanceFn>::new();
  let on_instance_submit = move |e: SubmitEvent| {
    e.prevent_default();
    still_pressed.set(false);
    if instance_term.get().unwrap_or("".into()).len() > 0 {
      set_instance_cookie.set(instance_term.get());
    } else {
      set_instance_cookie.set(Some("lemmy.world".to_string()));
    }
    if let Some(on_scroll_element) = scroll_element.get() {
      if let Some(se) = on_scroll_element.get() {
        se.set_scroll_left(0i32);
      }
    }
    response_cache.update(move |rc| {
      rc.remove(&(
        0usize,
        GetPosts {
          type_: Some(ListingType::All),
          sort: Some(SortType::Active),
          page: None,
          limit: Some(50),
          community_id: None,
          community_name: None,
          saved_only: None,
          liked_only: None,
          disliked_only: None,
          show_hidden: Some(true),
          show_read: Some(true),
          show_nsfw: Some(false),
          page_cursor: None,
        },
        logged_in.get(),
      ));
    });
    let mut query_params = query.get();
    query_params.remove("page".into());
    let navigate = use_navigate();
    navigate(
      &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
      Default::default(),
    );
  };

  let logged_in = Signal::derive(move || {
    if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site_signal.get() {
      Some(true)
    } else {
      Some(false)
    }
  });

  let online = expect_context::<RwSignal<OnlineSetter>>();
  let change_theme = ServerAction::<ChangeTheme>::new();

  let on_theme_submit = move |theme_name: &'static str| {
    move |e: MouseEvent| {
      e.prevent_default();
      set_theme_cookie.set(Some(theme_name.to_string()));
    }
  };

  let lang_action = ServerAction::<ChangeLangFn>::new();

  // let on_lang_submit = move |lang: Locale| {
  //   move |ev: SubmitEvent| {
  //     ev.prevent_default();
  //     i18n.set_locale(lang);
  //   }
  // };

  let on_navigate_login = move |e: MouseEvent| {
    e.prevent_default();
    use_navigate()("/l", NavigateOptions::default());
  };

  let instance_timer_handle = create_rw_signal(None);

  let on_pointer_down = {
    let timer_id = instance_timer_handle.clone();
    move |_| {
      let timer = set_timeout_with_handle(
        move || {
          still_pressed.set(true);
        },
        std::time::Duration::from_millis(500),
      )
      .ok();
      timer_id.set(timer);
    }
  };

  let on_pointer_up = {
    let timer_id = instance_timer_handle.clone();
    move |_| {
      if let Some(timer) = timer_id.get() {
        timer.clear();
      }
      timer_id.set(None);
    }
  };

  view! {
    <nav class="flex flex-row py-0 navbar">
      <div class={move || { (if search_show.get() { "hidden" } else { "flex" }).to_string() }}>
        <ActionForm attr:class={move || { if still_pressed.get() { "" } else { "hidden" } }} action={instance_action} on:submit={on_instance_submit}>
          <input
            class="pl-6 w-40 text-xl input"
            type="text"
            name="instance"
            prop:value={move || instance_term.get()}
            on:input={move |ev| {
              instance_term.set(Some(event_target_value(&ev)));
            }}
          />
        </ActionForm>
        <ul class="flex-nowrap items-center menu menu-horizontal">
          <li>
            <A
              href="/"
              attr:class={move || { if still_pressed.get() { "hidden" } else { "select-none text-xl whitespace-nowrap py-1/2" } }}
              on:pointerdown={on_pointer_down}
              on:pointerup={on_pointer_up}
              on:pointerleave={on_pointer_up}
              on:click={move |e: MouseEvent| {
                if still_pressed.get() {
                  still_pressed.set(false);
                  e.prevent_default();
                } else {
                  #[cfg(not(feature = "ssr"))]
                  spawn_local_scoped_with_cancellation(async move {
                    if let Ok(d) = IndexedDb::new().await {
                      d.set(
                          &ScrollPositionKey {
                            path: "/".into(),
                            query: "".into(),
                          },
                          &0i32,
                        )
                        .await;
                    }
                  });
                  if let Some(on_scroll_element) = scroll_element.get() {
                    if let Some(se) = on_scroll_element.get() {
                      se.set_scroll_left(0i32);
                    }
                  }
                  response_cache
                    .update(move |rc| {
                      rc.remove(
                        &(
                          0usize,
                          GetPosts {
                            type_: Some(ListingType::All),
                            sort: Some(SortType::Active),
                            page: None,
                            limit: Some(50),
                            community_id: None,
                            community_name: None,
                            saved_only: None,
                            liked_only: None,
                            disliked_only: None,
                            show_hidden: Some(true),
                            show_read: Some(true),
                            show_nsfw: Some(false),
                            page_cursor: None,
                          },
                          logged_in.get(),
                        ),
                      );
                    });
                }
              }}
            >
              {move || {
                if let Some(Ok(GetSiteResponse { site_view: SiteView { site: Site { icon: Some(i), .. }, .. }, .. })) = ssr_site_signal.get() {
                  view! { <img class="h-8 sm:hidden" src={i.inner().to_string()} /> }.into_any()
                } else {
                  view! { <img class="h-8" src="/favicon.png" /> }.into_any()
                }
              }}
              <span class="hidden sm:flex">
                {move || { if let Some(Ok(m)) = ssr_site_signal.get() { m.site_view.site.name } else { "A.O.S".to_string() } }}
              </span>
            </A>
          </li>
          <li class="hidden sm:flex z-[1]">
            <details>
              <summary>
                <Icon icon={Community} />
              </summary>
              <ul tabindex="0" class="shadow menu dropdown-content z-[1] bg-base-100 rounded-box">
                <li class={move || highlight_csr_filter(ListingType::All)} on:click={on_csr_filter_click(ListingType::All)}>
                  <span>"All"</span>
                </li>
                <li class={move || highlight_csr_filter(ListingType::Local)} on:click={on_csr_filter_click(ListingType::Local)}>
                  <span>"Local"</span>
                </li>
                <li
                  class={move || {
                    format!(
                      "{}{}",
                      highlight_csr_filter(ListingType::Subscribed),
                      if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site_signal.get() { "" } else { " btn-disabled" },
                    )
                  }}
                  on:click={on_csr_filter_click(ListingType::Subscribed)}
                >
                  <span>"Subscribed"</span>
                </li>
              </ul>
            </details>
          </li>
          <li class="hidden sm:flex z-[1]">
            <details>
              <summary>
                <Icon icon={Sort} />
              </summary>
              <ul tabindex="0" class="shadow menu dropdown-content z-[1] bg-base-100 rounded-box">
                <li
                  class={move || { (if SortType::Active == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                  on:click={on_sort_click(SortType::Active)}
                >
                  <span>"Active"</span>
                </li>
                <li
                  class={move || { (if SortType::TopAll == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                  on:click={on_sort_click(SortType::TopAll)}
                >
                  <span>"Top"</span>
                </li>
                <li
                  class={move || { (if SortType::Hot == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                  on:click={on_sort_click(SortType::Hot)}
                >
                  <span>"Hot"</span>
                </li>
                <li
                  class={move || { (if SortType::New == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                  on:click={on_sort_click(SortType::New)}
                >
                  <span>"New"</span>
                </li>
                <li
                  class={move || { (if SortType::Old == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                  on:click={on_sort_click(SortType::Old)}
                >
                  <span>"Old"</span>
                </li>
                <li
                  class={move || { (if SortType::Controversial == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                  on:click={on_sort_click(SortType::Controversial)}
                >
                  <span>"Controversial"</span>
                </li>
                <li
                  class={move || { (if SortType::Scaled == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                  on:click={on_sort_click(SortType::Scaled)}
                >
                  <span>{"Scaled"}</span>
                </li>
              </ul>
            </details>
          </li>
          <li class="flex sm:hidden">
            <details>
              <summary>
                <Icon icon={Filter} />
              </summary>
              <ul class="z-[1]">
                <li class="flex z-[1]">
                  <details>
                    <summary>
                      <Icon icon={Community} />
                    </summary>
                    <ul tabindex="0" class="shadow menu dropdown-content z-[1] bg-base-100 rounded-box">
                      <li class={move || highlight_csr_filter(ListingType::All)} on:click={on_csr_filter_click(ListingType::All)}>
                        <span>"All"</span>
                      </li>
                      <li class={move || highlight_csr_filter(ListingType::Local)} on:click={on_csr_filter_click(ListingType::Local)}>
                        <span>"Local"</span>
                      </li>
                      <li
                        class={move || {
                          format!(
                            "{}{}",
                            highlight_csr_filter(ListingType::Subscribed),
                            if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site_signal.get() { "" } else { " btn-disabled" },
                          )
                        }}
                        on:click={on_csr_filter_click(ListingType::Subscribed)}
                      >
                        <span>"Subscribed"</span>
                      </li>
                    </ul>
                  </details>
                </li>
                <li class="flex z-[1]">
                  <details>
                    <summary>
                      <Icon icon={Sort} />
                    </summary>
                    <ul tabindex="0" class="shadow menu dropdown-content z-[1] bg-base-100 rounded-box">
                      <li
                        class={move || { (if SortType::Active == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                        on:click={on_sort_click(SortType::Active)}
                      >
                        <span>"Active"</span>
                      </li>
                      <li
                        class={move || { (if SortType::TopAll == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                        on:click={on_sort_click(SortType::TopAll)}
                      >
                        <span>"Top"</span>
                      </li>
                      <li
                        class={move || { (if SortType::Hot == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                        on:click={on_sort_click(SortType::Hot)}
                      >
                        <span>"Hot"</span>
                      </li>
                      <li
                        class={move || { (if SortType::New == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                        on:click={on_sort_click(SortType::New)}
                      >
                        <span>"New"</span>
                      </li>
                      <li
                        class={move || { (if SortType::Old == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                        on:click={on_sort_click(SortType::Old)}
                      >
                        <span>"Old"</span>
                      </li>
                      <li
                        class={move || { (if SortType::Controversial == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                        on:click={on_sort_click(SortType::Controversial)}
                      >
                        <span>"Controversial"</span>
                      </li>
                      <li
                        class={move || { (if SortType::Scaled == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                        on:click={on_sort_click(SortType::Scaled)}
                      >
                        <span>{"Scaled"}</span>
                      </li>
                    </ul>
                  </details>
                </li>
              </ul>
            </details>
          </li>

        </ul>

      </div>
      <div class="flex flex-grow">
        <ActionForm
          attr:class={move || {
            (if search_show.get() { "form-control flex flex-grow" } else { "form-control hidden sm:flex flex-grow" }).to_string()
          }}
          action={search_action}
        >
          <input
            title={move || display_title.get()}
            class="w-full input"
            type="text"
            name="term"
            prop:value={move || display_title.get()}
            on:keypress={move |e: KeyboardEvent| {
              if e.key() == "Enter" {
                e.prevent_default();
                use_navigate()(&format!("/s?term={}", search_term.get()), NavigateOptions::default());
              }
            }}
            on:input={move |ev| {
              search_term.set(event_target_value(&ev));
            }}
          />
        </ActionForm>
      </div>
      <div class="flex-none">
        <button
          class="py-2 px-4"
          on:click={move |_| {
            search_show
              .update(|b| {
                *b = !*b;
              })
          }}
        >
          <Icon icon={Search} />
        </button>
      </div>
      <div class={move || { (if search_show.get() { "hidden" } else { "flex-none" }).to_string() }}>
        <ul class="flex-nowrap items-center menu menu-horizontal">
          <li class="hidden sm:flex">
            <details>
              <summary>
                <Icon icon={Translate} />
              </summary>
              <ul class="z-[1] [inset-inline-end:0]">
                <li>
                  <ActionForm attr:class="p-0" action={lang_action}>
                    <input type="hidden" name="lang" value="FR" />
                    <button class="py-2 px-4" type="submit">
                      "FR"
                    </button>
                  </ActionForm>
                </li>
                <li>
                  <ActionForm attr:class="p-0" action={lang_action}>
                    <input type="hidden" name="lang" value="EN" />
                    <button class="py-2 px-4" type="submit">
                      "EN"
                    </button>
                  </ActionForm>
                </li>
              </ul>
            </details>
          </li>
          <li class="hidden sm:flex">
            <details>
              <summary>
                <Icon icon={Palette} />
              </summary>
              <ul class="z-[1] [inset-inline-end:0]">
                <li>
                  <ActionForm attr:class="p-0" action={change_theme}>
                    <input type="hidden" name="theme" value="dark" />
                    <button class="py-2 px-4" type="submit" on:click={on_theme_submit("dark")}>
                      "Dark"
                    </button>
                  </ActionForm>
                </li>
                <li>
                  <ActionForm attr:class="p-0" action={change_theme}>
                    <input type="hidden" name="theme" value="light" />
                    <button class="py-2 px-4" type="submit" on:click={on_theme_submit("light")}>
                      "Light"
                    </button>
                  </ActionForm>
                </li>
                <li>
                  <ActionForm attr:class="p-0" action={change_theme}>
                    <input type="hidden" name="theme" value="retro" />
                    <button class="py-2 px-4" type="submit" on:click={on_theme_submit("retro")}>
                      "Retro"
                    </button>
                  </ActionForm>
                </li>
              </ul>
            </details>
          </li>
          <Show
            when={move || { if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site_signal.get() { true } else { false } }}
            fallback={move || {
              view! {
                <li>
                  <form class="p-0" action="/l" method="POST" on:click={on_navigate_login}>
                    <button class="py-2 px-4" type="submit">
                      <Icon icon={SignIn} />
                    </button>
                  </form>
                </li>
              }
            }}
          >
            <li>
              <details>
                <summary>
                  <Icon icon={User} />
                </summary>
                <ul class="z-[1] [inset-inline-end:0]">
                  <li class="flex sm:hidden">
                    <details>
                      <summary>
                        <Icon icon={Palette} />
                      </summary>
                      <ul class="z-[1] [inset-inline-end:0]">
                        <li>
                          <ActionForm attr:class="p-0" action={change_theme}>
                            <input type="hidden" name="theme" value="dark" />
                            <button class="py-2 px-4" type="submit" on:click={on_theme_submit("dark")}>
                              "Dark"
                            </button>
                          </ActionForm>
                        </li>
                        <li>
                          <ActionForm attr:class="p-0" action={change_theme}>
                            <input type="hidden" name="theme" value="light" />
                            <button class="py-2 px-4" type="submit" on:click={on_theme_submit("light")}>
                              "Light"
                            </button>
                          </ActionForm>
                        </li>
                        <li>
                          <ActionForm attr:class="p-0" action={change_theme}>
                            <input type="hidden" name="theme" value="retro" />
                            <button class="py-2 px-4" type="submit" on:click={on_theme_submit("retro")}>
                              "Retro"
                            </button>
                          </ActionForm>
                        </li>
                      </ul>
                    </details>
                  </li>
                  <div class="flex my-0 sm:hidden divider" />
                  <li>
                    <A href="/notifications">"Notifications"</A>
                  </li>
                  <li>
                    <A
                      on:click={move |e: MouseEvent| {
                        if e.ctrl_key() && e.shift_key() {
                          e.stop_propagation();
                          if let Some(Ok(GetSiteResponse { my_user: Some(m), .. })) = ssr_site_signal.get() {
                            let _ = window().location().set_href(&format!("//lemmy.world/u/{}", m.local_user_view.person.name));
                          }
                        }
                      }}
                      href={move || {
                        format!(
                          "/u/{}",
                          if let Some(Ok(GetSiteResponse { my_user: Some(m), .. })) = ssr_site_signal.get() {
                            m.local_user_view.person.name
                          } else {
                            String::default()
                          },
                        )
                      }}
                    >
                      "Profile"
                    </A>
                  </li>
                  <li>
                    <A attr:class="pointer-events-none text-base-content/50" href="/settings">
                      "Settings"
                    </A>
                  </li>
                  <div class="my-0 divider" />
                  <li>
                    <ActionForm action={logout_action}>
                      <button type="submit" on:click={on_logout_submit}>
                        "Logout"
                      </button>
                    </ActionForm>
                  </li>
                </ul>
              </details>
            </li>
          </Show>
        </ul>
      </div>
    </nav>
  }
}
