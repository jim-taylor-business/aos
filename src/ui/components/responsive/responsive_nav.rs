use std::collections::BTreeMap;

use crate::{
  errors::{message_from_error, LemmyAppError, LemmyAppResult},
  i18n::*,
  lemmy_client::*,
  ui::components::common::icon::{Icon, IconType::*},
  NotificationsRefresh, OnlineSetter, ResourceStatus, ResponseLoad,
};
use codee::string::FromToStringCodec;
use ev::MouseEvent;
use lemmy_api_common::{
  lemmy_db_schema::{source::site::Site, ListingType, SortType},
  lemmy_db_views::structs::{PaginationCursor, SiteView},
  person::GetUnreadCountResponse,
  post::{GetPostResponse, GetPosts, GetPostsResponse},
  site::GetSiteResponse,
};
use leptos::{html::Div, logging::log, *};
use leptos_router::*;
use leptos_use::*;
use leptos_use::{use_cookie_with_options, SameSite, UseCookieOptions};
use web_sys::{KeyboardEvent, SubmitEvent};

#[cfg(not(feature = "ssr"))]
use web_sys::VisibilityState;

#[server(LogoutFn, "/serverfn")]
pub async fn logout() -> Result<(), ServerFnError> {
  use leptos_actix::redirect;
  let result = LemmyClient.logout().await;
  match result {
    Ok(_o) => {
      let (_, set_auth_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
        "jwt",
        UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax),
      );
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
  use leptos_actix::redirect;
  redirect(&format!("/r/s/p?term={}", &term));
  Ok(())
}

#[server(InstanceFn, "/serverfn")]
pub async fn instance(instance: String) -> Result<(), ServerFnError> {
  let (get_instance_cookie, set_instance_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
    "instance",
    UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax),
  );
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
    UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax),
  );
  set_locale_cookie.set(Some(lang.to_lowercase()));
  Ok(())
}

#[server(ChangeThemeFn, "/serverfn")]
pub async fn change_theme(theme: String) -> Result<(), ServerFnError> {
  let (_, set_theme_cookie) =
    use_cookie_with_options::<String, FromToStringCodec>("theme", UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax));
  set_theme_cookie.set(Some(theme));
  Ok(())
}

