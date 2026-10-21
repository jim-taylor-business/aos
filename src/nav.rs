use crate::{
  BanType, KindType, OnlineSetter, PassedUrl, ReadAuthCookie, ReadInstanceCookie, WriteAuthCookie, WriteInstanceCookie, WriteThemeCookie,
  client::*,
  db::csr_indexed_db::*,
  errors::{LemmyAppError, LemmyAppErrorType, LemmyAppResult},
  icon::{IconType::*, *},
};
use lemmy_api_common::{
  lemmy_db_schema::{ListingType, SortType},
  lemmy_db_views::structs::PaginationCursor,
  post::{GetPostResponse, GetPosts, GetPostsResponse},
  site::GetSiteResponse,
};
use leptos::{html::Div, logging::log, prelude::*, server::codee::string::FromToStringCodec, task::spawn_local_scoped, *};
use leptos_router::{components::*, hooks::*, *};
use leptos_use::{SameSite, UseCookieOptions, use_cookie_with_options};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use web_sys::{KeyboardEvent, MouseEvent, SubmitEvent, VisibilityState};

#[server]
pub async fn logout_fn() -> Result<(), ServerFnError> {
  let (get_auth_cookie, set_auth_cookie) =
    use_cookie_with_options::<String, FromToStringCodec>("jwt", UseCookieOptions::default().max_age(691200000).path("/").same_site(SameSite::Lax));
  provide_context(ReadAuthCookie(get_auth_cookie));
  provide_context(WriteAuthCookie(set_auth_cookie));
  let (get_instance_cookie, set_instance_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
    "instance",
    UseCookieOptions::default().max_age(691200000).path("/").same_site(SameSite::Lax),
  );
  provide_context(ReadInstanceCookie(get_instance_cookie));
  provide_context(WriteInstanceCookie(set_instance_cookie));
  use leptos_axum::redirect;
  let result = LemmyClient.logout().await;
  match result {
    Ok(_o) => {
      let response = expect_context::<leptos_axum::ResponseOptions>();
      if let Ok(header_value) = format!("jwt=; SameSite=Lax; Path=/; Max-Age=691200").parse::<http::HeaderValue>() {
        response.insert_header(axum::http::header::SET_COOKIE, header_value);
      }
      Ok(())
    }
    Err(e) => {
      redirect(&format!("/login?error={}", serde_json::to_string(&e)?)[..]);
      Ok(())
    }
  }
}

#[server]
pub async fn search_action(term: String) -> Result<(), ServerFnError> {
  use leptos_axum::redirect;
  redirect(&format!("/s?term={}", &term));
  Ok(())
}

#[server]
pub async fn change_instance(instance: String) -> Result<(), ServerFnError> {
  let response = use_context::<leptos_axum::ResponseOptions>().ok_or(LemmyAppErrorType::InternalServerError)?;
  response.insert_header(
    axum::http::header::SET_COOKIE,
    http::HeaderValue::from_str(
      &cookie::Cookie::build(cookie::Cookie::new("instance", instance))
        .path("/")
        .same_site(cookie::SameSite::Lax)
        .max_age(cookie::time::SignedDuration::seconds(691200))
        .to_string(),
    )?,
  );

  Ok(())
}

#[server]
pub async fn change_lang_fn(lang: String) -> Result<(), ServerFnError> {
  let (_, set_locale_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
    "i18n_pref_locale",
    UseCookieOptions::default().max_age(691200000).path("/").same_site(SameSite::Lax),
  );
  set_locale_cookie.set(Some(lang.to_lowercase()));
  Ok(())
}

#[server]
pub async fn change_theme(theme: String) -> Result<(), ServerFnError> {
  let response = use_context::<leptos_axum::ResponseOptions>().ok_or(LemmyAppErrorType::InternalServerError)?;
  response.insert_header(
    axum::http::header::SET_COOKIE,
    http::HeaderValue::from_str(
      &cookie::Cookie::build(cookie::Cookie::new("theme", theme))
        .path("/")
        .same_site(cookie::SameSite::Lax)
        .max_age(cookie::time::SignedDuration::seconds(691200))
        .to_string(),
    )?,
  );

  Ok(())
}

#[component]
pub fn TopNav(
  scroll_element: Signal<Option<NodeRef<Div>>>,
  #[prop(optional)] next_page_cursor: RwSignal<(usize, Option<PaginationCursor>)>,
  #[prop(optional)] default_sort: Signal<Option<SortType>>,
  #[prop(optional)] post_view: RwSignal<Option<GetPostResponse>>,
  #[prop(optional, into)] lost_path: Option<RwSignal<String>>,
) -> impl IntoView {
  // let i18n = use_i18n();
  let ssr_site = expect_context::<Resource<Result<GetSiteResponse, LemmyAppError>>>();

  let ReadAuthCookie(get_auth_cookie) = expect_context::<ReadAuthCookie>();
  let WriteThemeCookie(set_theme_cookie) = expect_context::<WriteThemeCookie>();
  let _online = expect_context::<RwSignal<OnlineSetter>>();
  let response_cache = expect_context::<RwSignal<BTreeMap<(usize, GetPosts, Option<String>, bool), (i64, LemmyAppResult<GetPostsResponse>)>>>();

  let query = use_query_map();
  let _ssr_query_error =
    move || serde_json::from_str::<LemmyAppError>(&query.get().get("error").unwrap_or("".into())).ok().map(|e| (e, None::<Option<RwSignal<bool>>>));
  let ssr_list = move || serde_json::from_str::<ListingType>(&query.get().get("list").unwrap_or("".into())).unwrap_or(ListingType::All);
  let ssr_sort =
    move || serde_json::from_str::<SortType>(&query.get().get("sort").unwrap_or("".into())).unwrap_or(default_sort.get().unwrap_or(SortType::Active));
  let ssr_term = move || query.get().get("term").unwrap_or("".into());
  let ssr_kind = move || serde_json::from_str::<KindType>(&query.get().get("kind").unwrap_or("".into())).unwrap_or(KindType::Post);
  let ssr_bans = move || serde_json::from_str::<Vec<BanType>>(&query.get().get("bans").unwrap_or("".into())).unwrap_or(vec![]);

  let lg_filter_menu = NodeRef::<html::Details>::new();
  let lg_sort_menu = NodeRef::<html::Details>::new();
  let lg_language_menu = NodeRef::<html::Details>::new();
  let lg_theme_menu = NodeRef::<html::Details>::new();
  let sm_content_menu = NodeRef::<html::Details>::new();
  let sm_ui_menu = NodeRef::<html::Details>::new();
  let lg_kind_menu = NodeRef::<html::Details>::new();
  let lg_ban_menu = NodeRef::<html::Details>::new();

  let instance_input = NodeRef::<html::Input>::new();
  let search_input = NodeRef::<html::Input>::new();

  #[derive(PartialEq)]
  enum MenuType {
    LgFilter,
    LgSort,
    LgLanguage,
    LgTheme,
    SmContent,
    SmUi,
    LgKind,
    LgBan,
  }

  let reset_menus = move || {
    if let Some(m) = lg_filter_menu.get() {
      let _ = m.remove_attribute("open");
    }
    if let Some(m) = lg_sort_menu.get() {
      let _ = m.remove_attribute("open");
    }
    if let Some(m) = lg_language_menu.get() {
      let _ = m.remove_attribute("open");
    }
    if let Some(m) = lg_theme_menu.get() {
      let _ = m.remove_attribute("open");
    }
    if let Some(m) = sm_content_menu.get() {
      let _ = m.remove_attribute("open");
    }
    if let Some(m) = sm_ui_menu.get() {
      let _ = m.remove_attribute("open");
    }
    if let Some(m) = lg_kind_menu.get() {
      let _ = m.remove_attribute("open");
    }
    if let Some(m) = lg_ban_menu.get() {
      let _ = m.remove_attribute("open");
    }
  };

  let reset_others = move |this: MenuType| {
    if this != MenuType::LgFilter
      && let Some(m) = lg_filter_menu.get()
    {
      let _ = m.remove_attribute("open");
    }
    if this != MenuType::LgSort
      && let Some(m) = lg_sort_menu.get()
    {
      let _ = m.remove_attribute("open");
    }
    if this != MenuType::LgLanguage
      && let Some(m) = lg_language_menu.get()
    {
      let _ = m.remove_attribute("open");
    }
    if this != MenuType::LgTheme
      && let Some(m) = lg_theme_menu.get()
    {
      let _ = m.remove_attribute("open");
    }
    if this != MenuType::SmContent
      && let Some(m) = sm_content_menu.get()
    {
      let _ = m.remove_attribute("open");
    }
    if this != MenuType::SmUi
      && let Some(m) = sm_ui_menu.get()
    {
      let _ = m.remove_attribute("open");
    }
    if this != MenuType::LgKind
      && let Some(m) = lg_kind_menu.get()
    {
      let _ = m.remove_attribute("open");
    }
    if this != MenuType::LgBan
      && let Some(m) = lg_ban_menu.get()
    {
      let _ = m.remove_attribute("open");
    }
  };

  let on_sort_ssr = move |s: SortType| {
    let o = serde_json::to_string::<SortType>(&s).unwrap_or("Active".into());
    let mut query_params = query.get();
    query_params.remove("sort");
    query_params.remove("page");
    if default_sort.get().unwrap_or(SortType::Active) != s {
      query_params.insert("sort", o);
    }
    let params = query_params.clone();
    format!("{}{}", use_location().pathname.get(), query_params.to_query_string())
  };

  let on_sort_click = move |s: SortType| {
    move |e: MouseEvent| {
      e.prevent_default();
      let o = serde_json::to_string::<SortType>(&s).unwrap_or("Active".into());
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
          get_auth_cookie.get(),
          false,
        ));
      });
      let mut query_params = query.get();
      query_params.remove("sort");
      query_params.remove("page");
      if default_sort.get().unwrap_or(SortType::Active) != s {
        query_params.insert("sort", o);
      }
      let params = query_params.clone();
      #[cfg(not(feature = "ssr"))]
      spawn_local_scoped(async move {
        if let Ok(d) = IndexedDb::new().await {
          let _ = d.set(&ScrollPositionKey { path: use_location().pathname.get(), query: params.to_query_string() }, &0i32).await;
        }
        use_navigate()(&format!("{}{}", use_location().pathname.get(), query_params.to_query_string()), Default::default());
      });
      if let Some(on_scroll_element) = scroll_element.get() {
        if let Some(se) = on_scroll_element.get() {
          se.set_scroll_left(0i32);
        }
      }
      reset_menus();
    }
  };

  let on_csr_filter_click = move |l: ListingType| {
    move |e: MouseEvent| {
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
          get_auth_cookie.get(),
          false,
        ));
      });
      let mut query_params = query.get();
      query_params.remove("page");
      query_params.remove("list");
      if l != ListingType::All {
        query_params.insert("list", serde_json::to_string(&l).ok().unwrap_or("All".into()));
      }
      let params = query_params.clone();
      #[cfg(not(feature = "ssr"))]
      spawn_local_scoped(async move {
        if let Ok(d) = IndexedDb::new().await {
          let _ = d.set(&ScrollPositionKey { path: use_location().pathname.get(), query: params.to_query_string() }, &0i32).await;
        }
        use_navigate()(&format!("{}{}", use_location().pathname.get(), query_params.to_query_string()), Default::default());
      });
      if let Some(on_scroll_element) = scroll_element.get() {
        if let Some(se) = on_scroll_element.get() {
          se.set_scroll_left(0i32);
        }
      }
      reset_menus()
    }
  };

  let highlight_csr_filter = move |l: ListingType| {
    if l == ssr_list() { "menu-active" } else { "" }
  };

  let logout_action = ServerAction::<LogoutFn>::new();

  #[derive(PartialEq, Clone)]
  enum InputFocusState {
    Instance,
    Unfocused,
    Search,
  }
  let input_focus = RwSignal::new(InputFocusState::Unfocused);

  #[cfg(not(feature = "ssr"))]
  let visibility = expect_context::<Signal<VisibilityState>>();

  #[cfg(not(feature = "ssr"))]
  let _visibility_effect = Effect::new(move |_| match visibility.get() {
    VisibilityState::Visible => {}
    _ => {}
  });

  let on_logout_submit = move |e: MouseEvent| {
    e.prevent_default();
    spawn_local_scoped(async move {
      let result = LemmyClient.logout().await;
      match result {
        Ok(_o) => {
          let WriteAuthCookie(set_auth_cookie) = expect_context::<WriteAuthCookie>();
          set_auth_cookie.set(None);
          ssr_site.refetch();
        }
        Err(_e) => {}
      }
    });
  };

  let ReadInstanceCookie(get_instance_cookie) = expect_context::<ReadInstanceCookie>();
  let WriteInstanceCookie(set_instance_cookie) = expect_context::<WriteInstanceCookie>();

  let instance_term = RwSignal::new(get_instance_cookie.get());
  let on_instance_submit = move || {
    input_focus.set(InputFocusState::Unfocused);
    if instance_term.get().unwrap_or("".into()).len() > 0 {
      set_instance_cookie.set(instance_term.get());
    } else {
      set_instance_cookie.set(Some("lemmy.world".to_owned()));
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
        get_auth_cookie.get(),
        false,
      ));
    });
    #[cfg(not(feature = "ssr"))]
    spawn_local_scoped(async move {
      if let Ok(d) = IndexedDb::new().await {
        let _ = d.set(&ScrollPositionKey { path: "/".into(), query: "".into() }, &0i32).await;
        use_navigate()("/", Default::default());
      }
    });
    ssr_site.refetch();
  };

  let _online = expect_context::<RwSignal<OnlineSetter>>();
  let change_theme = ServerAction::<ChangeTheme>::new();
  let change_instance = ServerAction::<ChangeInstance>::new();
  let search_action = ServerAction::<SearchAction>::new();

  let on_theme_submit = move |theme_name: &'static str| {
    move |e: MouseEvent| {
      e.prevent_default();
      set_theme_cookie.set(Some(theme_name.to_string()));
      reset_menus();
    }
  };

  // let lang_action = ServerAction::<ChangeLangFn>::new();

  // let _on_lang_submit = move |lang: Locale| {
  //   move |ev: SubmitEvent| {
  //     ev.prevent_default();
  //     // i18n.set_locale(lang);
  //     reset_menus();
  //   }
  // };

  let _on_navigate_login = move |e: MouseEvent| {
    e.prevent_default();
    use_navigate()("/l", NavigateOptions::default());
  };

  let instance_timer_handle = RwSignal::new(None);

  let on_mouse_down = {
    let timer_id = instance_timer_handle.clone();
    move |_| {
      let timer = set_timeout_with_handle(
        move || {
          input_focus.set(InputFocusState::Instance);
          let _ = set_timeout_with_handle(
            move || {
              instance_input.get().map(|i| i.focus());
            },
            std::time::Duration::from_millis(0),
          )
          .ok();
        },
        std::time::Duration::from_millis(500),
      )
      .ok();
      timer_id.set(timer);
    }
  };

  let on_mouse_up = {
    let timer_id = instance_timer_handle.clone();
    move |_| {
      if let Some(timer) = timer_id.get() {
        timer.clear();
      }
      timer_id.set(None);
    }
  };

  let on_touch_start = {
    let timer_id = instance_timer_handle.clone();
    move |_| {
      let timer = set_timeout_with_handle(
        move || {
          input_focus.set(InputFocusState::Instance);
          let _ = set_timeout_with_handle(
            move || {
              instance_input.get().map(|i| i.focus());
            },
            std::time::Duration::from_millis(0),
          )
          .ok();
        },
        std::time::Duration::from_millis(500),
      )
      .ok();
      timer_id.set(timer);
    }
  };

  let on_touch_end = {
    let timer_id = instance_timer_handle.clone();
    move |_| {
      if let Some(timer) = timer_id.get() {
        timer.clear();
      }
      timer_id.set(None);
    }
  };

  let search_term = RwSignal::new("".to_owned());

  let query = use_query_map();
  let on_filter_ssr = move |l: ListingType| {
    let mut query_params = query.get();
    query_params.remove("page");
    query_params.remove("list");
    if l != ListingType::All {
      query_params.insert("list", serde_json::to_string(&l).ok().unwrap_or("All".into()));
    }
    let params = query_params.clone();
    format!("{}{}", use_location().pathname.get(), query_params.to_query_string())
  };

  let on_kind_ssr = move |l: KindType| {
    let mut query_params = query.get();
    query_params.remove("page");
    query_params.remove("kind");
    if l != KindType::Post {
      query_params.insert("kind", serde_json::to_string(&l).ok().unwrap_or("Post".into()));
    }
    let params = query_params.clone();
    format!("/s{}", /*use_location().pathname.get(), */ query_params.to_query_string())
  };

  let highlight_csr_kind = move |l: KindType| {
    if l == ssr_kind() { "menu-active" } else { "" }
  };

  let on_csr_kind_click = move |l: KindType| {
    move |e: MouseEvent| {
      // response_cache.update(move |rc| {
      //   rc.remove(&(
      //     0usize,
      //     GetPosts {
      //       type_: Some(l),
      //       sort: Some(ssr_sort()),
      //       page: None,
      //       limit: Some(50),
      //       community_id: None,
      //       community_name: None,
      //       saved_only: None,
      //       liked_only: None,
      //       disliked_only: None,
      //       show_hidden: Some(true),
      //       show_read: Some(true),
      //       show_nsfw: Some(false),
      //       page_cursor: None,
      //     },
      //     get_auth_cookie.get(),
      //   ));
      // });
      let mut query_params = query.get();
      query_params.remove("page");
      query_params.remove("kind");
      if l != KindType::Post {
        query_params.insert("kind", serde_json::to_string(&l).ok().unwrap_or("Post".into()));
      }
      let params = query_params.clone();
      #[cfg(not(feature = "ssr"))]
      // spawn_local_scoped(async move {
      //   if let Ok(d) = IndexedDb::new().await {
      //     let _ = d.set(&ScrollPositionKey { path: use_location().pathname.get(), query: params.to_query_string() }, &0i32).await;
      //   }
      use_navigate()(&format!("/s{}", /*use_location().pathname.get(), */ query_params.to_query_string()), Default::default());
      // });
      // if let Some(on_scroll_element) = scroll_element.get() {
      //   if let Some(se) = on_scroll_element.get() {
      //     se.set_scroll_left(0i32);
      //   }
      // }
      reset_menus()
    }
  };

  let on_bans_ssr = move |l: BanType| {
    let mut query_params = query.get();
    query_params.remove("page");
    query_params.remove("bans");
    let mut bans = ssr_bans();
    if bans.contains(&l) {
      bans.retain(|b| *b != l)
    } else {
      bans.push(l.clone());
    }
    if bans.len() > 0 {
      query_params.insert("bans", serde_json::to_string(&bans).ok().unwrap_or("".into()));
    }
    let params = query_params.clone();
    format!("{}{}", use_location().pathname.get(), query_params.to_query_string())
  };

  let highlight_csr_bans = move |l: BanType| {
    if ssr_bans().contains(&l) { "menu-active" } else { "" }
  };

  let on_csr_bans_click = move |l: BanType| {
    move |e: MouseEvent| {
      let mut bans = ssr_bans();
      if bans.contains(&l) {
        bans.retain(|b| *b != l)
      } else {
        bans.push(l.clone());
      }
      let nsfw = bans.contains(&BanType::Nsfw);
      let ban = bans.contains(&BanType::Ban);
      response_cache.update(move |rc| {
        rc.remove(&(
          0usize,
          GetPosts {
            type_: Some(ssr_list()),
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
            show_nsfw: Some(nsfw),
            page_cursor: None,
          },
          get_auth_cookie.get(),
          ban,
        ));
      });
      let mut query_params = query.get();
      query_params.remove("page");
      query_params.remove("bans");
      if bans.len() > 0 {
        query_params.insert("bans", serde_json::to_string(&bans).ok().unwrap_or("".into()));
      }
      let params = query_params.clone();
      #[cfg(not(feature = "ssr"))]
      spawn_local_scoped(async move {
        if let Ok(d) = IndexedDb::new().await {
          let _ = d.set(&ScrollPositionKey { path: use_location().pathname.get(), query: params.to_query_string() }, &0i32).await;
        }
        use_navigate()(&format!("{}{}", use_location().pathname.get(), query_params.to_query_string()), Default::default());
      });
      if let Some(on_scroll_element) = scroll_element.get() {
        if let Some(se) = on_scroll_element.get() {
          se.set_scroll_left(0i32);
        }
      }
      reset_menus()
    }
  };

  let display_title = Signal::derive(move || {
    let s = if ssr_term().len() > 0 {
      ssr_term()
    } else if let Some(l) = lost_path {
      l.get()
        .replace(".", " ")
        .replace("/", " ")
        .replace("&", " ")
        .replace("%", " ")
        .replace("?", " ")
        .replace("=", " ")
        .replace("+", " ")
        .replace("-", " ")
        .replace("_", " ")
    } else {
      if let Some(pv) = post_view.get() {
        let community_title = if pv.post_view.community.local {
          format!("{}", pv.post_view.community.name)
        } else {
          format!(
            "{}@{}",
            pv.post_view.community.name,
            if let Some(h) = pv.post_view.community.actor_id.inner().host() { h.to_string() } else { "".to_owned() }
          )
        };
        format!("{} by {} in {}", pv.post_view.post.name, pv.post_view.creator.actor_id.to_string()[8..].to_string(), community_title)
      } else {
        "".to_owned()
      }
    };
    search_term.set(s.clone());
    s
  });

  view! {
    <Transition fallback={|| {}}>
      {move || {
        let site = ssr_site.get();
        let site_details = Memo::new(move |_| if let Some(Ok(s)) = ssr_site.get() { s.site_view.site.name.clone() } else { String::new() });
        let icon_details = Memo::new(move |_| { if let Some(Ok(s)) = ssr_site.get() { Some(s.site_view.site.icon.clone()) } else { None } });
        let logged_in = Memo::new(move |_| { if let Some(Ok(GetSiteResponse { my_user: Some(m), .. })) = ssr_site.get() { true } else { false } });

        view! {
          <nav class="flex flex-row py-0 navbar">
            <div class={move || { (if input_focus.get() == InputFocusState::Search { "hidden" } else { "flex" }).to_string() }}>
              <div class={move || { if input_focus.get() == InputFocusState::Instance { "" } else { "hidden" } }}>
                <input
                  class="pl-2 h-full text-xl sm:pl-6 w-46 input"
                  type="text"
                  name="instance"
                  node_ref={instance_input}
                  prop:value={move || instance_term.get()}
                  on:keydown={move |e: KeyboardEvent| {
                    if e.key_code() == 27 {
                      e.prevent_default();
                      input_focus.set(InputFocusState::Unfocused);
                    }
                  }}
                  on:keypress={move |e: KeyboardEvent| {
                    if e.key_code() == 13 {
                      e.prevent_default();
                      on_instance_submit()
                    }
                  }}
                  on:input={move |ev| {
                    instance_term.set(Some(event_target_value(&ev)));
                  }}
                  on:blur={move |_| {
                    input_focus.set(InputFocusState::Unfocused);
                  }}
                />
              </div>
              <ul class="flex-nowrap items-center menu menu-horizontal">
                <li>
                  <a
                    href="/"
                    class={move || {
                      if input_focus.get() == InputFocusState::Instance { "hidden" } else { "select-none text-xl whitespace-nowrap py-1/2" }
                    }}
                    on:mousedown={on_mouse_down}
                    on:mouseup={on_mouse_up}
                    on:touchstart={on_touch_start}
                    on:touchend={on_touch_end}
                    on:click={move |e: MouseEvent| {
                      e.prevent_default();
                      if input_focus.get() == InputFocusState::Instance {
                        input_focus.set(InputFocusState::Unfocused);
                      } else {
                        next_page_cursor.set((0, None));
                        #[cfg(not(feature = "ssr"))]
                        spawn_local_scoped(async move {
                          if let Ok(d) = IndexedDb::new().await {
                            let _ = d
                              .set(
                                &ScrollPositionKey {
                                  path: "/".into(),
                                  query: "".into(),
                                },
                                &0i32,
                              )
                              .await;
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
                                  get_auth_cookie.get(),
                                  false,
                                ),
                              );
                            });
                          use_navigate()("/", Default::default());
                        });
                        if let Some(on_scroll_element) = scroll_element.get() {
                          if let Some(se) = on_scroll_element.get() {
                            se.set_scroll_left(0i32);
                          }
                        }
                      }
                    }}
                  >
                    {move || {
                      if let Some(Some(i)) = icon_details.get() {
                        view! { <img class="h-8 sm:hidden" src={i.inner().to_string()} /> }.into_any()
                      } else {
                        view! { <img class="h-8" src="/favicon.png" /> }.into_any()
                      }
                    }}
                    <span class="hidden sm:flex">
                      {move || {
                        let m = site_details.get();
                        if m.len() > 0 { m } else { "A.O.S".to_owned() }
                      }}
                    </span>
                  </a>
                </li>
                <li class="hidden sm:flex">
                  <details node_ref={lg_filter_menu}>
                    <summary on:click={move |_| reset_others(MenuType::LgFilter)}>
                      <Icon icon={Community} />
                    </summary>
                    <ul class="z-[1] bg-base-200">
                      <li>
                        <a
                          href={on_filter_ssr(ListingType::All)}
                          class={move || highlight_csr_filter(ListingType::All)}
                          on:click={on_csr_filter_click(ListingType::All)}
                        >
                          "All"
                        </a>
                      </li>
                      <li>
                        <a
                          href={on_filter_ssr(ListingType::Local)}
                          class={move || highlight_csr_filter(ListingType::Local)}
                          on:click={on_csr_filter_click(ListingType::Local)}
                        >
                          "Local"
                        </a>
                      </li>
                      <li>
                        <a
                          href={on_filter_ssr(ListingType::Subscribed)}
                          class={move || highlight_csr_filter(ListingType::Subscribed)}
                          on:click={on_csr_filter_click(ListingType::Subscribed)}
                        >
                          "Subscribed"
                        </a>
                      </li>
                    </ul>
                  </details>
                </li>
                <li class="hidden sm:flex">
                  <details node_ref={lg_sort_menu}>
                    <summary on:click={move |_| reset_others(MenuType::LgSort)}>
                      <Icon icon={Sort} />
                    </summary>
                    <ul class="z-[1] bg-base-200">
                      <li>
                        <a
                          class={move || { (if SortType::Active == ssr_sort() { "menu-active" } else { "" }).to_string() }}
                          href={on_sort_ssr(SortType::Active)}
                          on:click={on_sort_click(SortType::Active)}
                        >
                          "Active"
                        </a>
                      </li>
                      <li>
                        <a
                          class={move || { (if SortType::TopAll == ssr_sort() { "menu-active" } else { "" }).to_string() }}
                          href={on_sort_ssr(SortType::TopAll)}
                          on:click={on_sort_click(SortType::TopAll)}
                        >
                          "Top"
                        </a>
                      </li>
                      <li>
                        <a
                          class={move || { (if SortType::Hot == ssr_sort() { "menu-active" } else { "" }).to_string() }}
                          href={on_sort_ssr(SortType::Hot)}
                          on:click={on_sort_click(SortType::Hot)}
                        >
                          "Hot"
                        </a>
                      </li>
                      <li>
                        <a
                          class={move || { (if SortType::New == ssr_sort() { "menu-active" } else { "" }).to_string() }}
                          href={on_sort_ssr(SortType::New)}
                          on:click={on_sort_click(SortType::New)}
                        >
                          "New"
                        </a>
                      </li>
                      <li>
                        <a
                          class={move || { (if SortType::Old == ssr_sort() { "menu-active" } else { "" }).to_string() }}
                          href={on_sort_ssr(SortType::Old)}
                          on:click={on_sort_click(SortType::Old)}
                        >
                          "Old"
                        </a>
                      </li>
                      <li>
                        <a
                          class={move || { (if SortType::Controversial == ssr_sort() { "menu-active" } else { "" }).to_string() }}
                          href={on_sort_ssr(SortType::Controversial)}
                          on:click={on_sort_click(SortType::Controversial)}
                        >
                          "Controversial"
                        </a>
                      </li>
                      <li>
                        <a
                          class={move || { (if SortType::Scaled == ssr_sort() { "menu-active" } else { "" }).to_string() }}
                          href={on_sort_ssr(SortType::Scaled)}
                          on:click={on_sort_click(SortType::Scaled)}
                        >
                          "Scaled"
                        </a>
                      </li>
                      <li>
                        <a
                          class={move || { (if SortType::NewComments == ssr_sort() { "menu-active" } else { "" }).to_string() }}
                          href={on_sort_ssr(SortType::NewComments)}
                          on:click={on_sort_click(SortType::NewComments)}
                        >
                          "Comment"
                        </a>
                      </li>
                    </ul>
                  </details>
                </li>
                <li class="hidden sm:flex">
                  <details node_ref={lg_kind_menu}>
                    <summary on:click={move |_| reset_others(MenuType::LgKind)}>
                      <Icon icon={Cube} />
                    </summary>
                    <ul class="z-[1] bg-base-200">
                      <li>
                        <a
                          href={on_kind_ssr(KindType::Post)}
                          class={move || highlight_csr_kind(KindType::Post)}
                          on:click={on_csr_kind_click(KindType::Post)}
                        >
                          "Posts"
                        </a>
                      </li>
                      <li>
                        <a
                          href={on_kind_ssr(KindType::Comment)}
                          class={move || highlight_csr_kind(KindType::Comment)}
                          on:click={on_csr_kind_click(KindType::Comment)}
                        >
                          "Comments"
                        </a>
                      </li>
                      <li>
                        <a
                          href={on_kind_ssr(KindType::Community)}
                          class={move || highlight_csr_kind(KindType::Community)}
                          on:click={on_csr_kind_click(KindType::Community)}
                        >
                          "Communities"
                        </a>
                      </li>
                      <li>
                        <a
                          href={on_kind_ssr(KindType::User)}
                          class={move || highlight_csr_kind(KindType::User)}
                          on:click={on_csr_kind_click(KindType::User)}
                        >
                          "Users"
                        </a>
                      </li>
                    </ul>
                  </details>
                </li>
                <li class="flex sm:hidden">
                  <details node_ref={sm_content_menu}>
                    <summary on:click={move |_| reset_others(MenuType::SmContent)}>
                      <Icon icon={Filter} />
                    </summary>
                    <ul class="z-[1] bg-base-200">
                      <li class="flex">
                        <details>
                          <summary>
                            <Icon icon={Community} />
                          </summary>
                          <ul>
                            <li>
                              <a
                                href={on_filter_ssr(ListingType::All)}
                                class={move || highlight_csr_filter(ListingType::All)}
                                on:click={on_csr_filter_click(ListingType::All)}
                              >
                                "All"
                              </a>
                            </li>
                            <li>
                              <a
                                href={on_filter_ssr(ListingType::Local)}
                                class={move || highlight_csr_filter(ListingType::Local)}
                                on:click={on_csr_filter_click(ListingType::Local)}
                              >
                                "Local"
                              </a>
                            </li>
                            <li>
                              <a
                                href={on_filter_ssr(ListingType::Subscribed)}
                                class={move || highlight_csr_filter(ListingType::Subscribed)}
                                on:click={on_csr_filter_click(ListingType::Subscribed)}
                              >
                                "Subscribed"
                              </a>
                            </li>
                          </ul>
                        </details>
                      </li>
                      <li class="flex">
                        <details>
                          <summary>
                            <Icon icon={Sort} />
                          </summary>
                          <ul>
                            <li>
                              <a
                                class={move || { (if SortType::Active == ssr_sort() { "menu-active" } else { "" }).to_string() }}
                                href={on_sort_ssr(SortType::Active)}
                                on:click={on_sort_click(SortType::Active)}
                              >
                                "Active"
                              </a>
                            </li>
                            <li>
                              <a
                                class={move || { (if SortType::TopAll == ssr_sort() { "menu-active" } else { "" }).to_string() }}
                                href={on_sort_ssr(SortType::TopAll)}
                                on:click={on_sort_click(SortType::TopAll)}
                              >
                                "Top"
                              </a>
                            </li>
                            <li>
                              <a
                                class={move || { (if SortType::Hot == ssr_sort() { "menu-active" } else { "" }).to_string() }}
                                href={on_sort_ssr(SortType::Hot)}
                                on:click={on_sort_click(SortType::Hot)}
                              >
                                "Hot"
                              </a>
                            </li>
                            <li>
                              <a
                                class={move || { (if SortType::New == ssr_sort() { "menu-active" } else { "" }).to_string() }}
                                href={on_sort_ssr(SortType::New)}
                                on:click={on_sort_click(SortType::New)}
                              >
                                "New"
                              </a>
                            </li>
                            <li>
                              <a
                                class={move || { (if SortType::Old == ssr_sort() { "menu-active" } else { "" }).to_string() }}
                                href={on_sort_ssr(SortType::Old)}
                                on:click={on_sort_click(SortType::Old)}
                              >
                                "Old"
                              </a>
                            </li>
                            <li>
                              <a
                                class={move || { (if SortType::Controversial == ssr_sort() { "menu-active" } else { "" }).to_string() }}
                                href={on_sort_ssr(SortType::Controversial)}
                                on:click={on_sort_click(SortType::Controversial)}
                              >
                                "Controversial"
                              </a>
                            </li>
                            <li>
                              <a
                                class={move || { (if SortType::Scaled == ssr_sort() { "menu-active" } else { "" }).to_string() }}
                                href={on_sort_ssr(SortType::Scaled)}
                                on:click={on_sort_click(SortType::Scaled)}
                              >
                                "Scaled"
                              </a>
                            </li>
                            <li>
                              <a
                                class={move || { (if SortType::NewComments == ssr_sort() { "menu-active" } else { "" }).to_string() }}
                                href={on_sort_ssr(SortType::NewComments)}
                                on:click={on_sort_click(SortType::NewComments)}
                              >
                                "Comment"
                              </a>
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
              <ActionForm attr:class="flex flex-grow" action={search_action}>
                <div class={move || {
                  (if input_focus.get() == InputFocusState::Search { "form-control flex flex-grow" } else { "form-control hidden sm:flex flex-grow" })
                    .to_string()
                }}>
                  <input
                    title={move || display_title.get()}
                    class="w-full input"
                    type="text"
                    name="term"
                    prop:value={move || display_title.get()}
                    node_ref={search_input}
                    on:keypress={move |e: KeyboardEvent| {
                      if e.key_code() == 13 {
                        e.prevent_default();
                        use_navigate()(&format!("/s?term={}", search_term.get()), NavigateOptions::default());
                      }
                    }}
                    on:input={move |ev| {
                      search_term.set(event_target_value(&ev));
                    }}
                    on:blur={move |_| {
                      let _ = set_timeout_with_handle(
                          move || {
                            input_focus.set(InputFocusState::Unfocused);
                          },
                          std::time::Duration::new(0, 750_000_000),
                        )
                        .ok();
                    }}
                  />
                </div>
                <div class="flex-none">
                  <button
                    type="submit"
                    class={move || { (if input_focus.get() == InputFocusState::Search { "hidden" } else { "py-2 px-4 sm:hidden" }).to_string() }}
                    on:click={move |e| {
                      e.prevent_default();
                      if input_focus.get() == InputFocusState::Search {
                        input_focus.set(InputFocusState::Unfocused);
                      } else {
                        input_focus.set(InputFocusState::Search);
                      }
                      let _ = set_timeout_with_handle(
                          move || {
                            search_input.get().map(|i| i.focus());
                          },
                          std::time::Duration::from_millis(0),
                        )
                        .ok();
                    }}
                  >
                    <Icon icon={Search} />
                  </button>
                  <button
                    class={move || {
                      (if input_focus.get() == InputFocusState::Search { "py-2 px-4" } else { "hidden sm:inline-block sm:py-2 sm:px-4" }).to_string()
                    }}
                    on:click={move |_| {
                      use_navigate()(&format!("/s?term={}", search_term.get()), NavigateOptions::default());
                    }}
                  >
                    <Icon icon={Search} />
                  </button>
                </div>
              </ActionForm>
            </div>
            <div class={move || { format!("{}", if input_focus.get() == InputFocusState::Search { "hidden" } else { "flex-none" }) }}>
              <ul class="flex-nowrap items-center menu menu-horizontal">
                <li class="hidden sm:flex">
                  <details node_ref={lg_ban_menu}>
                    <summary on:click={move |_| reset_others(MenuType::LgBan)}>
                      <Icon icon={Ban} />
                    </summary>
                    <ul class="z-[1] [inset-inline-end:0] bg-base-200">
                      <li>
                        <a
                          href={on_bans_ssr(BanType::Nsfw)}
                          class={move || highlight_csr_bans(BanType::Nsfw)}
                          on:click={on_csr_bans_click(BanType::Nsfw)}
                        >
                          "NSFW"
                        </a>
                      </li>
                      <li>
                        <a
                          href={on_bans_ssr(BanType::Ban)}
                          class={move || highlight_csr_bans(BanType::Ban)}
                          on:click={on_csr_bans_click(BanType::Ban)}
                        >
                          "Banned"
                        </a>
                      </li>
                    </ul>
                  </details>
                </li>
                <li class="hidden sm:flex">
                  <details node_ref={lg_language_menu}>
                    <summary on:click={move |_| reset_others(MenuType::LgLanguage)}>
                      <Icon icon={Translate} />
                    </summary>
                    <ul class="z-[1] [inset-inline-end:0] bg-base-200">
                      <li>
                        // <ActionForm attr:class="p-0" action={lang_action}>
                        // <input type="hidden" name="lang" value="FR" />
                        <button disabled class="py-2 px-4" type="submit">
                          "FR"
                        </button>
                      // </ActionForm>
                      </li>
                      <li>
                        // <ActionForm attr:class="p-0" action={lang_action}>
                        // <input type="hidden" name="lang" value="EN" />
                        <button disabled class="py-2 px-4" type="submit">
                          "EN"
                        </button>
                      // </ActionForm>
                      </li>
                    </ul>
                  </details>
                </li>
                <li class="hidden sm:flex">
                  <details node_ref={lg_theme_menu}>
                    <summary on:click={move |_| reset_others(MenuType::LgTheme)}>
                      <Icon icon={Palette} />
                    </summary>
                    <ul class="z-[1] [inset-inline-end:0] bg-base-200">
                      <li data-theme="dark">
                        <ActionForm attr:class="p-0" action={change_theme}>
                          <input type="hidden" name="theme" value="dark" />
                          <button class="py-2 px-4" type="submit" on:click={on_theme_submit("dark")}>
                            "Dark"
                          </button>
                        </ActionForm>
                      </li>
                      <li data-theme="light">
                        <ActionForm attr:class="p-0" action={change_theme}>
                          <input type="hidden" name="theme" value="light" />
                          <button class="py-2 px-4" type="submit" on:click={on_theme_submit("light")}>
                            "Light"
                          </button>
                        </ActionForm>
                      </li>
                      <li data-theme="retro">
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
                  when={move || { logged_in.get() }}
                  fallback={move || {
                    view! {
                      <li>
                        <form method="post" action="/l" class="p-0">
                          <input
                            type="hidden"
                            name="source_url"
                            value={move || format!("{}{}", use_location().pathname.get(), query.get().to_query_string())}
                          />
                          <button
                            type="submit"
                            class="py-2 px-4"
                            on:click={move |e: MouseEvent| {
                              e.prevent_default();
                              let passed = expect_context::<RwSignal<PassedUrl>>();
                              passed.set(PassedUrl(Some(format!("{}{}", use_location().pathname.get(), query.get().to_query_string()))));
                              use_navigate()("/l", NavigateOptions::default());
                            }}
                          >
                            <Icon icon={SignIn} />
                          </button>
                        </form>
                      </li>
                    }
                  }}
                >
                  <li>
                    <details node_ref={sm_ui_menu}>
                      <summary on:click={move |_| reset_others(MenuType::SmUi)}>
                        <Icon icon={User} />
                      </summary>
                      <ul class="z-[1] [inset-inline-end:0] bg-base-200">
                        <li class="flex sm:hidden">
                          <details>
                            <summary>
                              <Icon icon={Palette} />
                            </summary>
                            <ul class="[inset-inline-end:0]">
                              <li data-theme="dark">
                                <ActionForm attr:class="p-0" action={change_theme}>
                                  <input type="hidden" name="theme" value="dark" />
                                  <button data-theme="dark" class="py-2 px-4" type="submit" on:click={on_theme_submit("dark")}>
                                    "Dark"
                                  </button>
                                </ActionForm>
                              </li>
                              <li data-theme="light">
                                <ActionForm attr:class="p-0" action={change_theme}>
                                  <input type="hidden" name="theme" value="light" />
                                  <button data-theme="light" class="py-2 px-4" type="submit" on:click={on_theme_submit("light")}>
                                    "Light"
                                  </button>
                                </ActionForm>
                              </li>
                              <li data-theme="retro">
                                <ActionForm attr:class="p-0" action={change_theme}>
                                  <input type="hidden" name="theme" value="retro" />
                                  <button data-theme="retro" class="py-2 px-4" type="submit" on:click={on_theme_submit("retro")}>
                                    "Retro"
                                  </button>
                                </ActionForm>
                              </li>
                            </ul>
                          </details>
                        </li>
                        <li>
                          <ActionForm attr:class="p-0" action={logout_action}>
                            <button type="submit" class="py-2 px-4" on:click={on_logout_submit}>
                              <Icon icon={SignOut} />
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
          .into_any()
      }}
    </Transition>
  }
}