#[component]
pub fn ResponsiveTopNav(
  ssr_site: Resource<(Option<String>, Option<String>), Result<GetSiteResponse, LemmyAppError>>,
  #[prop(optional)] default_sort: MaybeProp<SortType>,
  #[prop(optional)] post_view: MaybeSignal<Option<GetPostResponse>>,
  // #[prop(optional)] post_name: MaybeSignal<String>,
  // #[prop(optional)] user_name: MaybeSignal<String>,
  // #[prop(optional)] community_name: MaybeSignal<String>,
) -> impl IntoView {
  let i18n = use_i18n();

  let (_, set_theme_cookie) =
    use_cookie_with_options::<String, FromToStringCodec>("theme", UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax));

  let online = expect_context::<RwSignal<OnlineSetter>>();
  let error = expect_context::<RwSignal<Vec<Option<(LemmyAppError, Option<RwSignal<bool>>)>>>>();
  // let csr_resources = expect_context::<RwSignal<BTreeMap<(usize, ResourceStatus), (Option<PaginationCursor>, Option<GetPostsResponse>)>>>();
  // let csr_next_page_cursor = expect_context::<RwSignal<(usize, Option<PaginationCursor>)>>();
  let response_cache = expect_context::<RwSignal<BTreeMap<(usize, GetPosts), (i64, LemmyAppResult<GetPostsResponse>)>>>();
  // let response_load = expect_context::<RwSignal<ResponseLoad>>();

  // let ssr_error = RwSignal::new::<Option<(LemmyAppError, Option<RwSignal<bool>>)>>(None);

  // if let Some(Err(e)) = site_signal.get() {
  //   ssr_error.set(Some((e, None)));
  // }
  //

  let scroll_element = expect_context::<RwSignal<Option<NodeRef<Div>>>>();

  let query = use_query_map();

  let ssr_query_error = move || {
    serde_json::from_str::<LemmyAppError>(&query.get().get("error").cloned().unwrap_or("".into()))
      .ok()
      .map(|e| (e, None::<Option<RwSignal<bool>>>))
  };
  let ssr_list = move || serde_json::from_str::<ListingType>(&query.get().get("list").cloned().unwrap_or("".into())).unwrap_or(ListingType::All);
  let ssr_sort = move || {
    serde_json::from_str::<SortType>(&query.get().get("sort").cloned().unwrap_or("".into())).unwrap_or(default_sort.get().unwrap_or(SortType::Active))
  };

  // #[cfg(not(feature = "ssr"))]
  // let (get_scroll_cookie, set_scroll_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
  //   "scroll",
  //   UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax),
  // );

  let on_sort_click = move |s: SortType| {
    move |_e: MouseEvent| {
      // csr_next_page_cursor.set((0, None));
      let r = serde_json::to_string::<SortType>(&s);
      let mut query_params = query.get();
      match r {
        Ok(o) => {
          query_params.insert("sort".into(), o);
        }
        Err(e) => {
          error.update(|es| es.push(Some((e.into(), None))));
        }
      }
      if default_sort.get().unwrap_or(SortType::Active) == s {
        query_params.remove("sort".into());
      }
      query_params.remove("page".into());
      let navigate = leptos_router::use_navigate();
      if let Ok(Some(s)) = window().local_storage() {
        let _ = s.set_item(&format!("{}{}", use_location().pathname.get(), query_params.to_query_string()), "0");
      }
      // if let Some(on_scroll_element) = scroll_element.get() {
      //   if let Some(se) = on_scroll_element.get() {
      //     se.set_scroll_left(0i32);
      //   }
      // }
      //

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
      let navigate = leptos_router::use_navigate();
      if l == ListingType::All {
        query_params.remove("list".into());
      } else {
        query_params.insert("list".into(), serde_json::to_string(&l).ok().unwrap());
      }
      if let Ok(Some(s)) = window().local_storage() {
        let _ = s.set_item(&format!("{}{}", use_location().pathname.get(), query_params.to_query_string()), "0");
      }
      // if let Some(on_scroll_element) = scroll_element.get() {
      //   if let Some(se) = on_scroll_element.get() {
      //     se.set_scroll_left(0i32);
      //   }
      // }
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

  // let ssr_error = move || query.with(|params| params.get("error").cloned());

  // if let Some(e) = ssr_error() {
  //   if !e.is_empty() {
  //     let r = serde_json::from_str::<LemmyAppError>(&e[..]);

  //     match r {
  //       Ok(e) => {
  //         error.set(Some((e, None)));
  //       }
  //       Err(_) => {
  //         logging::error!("error decoding error - log and ignore in UI?");
  //       }
  //     }
  //   }
  // }

  let notifications_refresh = expect_context::<RwSignal<NotificationsRefresh>>();
  // let uri = expect_context::<RwSignal<UriSetter>>();

  let logout_action = create_server_action::<LogoutFn>();

  let refresh = RwSignal::new(true);
  let search_show = RwSignal::new(false);
  let still_pressed = create_rw_signal(false);

  // let unread_visibility: RwSignal<Option<Signal<VisibilityState>>> = RwSignal::new(None);
  // let unread_effect: RwSignal<Option<Effect<()>>> = RwSignal::new(None);
  // let unread_interval: RwSignal<Option<IntervalHandle>> = RwSignal::new(None);

  #[cfg(not(feature = "ssr"))]
  let visibility = expect_context::<Signal<VisibilityState>>();

  #[cfg(not(feature = "ssr"))]
  let _visibility_effect = Effect::new(move |_| match visibility.get() {
    VisibilityState::Visible => {
      refresh.update(|b| *b = !*b);
    }
    VisibilityState::Hidden => {}
    _ => {}
  });

  // #[cfg(not(feature = "ssr"))]
  // let _unread_interval_handle = set_interval_with_handle(
  //   move || match visibility.get() {
  //     VisibilityState::Visible => {
  //       refresh.update(|b| *b = !*b);
  //     }
  //     VisibilityState::Hidden => {}
  //     _ => {}
  //   },
  //   std::time::Duration::from_millis(60000),
  // )
  // .ok();

  let on_logout_submit = move |ev: SubmitEvent| {
    ev.prevent_default();

    create_local_resource(
      move || (),
      move |()| async move {
        let result = LemmyClient.logout().await;
        match result {
          Ok(_o) => {
            let (_, set_auth_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
              "jwt",
              UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax),
            );
            set_auth_cookie.set(None);
            let (get_auth_cookie, set_auth_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
              "jwt",
              UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax),
            );
            set_theme_cookie.set(None);
          }
          Err(e) => {
            logging::warn!("logout error {:#?}", e);
            error.update(|es| es.push(Some((e, None))));
          }
        }
      },
    );
  };

  let search_action = create_server_action::<SearchFn>();
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
    use_navigate()(&format!("/r/s/p?term={}", search_term.get()), NavigateOptions::default());
  };

  let (get_instance_cookie, set_instance_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
    "instance",
    UseCookieOptions::default().max_age(604800000).path("/").same_site(SameSite::Lax),
  );
  let instance_term = RwSignal::new(get_instance_cookie.get());
  let instance_action = create_server_action::<InstanceFn>();
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
      ));
    });
    let mut query_params = query.get();
    query_params.remove("page".into());
    let navigate = leptos_router::use_navigate();
    // if let Ok(Some(s)) = window().local_storage() {
    //   let _ = s.set_item(&format!("{}{}", use_location().pathname.get(), query_params.to_query_string()), "0");
    // }
    navigate(
      &format!("{}{}", use_location().pathname.get(), query_params.to_query_string()),
      Default::default(),
    );
  };

  let logged_in = Signal::derive(move || {
    if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site.get() {
      Some(true)
    } else {
      Some(false)
    }
  });

  // let unread_resource = Resource::new(
  //   move || (refresh.get(), logged_in.get(), notifications_refresh.get()),
  //   move |(_refresh, logged_in, _notifications_refresh)| async move {
  //     if online.get().0 {
  //       let result = if logged_in == Some(true) {
  //         LemmyClient.unread_count().await
  //       } else {
  //         Ok(GetUnreadCountResponse {
  //           replies: 0,
  //           mentions: 0,
  //           private_messages: 0,
  //         })
  //       };
  //       match result {
  //         Ok(o) => Ok(o),
  //         Err(e) => {
  //           error.update(|es| es.push(Some((e.clone(), None))));
  //           Err(e)
  //         }
  //       }
  //     } else {
  //       Ok(GetUnreadCountResponse {
  //         replies: 0,
  //         mentions: 0,
  //         private_messages: 0,
  //       })
  //     }
  //   },
  // );

  let online = expect_context::<RwSignal<OnlineSetter>>();
  let theme_action = create_server_action::<ChangeThemeFn>();

  let on_theme_submit = move |theme_name: &'static str| {
    move |ev: SubmitEvent| {
      ev.prevent_default();
      set_theme_cookie.set(Some(theme_name.to_string()));
    }
  };

  let lang_action = create_server_action::<ChangeLangFn>();

  let on_lang_submit = move |lang: Locale| {
    move |ev: SubmitEvent| {
      ev.prevent_default();
      i18n.set_locale(lang);
    }
  };

  let on_navigate_login = move |ev: SubmitEvent| {
    ev.prevent_default();
    // let l = use_location();
    // uri.set(UriSetter(format!("{}{}", l.pathname.get(), l.query.get().to_query_string())));
    use_navigate()("/login", NavigateOptions::default());
  };

  let instance_timer_handle = create_rw_signal(None);
  // let pressed = create_rw_signal(false);

  let on_pointer_down = {
    let timer_id = instance_timer_handle.clone();
    // let pressed = pressed.clone();
    // let on_long_press = std::rc::Rc::new(on_long_press);

    move |_| {
      // pressed.set(true);
      // let on_long_press = on_long_press.clone();
      let timer = set_timeout_with_handle(
        move || {
          // vote_show.set(!vote_show.get());
          // still_down.set(true);
          // if pressed.get() {
          still_pressed.set(true);
          // log!("long press");
          // log!("{}", still_pressed.get());
          // }
        },
        std::time::Duration::from_millis(500),
      )
      .ok();
      // let timer = gloo_timers::callback::Timeout::new(duration, move || {
      //   if pressed.get() {
      //     on_long_press();
      //   }
      // });
      timer_id.set(timer);
    }
  };

  let on_pointer_up = {
    let timer_id = instance_timer_handle.clone();
    // let pressed = pressed.clone();
    move |_| {
      // log!("{}", still_pressed.get());
      // pressed.set(false);
      // still_pressed.set(false);
      if let Some(timer) = timer_id.get() {
        timer.clear();
      }
      timer_id.set(None);
    }
  };

  view! {
    <nav class="flex flex-row py-0 navbar">
      <div class={move || { (if search_show.get() { "hidden" } else { "flex" }).to_string() }}>
        // class="flex-grow flex"
        <ActionForm
          class={move || { if still_pressed.get() { "" } else { "hidden" } }}
          action={instance_action}
          on:submit={on_instance_submit}
        >
          <input
            // placeholder=move || display_title.get()
            // title={move || display_title.get()}
            class="w-40 input text-xl pl-6"
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
              href="/r"
              class={move || { if still_pressed.get() { "hidden" } else { "select-none text-xl whitespace-nowrap py-1/2" } }}
              // class="text-xl whitespace-nowrap py-1/2"
              on:pointerdown=on_pointer_down
              on:pointerup=on_pointer_up
              on:pointerleave=on_pointer_up
              on:click={move |e: MouseEvent| {
                // log!("{}", still_pressed.get());
                if still_pressed.get() {
                  // pressed.set(false);
                  still_pressed.set(false);
                  e.prevent_default();
                } else {
                  if let Some(on_scroll_element) = scroll_element.get() {
                    if let Some(se) = on_scroll_element.get() {
                      se.set_scroll_left(0i32);
                    }
                  }
                  response_cache.update(move |rc| {
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
                        ),
                      );
                  });
                }
              }}
            >
              {move || {
                if let Some(Ok(GetSiteResponse { site_view: SiteView { site: Site { icon: Some(i), .. }, .. }, .. })) = ssr_site.get() {
                  view! { <img class="h-8 sm:hidden" src={i.inner().to_string()} /> }
                } else {
                  view! { <img class="h-8" src="/favicon.png" /> }
                }
              }}
              <span class="hidden sm:flex">
                {move || { if let Some(Ok(m)) = ssr_site.get() { m.site_view.site.name } else { "A.O.S".to_string() } }}
              </span>
            </A>
          </li>
          // <li>
          // // <div class="hidden mr-3 sm:inline-block join">
          // //   <button class="btn join-item btn-active">"Posts"</button>
          // //   <button class="btn join-item btn-disabled">"Comments"</button>
          // // </div>
          // <div class="hidden sm:block join">
          // <A
          // href={move || {
          // let mut query_params = query.get();
          // query_params.insert("list".into(), serde_json::to_string(&ListingType::Subscribed).ok().unwrap());
          // query_params.remove("page".into());
          // // query_params.remove("prev".into());
          // format!("{}{}", use_location().pathname.get(), query_params.to_query_string())
          // }}
          // on:click={ move |e: MouseEvent| {
          // #[cfg(not(feature = "ssr"))]
          // set_scroll_cookie.set(Some("0".into()));
          // csr_next_page_cursor.set((0, None));

          // // response_cache.set(BTreeMap::new());
          // }}
          // class={move || {
          // format!(
          // "btn join-item{}{}",
          // if ListingType::Subscribed == ssr_list() { " btn-active" } else { "" },
          // if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site.get() { "" } else { " btn-disabled" },
          // )
          // }}
          // >
          // "Subscribed"
          // </A>
          // <A
          // href={move || {
          // let mut query_params = query.get();
          // query_params.insert("list".into(), serde_json::to_string(&ListingType::Local).ok().unwrap());
          // query_params.remove("page".into());
          // // query_params.remove("prev".into());
          // format!("{}{}", use_location().pathname.get(), query_params.to_query_string())
          // }}
          // on:click={ move |e: MouseEvent| {
          // #[cfg(not(feature = "ssr"))]
          // set_scroll_cookie.set(Some("0".into()));
          // csr_next_page_cursor.set((0, None));

          // // response_cache.set(BTreeMap::new());
          // }}
          // class={move || format!("btn join-item{}", if ListingType::Local == ssr_list() { " btn-active" } else { "" })}
          // >
          // "Local"
          // </A>
          // <A
          // href={move || {
          // let mut query_params = query.get();
          // query_params.remove("list".into());
          // query_params.remove("page".into());
          // // query_params.remove("prev".into());
          // format!("{}{}", use_location().pathname.get(), query_params.to_query_string())
          // }}
          // on:click={ move |e: MouseEvent| {
          // #[cfg(not(feature = "ssr"))]
          // set_scroll_cookie.set(Some("0".into()));
          // csr_next_page_cursor.set((0, None));

          // // response_cache.set(BTreeMap::new());
          // }}
          // class={move || format!("btn join-item{}", if ListingType::All == ssr_list() { " btn-active" } else { "" })}
          // >
          // "All"
          // </A>
          // </div>
          // </li>
          <li class="hidden sm:flex z-[1]">
            // <div class="dropdown">
            // <label tabindex="0" class="btn">
            // "List"
            // </label>
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
                      if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site.get() { "" } else { " btn-disabled" },
                    )
                  }}
                  on:click={on_csr_filter_click(ListingType::Subscribed)}
                >
                  <span>"Subscribed"</span>
                </li>
              </ul>
            // </div>
            </details>
          </li>
          <li class="hidden sm:flex z-[1]">

            <details>
              <summary>
                <Icon icon={Sort} />
              </summary>

              // <div class="dropdown">
              // <label tabindex="0" class="btn">
              // "Sort"
              // </label>
              <ul tabindex="0" class="shadow menu dropdown-content z-[1] bg-base-100 rounded-box">
                <li
                  class={move || { (if SortType::Active == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                  on:click={on_sort_click(SortType::Active)}
                >
                  <span>{t!(i18n, active)}</span>
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
                // <li class={move || { (if SortType::Hot == ssr_sort() { "btn-active" } else { "" }).to_string() }} on:click={on_sort_click(SortType::Hot)}>
                // <span>{t!(i18n, hot)}</span>
                // </li>
                <li
                  class={move || { (if SortType::Scaled == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                  on:click={on_sort_click(SortType::Scaled)}
                >
                  <span>{"Scaled"}</span>
                </li>
              // <li class={move || { (if SortType::New == ssr_sort() { "btn-active" } else { "" }).to_string() }} on:click={on_sort_click(SortType::New)}>
              // <span>{t!(i18n, new)}</span>
              // </li>
              </ul>
            // </div>
            </details>
          </li>
          <li class="flex sm:hidden">
            <details>
              <summary>
                // {move || {
                // if let Some(Ok(GetSiteResponse { my_user: Some(m), .. })) = ssr_site.get() {
                // m.local_user_view.person.display_name.unwrap_or(m.local_user_view.person.name)
                // } else {
                // String::default()
                // }
                // }}
                <Icon icon={Filter} />
              </summary>
              <ul class="z-[1]">
                <li class="flex z-[1]">
                  // <div class="dropdown">
                  // <label tabindex="0" class="btn">
                  // "List"
                  // </label>
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
                            if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site.get() { "" } else { " btn-disabled" },
                          )
                        }}
                        on:click={on_csr_filter_click(ListingType::Subscribed)}
                      >
                        <span>"Subscribed"</span>
                      </li>
                    </ul>
                  // </div>
                  </details>
                </li>
                <li class="flex z-[1]">

                  <details>
                    <summary>
                      <Icon icon={Sort} />
                    </summary>

                    // <div class="dropdown">
                    // <label tabindex="0" class="btn">
                    // "Sort"
                    // </label>
                    <ul tabindex="0" class="shadow menu dropdown-content z-[1] bg-base-100 rounded-box">
                      <li
                        class={move || { (if SortType::Active == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                        on:click={on_sort_click(SortType::Active)}
                      >
                        <span>{t!(i18n, active)}</span>
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
                      // <li class={move || { (if SortType::Hot == ssr_sort() { "btn-active" } else { "" }).to_string() }} on:click={on_sort_click(SortType::Hot)}>
                      // <span>{t!(i18n, hot)}</span>
                      // </li>
                      <li
                        class={move || { (if SortType::Scaled == ssr_sort() { "btn-active" } else { "" }).to_string() }}
                        on:click={on_sort_click(SortType::Scaled)}
                      >
                        <span>{"Scaled"}</span>
                      </li>
                    // <li class={move || { (if SortType::New == ssr_sort() { "btn-active" } else { "" }).to_string() }} on:click={on_sort_click(SortType::New)}>
                    // <span>{t!(i18n, new)}</span>
                    // </li>
                    </ul>
                  // </div>
                  </details>
                </li>
              </ul>
            </details>
          </li>

        </ul>

      </div>
      <div class="flex flex-grow">
        // <ActionForm
        // // class={move || { (if search_show.get() { "form-control hidden sm:flex flex-grow" } else { "form-control hidden sm:flex flex-grow" }).to_string() }}
        // class="form-control hidden sm:flex flex-grow"
        // action={search_action} on:submit={on_search_submit}>
        // <input
        // // placeholder=move || display_title.get()
        // title=move || display_title.get()
        // class="input w-full"
        // type="text" name="term"
        // prop:value={move || display_title.get()}
        // on:input={move |ev| {
        // search_term.set(event_target_value(&ev));
        // }}
        // />
        // </ActionForm>
        <ActionForm
          class={move || { (if search_show.get() { "form-control flex flex-grow" } else { "form-control hidden sm:flex flex-grow" }).to_string() }}
          // class="form-control hidden sm:flex flex-grow"
          action={search_action}
          on:submit={on_search_submit}
        >
          <input
            // placeholder=move || display_title.get()
            title={move || display_title.get()}
            class="w-full input"
            type="text"
            name="term"
            prop:value={move || display_title.get()}
            on:input={move |ev| {
              search_term.set(event_target_value(&ev));
            }}
          />
        </ActionForm>
      // <div class="flex flex-grow">
      // </div>

      // <form class="form-control flex-grow" action="/responsive/s/p" method="GET">
      // <input name="term" type="text"
      // // on:keypress={|e: KeyboardEvent| {
      // //   if e.key.eq("\n") {

      // //   } else {

      // //   }
      // //   log!("{:#?}", e);
      // // }}
      // placeholder=move || display_title.get() //=move || if let Some(pv) = post_view.get() { format!("{} by {} in {}", pv.post_view.post.name, pv.post_view.creator.name, pv.community_view.community.name) } else { "".to_string() }
      // title=move || display_title.get() //move || if let Some(pv) = post_view.get() { format!("{} by {} in {}", pv.post_view.post.name, pv.post_view.creator.name, pv.community_view.community.name) } else { "".to_string() }
      // class="input w-full" />
      // // <button class="py-2 px-4" type="submit">
      // //   <Icon icon={SignIn} />
      // //   // {t!(i18n, login)}
      // // </button>
      // </form>
      </div>
      // <div class="navbar-center">
      // //        <A href={move || format!("/responsive/p/{}", post_view.get().post.id)} class=" hover:text-accent">
      // <span class="block text-lg break-words" inner_html={post_name.get()} />
      // // </A>
      // <span class="block mb-1">
      // // <span>{abbr_duration}</span>
      // " ago by "
      // <a
      // // href={move || format!("{}", post_view.get().creator.actor_id)}
      // target="_blank"
      // class="inline text-sm break-words hover:text-secondary"
      // >
      // <span inner_html={user_name.get()} />
      // </a>
      // " in "
      // <a
      // class="inline text-sm break-words hover:text-secondary"
      // // href={if post_view.get().community.local {
      // //   format!("/responsive/c/{}", post_view.get().community.name)
      // // } else {
      // //   format!("/responsive/c/{}@{}", post_view.get().community.name, post_view.get().community.actor_id.inner().host().unwrap().to_string())
      // // }}
      // // on:click={ move |e: MouseEvent| {
      // //   csr_resources.set(BTreeMap::new());
      // // }}
      // >
      // <span inner_html={community_name.get()} />
      // </a>
      // </span>
      // </div>
      <div class="flex-none">
        <button
          class="py-2 px-4"
          // class={move || { (if search_show.get() { "hidden py-2 px-4" } else { "block py-2 px-4" }).to_string() }}
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
        // class="flex-none">
        <ul class="flex-nowrap items-center menu menu-horizontal">
          // <li class="flex">
          // </li>
          <li class="hidden sm:flex">
            <details>
              <summary>
                <Icon icon={Translate} />
              </summary>
              <ul class="z-[1] [inset-inline-end:0]">
                <li>
                  <ActionForm class="p-0" action={lang_action} on:submit={on_lang_submit(Locale::fr)}>
                    <input type="hidden" name="lang" value="FR" />
                    <button class="py-2 px-4" type="submit">
                      "FR"
                    </button>
                  </ActionForm>
                </li>
                <li>
                  <ActionForm class="p-0" action={lang_action} on:submit={on_lang_submit(Locale::en)}>
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
                  <ActionForm class="p-0" action={theme_action} on:submit={on_theme_submit("dark")}>
                    <input type="hidden" name="theme" value="dark" />
                    <button class="py-2 px-4" type="submit">
                      "Dark"
                    </button>
                  </ActionForm>
                </li>
                <li>
                  <ActionForm class="p-0" action={theme_action} on:submit={on_theme_submit("light")}>
                    <input type="hidden" name="theme" value="light" />
                    <button class="py-2 px-4" type="submit">
                      "Light"
                    </button>
                  </ActionForm>
                </li>
                <li>
                  <ActionForm class="p-0" action={theme_action} on:submit={on_theme_submit("retro")}>
                    <input type="hidden" name="theme" value="retro" />
                    <button class="py-2 px-4" type="submit">
                      "Retro"
                    </button>
                  </ActionForm>
                </li>
              </ul>
            </details>
          </li>
          // <Transition fallback={|| {}}>
          // {move || {
          // unread_resource
          // .get()
          // .map(|u| {
          // let unread = if let Ok(c) = u.clone() { format!(", {} unread", c.replies + c.mentions + c.private_messages) } else { "".into() };
          // view! {
          // <li title={move || {
          // format!(
          // "{}{}{}",
          // if error.get().len() > 0 { format!("{} errors, ", error.get().len()) } else { "".into() },
          // if online.get().0 { "app online" } else { "app offline" },
          // unread,
          // )
          // }}>
          // <li>
          // <A href="/notifications">
          // <span class="flex flex-row items-center">
          // {move || {
          // let v = error.get();
          // (v.len() > 0)
          // .then(move || {
          // let l = v.len();
          // view! { <div class="badge badge-error badge-xs">{l}</div> }
          // })
          // }}
          // <span>
          // {move || { (!online.get().0).then(move || view! { <div class="absolute top-0 badge badge-warning badge-xs" /> }) }}
          // <Icon icon={Notifications} />
          // </span>
          // // {if let Ok(c) = u {
          // //   (c.replies + c.mentions + c.private_messages > 0)
          // //     .then(move || view! { <div class="badge badge-primary badge-xs">{c.replies + c.mentions + c.private_messages}</div> })
          // // } else {
          // //   None
          // // }}
          // </span>
          // </A>
          // </li>
          // }
          // })
          // }}
          // </Transition>
          <Show
            when={move || { if let Some(Ok(GetSiteResponse { my_user: Some(_), .. })) = ssr_site.get() { true } else { false } }}
            fallback={move || {
              view! {
                // let l = use_location();
                <li>
                  // <ActionForm action="/login" on:submit=|_| {}>
                  // <input type="hidden" name="uri" value=move || format!("{}{}", l.pathname.get(), l.query.get().to_query_string())/>
                  // <button type="submit">"lowgin"</button>
                  // </ActionForm>
                  // <Form action="/login" method="POST" on:submit=|_| {}>
                  // <input type="hidden" name="theme" value="retro"/>
                  // <button type="submit">"LOGIN"</button>
                  // </Form>
                  <form class="p-0" action="/login" method="POST" on:submit={on_navigate_login}>
                    <button class="py-2 px-4" type="submit">
                      <Icon icon={SignIn} />
                    // {t!(i18n, login)}
                    </button>
                  </form>
                // <A href="/login">{t!(i18n, login)}</A>
                </li>
              }
            }}
          >
            <li>
              <details>
                <summary>
                  // {move || {
                  // if let Some(Ok(GetSiteResponse { my_user: Some(m), .. })) = ssr_site.get() {
                  // m.local_user_view.person.display_name.unwrap_or(m.local_user_view.person.name)
                  // } else {
                  // String::default()
                  // }
                  // }}
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
                          <ActionForm class="p-0" action={theme_action} on:submit={on_theme_submit("dark")}>
                            <input type="hidden" name="theme" value="dark" />
                            <button class="py-2 px-4" type="submit">
                              "Dark"
                            </button>
                          </ActionForm>
                        </li>
                        <li>
                          <ActionForm class="p-0" action={theme_action} on:submit={on_theme_submit("light")}>
                            <input type="hidden" name="theme" value="light" />
                            <button class="py-2 px-4" type="submit">
                              "Light"
                            </button>
                          </ActionForm>
                        </li>
                        <li>
                          <ActionForm class="p-0" action={theme_action} on:submit={on_theme_submit("retro")}>
                            <input type="hidden" name="theme" value="retro" />
                            <button class="py-2 px-4" type="submit">
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
                          if let Some(Ok(GetSiteResponse { my_user: Some(m), .. })) = ssr_site.get() {
                            let _ = window().location().set_href(&format!("//lemmy.world/u/{}", m.local_user_view.person.name));
                          }
                        }
                      }}
                      href={move || {
                        format!(
                          "/u/{}",
                          if let Some(Ok(GetSiteResponse { my_user: Some(m), .. })) = ssr_site.get() {
                            m.local_user_view.person.name
                          } else {
                            String::default()
                          },
                        )
                      }}
                    >
                      {t!(i18n, profile)}
                    </A>
                  </li>
                  <li>
                    <A class="pointer-events-none text-base-content/50" href="/settings">
                      {t!(i18n, settings)}
                    </A>
                  </li>
                  <div class="my-0 divider" />
                  <li>
                    <ActionForm action={logout_action} on:submit={on_logout_submit}>
                      <button type="submit">{t!(i18n, logout)}</button>
                    </ActionForm>
                  </li>
                </ul>
              </details>
            </li>
          </Show>
        </ul>
      </div>
    </nav>
    // <Show
    // when=move || error.get().is_some()
    // fallback=move || {
    // view! { <div class="hidden"></div> }
    // }
    // >

    // {move || {
    // site_signal.get()
    // .map(|res| {

    // if let Err(err) = res {
    // view! {
    // <div class="container mx-auto alert alert-error mb-8">
    // <span>"S" {message_from_error(&err)} " - " {err.content}</span>
    // <div>
    // <A href=use_location().pathname.get() class="btn btn-sm"> "Retry" </A>
    // </div>
    // </div>
    // }
    // } else {
    // view! {
    // <div class="hidden" />
    // }

    // }
    // })
    // }}

    {move || {
      ssr_query_error()
        .map(|err| {
          let mut query_params = query.get();
          query_params.remove("error".into());
          view! {
            <div class="container mx-auto mb-8 alert alert-error">
              <span>{message_from_error(&err.0)} " - " {err.0.content}</span>
              <div>
                <A class="btn btn-sm" href={format!("./?{}", &query_params.to_query_string())}>
                  "Clear"
                </A>
              </div>
            </div>
          }
        })
    }}
  }
}

#[component]
pub fn BottomNav(ssr_site: Resource<(Option<String>, Option<String>), Result<GetSiteResponse, LemmyAppError>>) -> impl IntoView {
  let i18n = use_i18n();
  const FE_VERSION: &str = env!("CARGO_PKG_VERSION");
  const GIT_HASH: std::option::Option<&'static str> = option_env!("GIT_HASH");

  let version = move || {
    if let Some(Ok(m)) = ssr_site.get() {
      m.version
    } else {
      "A.O.S".to_string()
    }
  };

  view! {
    <nav class="container hidden mx-auto lg:flex navbar">
      <div class="w-auto navbar-start" />
      <div class="w-auto navbar-end grow">
        <ul class="flex-nowrap items-center menu menu-horizontal">
          <li>
            <a href="//github.com/jim-taylor-business/aos/releases" class="text-md">
              "FE: "
              {FE_VERSION}
              "."
              {GIT_HASH}
            </a>
          </li>
          <li>
            <a href="//github.com/LemmyNet/lemmy/releases" class="text-md">
              "BE: "
              {move || version()}
            </a>
          </li>
          <li>
            <A href="/modlog" class="pointer-events-none text-md text-base-content/50">
              {t!(i18n, modlog)}
            </A>
          </li>
          <li>
            <A href="/instances" class="pointer-events-none text-md text-base-content/50">
              {t!(i18n, instances)}
            </A>
          </li>
          <li>
            <a href="//join-lemmy.org/docs/en/index.html" class="text-md">
              {t!(i18n, docs)}
            </a>
          </li>
          <li>
            <a href="//github.com/LemmyNet" class="text-md">
              {t!(i18n, code)}
            </a>
          </li>
          <li>
            <a href="//join-lemmy.org" class="text-md">
              "join-lemmy.org"
            </a>
          </li>
        </ul>
      </div>
    </nav>
  }
}
