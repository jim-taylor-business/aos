use crate::prelude::*;

#[component]
pub fn UserListings(people: Signal<Vec<PersonView>>, page_number: RwSignal<usize>, hide: bool, #[prop(optional)] heroes: usize) -> impl IntoView {
  let post_number = RwSignal::new(page_number.get());
  view! {
    <For each={move || people.get()} key={|pv| pv.person.id} let:pv>
      {
        post_number.update(|n| *n = *n + 1usize);
        view! { <UserListing hide person_view={pv} post_number={post_number.get()} /> }.into_any()
      }
    </For>
  }
  .into_any()
}

#[component]
pub fn UserListing(person_view: PersonView, post_number: usize, hide: bool) -> impl IntoView {
  let i18n = use_i18n();
  let ReadInstanceCookie(get_instance_cookie) = expect_context::<ReadInstanceCookie>();
  let thumbnail = RwSignal::new(String::from(""));
  let host = person_view.person.actor_id.clone().inner().host_str().map(str::to_owned);
  let from = Memo::new(move |_| external_host_attribution(host.clone(), get_instance_cookie.get()));
  let name = html_escape::encode_safe(&person_view.person.display_name.unwrap_or(person_view.person.name.clone())).to_string();
  let a = person_view.person.actor_id;
  let u = person_view.person.name.clone();
  let creator_name = Memo::new(move |_| federated_creator_handle(&u, a.domain().as_deref(), None));
  let creator_name_encoded = Memo::new(move |_| html_escape::encode_safe(&name).to_string());
  let published_at = person_view.person.published.timestamp_millis();
  let abbr_duration = Memo::new(move |_| relative_time(published_at));

  view! {
    <div class="px-4 pb-6 flex gap-x-4 break-inside-avoid">
      <div class="flex items-start pt-2">
        <A
          attr:class="flex flex-col h-full"
          href={move || format!("/u/{}?person_id={}", creator_name.get(), person_view.person.id.0)}
        >
          {move || {
            if let Some(ref t) = person_view.person.avatar {
              let h = t.inner().to_string();
              thumbnail.set(h);
              view! {
                <div class="flex shrink grow basis-0 min-h-16">
                  <div class="shrink grow basis-0 truncate w-24">
                    <img
                      class={move || format!("w-24{}", if thumbnail.get().eq(&"/lemmy.svg".to_owned()) { " h-16" } else { "" })}
                      src={move || thumbnail.get()}
                      on:error={move |_e| {
                        thumbnail.set("/lemmy.svg".into());
                      }}
                    />
                  </div>
                </div>
              }.into_any()
            } else {
              view! {
                <div class="block w-24 truncate">
                  <img class="w-24 h-16" src="/lemmy.svg" />
                </div>
              }.into_any()
            }
          }}
        </A>
      </div>
      <div>
        <A href={move || format!("/u/{}?person_id={}", creator_name.get(), person_view.person.id.0)} attr:class="block hover:text-accent">
          <span class="overflow-y-auto text-xl wrap-anywhere" inner_html={creator_name_encoded.get()} />
        </A>
        <span class="block mt-1 mb-1 text-sm wrap-anywhere">
          <span>{abbr_duration}</span>
          " "
          <span>{from}</span>
        </span>
        <div class="flex flex-wrap">
          <span
            class="flex items-center"
            title={move || t_string!(i18n, post_count, count = person_view.counts.post_count)}
          >
            <Icon icon={Icon::Archive} class="inline" />
            <span class="text-sm">{person_view.counts.post_count}</span>
          </span>
          <span
            class="flex items-center pl-2"
            title={move || t_string!(i18n, comment_count, count = person_view.counts.comment_count)}
          >
            <Icon icon={Icon::Comments} class="inline" />
            <span class="text-sm">{person_view.counts.comment_count}</span>
          </span>
        </div>
      </div>
    </div>
  }
  .into_any()
}

#[component]
pub fn CommunityListings(
  communities: Signal<Vec<CommunityView>>,
  page_number: RwSignal<usize>,
  hide: bool,
  #[prop(optional)] heroes: usize,
) -> impl IntoView {
  let post_number = RwSignal::new(page_number.get());
  view! {
    <For each={move || communities.get()} key={|pv| pv.community.id} let:pv>
      {
        post_number.update(|n| *n = *n + 1usize);
        view! { <CommunityListing hide community={pv} post_number={post_number.get()} /> }.into_any()
      }
    </For>
  }
  .into_any()
}

#[component]
pub fn CommunityListing(community: CommunityView, post_number: usize, hide: bool) -> impl IntoView {
  let i18n = use_i18n();
  let ReadInstanceCookie(get_instance_cookie) = expect_context::<ReadInstanceCookie>();
  let thumbnail = RwSignal::new(String::from(""));

  let host = community.community.actor_id.inner().host_str().map(str::to_owned);
  let from = Memo::new(move |_| external_host_attribution(host.clone(), get_instance_cookie.get()));
  let title = if community.community.title.len() > 0 { community.community.title } else { community.community.name.clone() };
  let community_title =
    community_title_with_federation(&title, community.community.local, community.community.actor_id.inner().host().map(|host| host.to_string()));
  let community_title_encoded = Memo::new(move |_| html_escape::encode_safe(&community_title).to_string());
  let community_name = Memo::new(move |_| {
    if community.community.local {
      format!("{}", community.community.name)
    } else {
      format!(
        "{}@{}",
        community.community.name,
        if let Some(h) = community.community.actor_id.inner().host() { h.to_string() } else { "".to_owned() }
      )
    }
  });

  let published_at = community.community.published.timestamp_millis();
  let abbr_duration = Memo::new(move |_| relative_time(published_at));

  view! {
    <div class="px-4 pb-6 flex gap-x-4 break-inside-avoid">
      <div class="flex items-start pt-2">
        <A
          attr:class="flex flex-col h-full"
          href={move || format!("/c/{}", community_name.get())}
        >
          {move || {
            if let Some(ref t) = community.community.icon {
              let h = t.inner().to_string();
              thumbnail.set(h);
              view! {
                <div class="flex shrink grow basis-0 min-h-16">
                  <div class="shrink grow basis-0 truncate w-24">
                    <img
                      class={move || format!("w-24{}", if thumbnail.get().eq(&"/lemmy.svg".to_owned()) { " h-16" } else { "" })}
                      src={move || thumbnail.get()}
                      on:error={move |_e| {
                        thumbnail.set("/lemmy.svg".into());
                      }}
                    />
                  </div>
                </div>
              }.into_any()
            } else {
              view! {
                <div class="block w-24 truncate">
                  <img class="w-24 h-16" src="/lemmy.svg" />
                </div>
              }.into_any()
            }
          }}
        </A>
      </div>
      <div>
        <A href={move || format!("/c/{}", community_name.get())} attr:class="block hover:text-accent">
          <span class="overflow-y-auto text-xl wrap-anywhere" inner_html={community_title_encoded.get()} />
        </A>
        <span class="block mt-1 mb-1 text-sm wrap-anywhere">
          <span>{abbr_duration}</span>
          " "
          <span>{from}</span>
        </span>
        <div class="flex flex-wrap">
          <span
            class="flex items-center"
            title={move || t_string!(i18n, subscribers_count, count = community.counts.subscribers)}
          >
            <Icon icon={Icon::Subscribe} class="inline" />
            <span class="text-sm">{community.counts.subscribers}</span>
          </span>
          <span
            class="flex items-center pl-2"
            title={move || t_string!(i18n, active_users_count, count = community.counts.users_active_month)}
          >
            <Icon icon={Icon::Community} class="inline" />
            <span class="text-sm">{community.counts.users_active_month}</span>
          </span>
          <span
            class="flex items-center pl-2"
            title={move || t_string!(i18n, posts_count, count = community.counts.posts)}
          >
            <Icon icon={Icon::Archive} class="inline" />
            <span class="text-sm">{community.counts.posts}</span>
          </span>
          <span
            class="flex items-center pl-2"
            title={move || t_string!(i18n, comments_count_short, count = community.counts.comments)}
          >
            <Icon icon={Icon::Comments} class="inline" />
            <span class="text-sm">{community.counts.comments}</span>
          </span>
        </div>
      </div>
    </div>
  }
  .into_any()
}

#[component]
pub fn Listings(posts: Signal<Vec<PostView>>, page_number: RwSignal<usize>, hide: bool, #[prop(optional)] heroes: usize) -> impl IntoView {
  let post_number = RwSignal::new(page_number.get());
  view! {
    <For each={move || posts.get()} key={|pv| pv.post.id} let:pv>
      {
        post_number.update(|n| *n = *n + 1usize);
        if post_number.get() <= heroes {
          view! { <Hero hide post_id={Signal::derive(move || pv.post.id)} /> }.into_any()
        } else {
          view! { <Listing hide post_view={pv} post_number={post_number.get()} /> }.into_any()
        }
      }
    </For>
  }
  .into_any()
}

#[server]
pub async fn vote_post(post_id: i32, score: i16) -> Result<Option<PostResponse>, ServerFnError> {
  provide_cookies();
  let form = CreatePostLike { post_id: PostId(post_id), score };
  let result = LemmyClient.like_post(form).await;
  use leptos_axum::redirect;
  match result {
    Ok(o) => Ok(Some(o)),
    Err(e) => {
      redirect(&format!("/?error={}", serde_json::to_string(&e)?)[..]);
      Ok(None)
    }
  }
}

#[server]
pub async fn post_save(post_id: i32, save: bool) -> Result<Option<PostResponse>, ServerFnError> {
  provide_cookies();
  let form = SavePost { post_id: PostId(post_id), save };
  let result = LemmyClient.save_post(form).await;
  use leptos_axum::redirect;
  match result {
    Ok(o) => Ok(Some(o)),
    Err(e) => {
      redirect(&format!("/?error={}", serde_json::to_string(&e)?)[..]);
      Ok(None)
    }
  }
}

#[server]
pub async fn block_user(person_id: i32, block: bool) -> Result<Option<BlockPersonResponse>, ServerFnError> {
  provide_cookies();
  let form = BlockPerson { person_id: PersonId(person_id), block };
  let result = LemmyClient.block_user(form).await;
  use leptos_axum::redirect;
  match result {
    Ok(o) => Ok(Some(o)),
    Err(e) => {
      redirect(&format!("/?error={}", serde_json::to_string(&e)?)[..]);
      Ok(None)
    }
  }
}

fn validate_report(form: &CreatePostReport) -> Option<LemmyAppErrorType> {
  if form.reason.is_empty() {
    return Some(LemmyAppErrorType::MissingReason);
  }
  None
}

async fn try_report(form: CreatePostReport) -> Result<PostReportResponse, LemmyAppError> {
  let val = validate_report(&form);
  match val {
    None => {
      let result = LemmyClient.report_post(form).await;
      match result {
        Ok(o) => Ok(o),
        Err(e) => Err(e),
      }
    }
    Some(e) => Err(LemmyAppError { error_type: e.clone(), content: format!("{}", form.post_id.0) }),
  }
}

#[server]
pub async fn report_post(post_id: i32, reason: String) -> Result<Option<PostReportResponse>, ServerFnError> {
  provide_cookies();
  let form = CreatePostReport { post_id: PostId(post_id), reason };
  let result = try_report(form).await;
  use leptos_axum::redirect;
  match result {
    Ok(o) => Ok(Some(o)),
    Err(e) => {
      redirect(&format!("/?error={}", serde_json::to_string(&e)?)[..]);
      Ok(None)
    }
  }
}

#[component]
pub fn Listing(post_view: PostView, post_number: usize, /*reply_show: RwSignal<bool>, */ hide: bool) -> impl IntoView {
  let i18n = use_i18n();
  let ssr_site = expect_context::<Resource<Result<GetSiteResponse, LemmyAppError>>>();
  let online = expect_context::<RwSignal<OnlineSetter>>();
  let ReadInstanceCookie(get_instance_cookie) = expect_context::<ReadInstanceCookie>();
  let post_view = RwSignal::new(post_view);
  let loading = RwSignal::new(false);
  let error = RwSignal::new(false);
  let latest_error = RwSignal::new(LemmyAppError { error_type: LemmyAppErrorType::Unknown, content: "".to_owned() });
  let vote_action = ServerAction::<VotePost>::new();

  let on_vote_submit = move |e: MouseEvent, score: i16| {
    e.prevent_default();
    Resource::new(
      move || (),
      move |()| async move {
        let form = CreatePostLike { post_id: post_view.get().post.id, score };
        let result = LemmyClient.like_post(form).await;
        #[cfg(not(feature = "ssr"))]
        loading.set(false);
        match result {
          Ok(o) => {
            post_view.set(o.post_view);
            let post_response_cache = expect_context::<RwSignal<BTreeMap<(GetPost, Option<String>), (i64, LemmyAppResult<GetPostResponse>)>>>();
            post_response_cache.update(|prc| {
              prc.iter_mut().for_each(|(f, r)| {
                if let (_, Ok(w)) = r {
                  let id = post_view.get().post.id.0;
                  if w.post_view.post.id.0 == id {
                    w.post_view = post_view.get();
                  }
                }
              });
            });
            let response_cache =
              expect_context::<RwSignal<BTreeMap<(usize, GetPosts, Option<String>, bool), (i64, LemmyAppResult<GetPostsResponse>)>>>();
            response_cache.update(|rc| {
              rc.iter_mut().for_each(|(f, r)| {
                if let (_, Ok(w)) = r {
                  let id = post_view.get().post.id.0;
                  for p in w.posts.iter_mut() {
                    if p.post.id.0 == id {
                      *p = post_view.get();
                      let result_clone = r.1.clone();
                      let form_clone = f.1.clone();
                      spawn_local_scoped(async move {
                        if let Ok(d) = IndexedDb::new().await {
                          if let Ok(_c) = d.set::<GetPosts, Result<GetPostsResponse, LemmyAppError>>(&form_clone, &result_clone).await {}
                        }
                      });
                      break;
                    }
                  }
                }
              });
            });
          }
          Err(e) => {
            error.set(true);
            latest_error.set(e);
          }
        }
      },
    );
  };

  let on_up_vote_submit = move |e: MouseEvent| {
    #[cfg(not(feature = "ssr"))]
    loading.set(true);
    let score = if Some(1) == post_view.get().my_vote { 0 } else { 1 };
    on_vote_submit(e, score);
  };

  let _on_down_vote_submit = move |e: MouseEvent| {
    let score = if Some(-1) == post_view.get().my_vote { 0 } else { -1 };
    on_vote_submit(e, score);
  };

  let _save_post_action = ServerAction::<PostSave>::new();

  let _on_save_submit = move |e: MouseEvent| {
    e.prevent_default();
    Resource::new(
      move || (),
      move |()| async move {
        let form = SavePost { post_id: post_view.get().post.id, save: !post_view.get().saved };
        let result = LemmyClient.save_post(form).await;
        match result {
          Ok(o) => {
            post_view.set(o.post_view);
          }
          Err(_e) => {}
        }
      },
    );
  };

  let _block_user_action = ServerAction::<BlockUser>::new();

  let _on_block_submit = move |e: MouseEvent| {
    e.prevent_default();
    Resource::new(
      move || (),
      move |()| async move {
        let form = BlockPerson { person_id: post_view.get().creator.id, block: true };
        let result = LemmyClient.block_user(form).await;
        match result {
          Ok(_o) => {}
          Err(_e) => {}
        }
      },
    );
  };

  let _report_post_action = ServerAction::<ReportPost>::new();
  let report_validation = RwSignal::new(String::from(""));

  let query = use_query_map();
  let ssr_error = move || query.with(|params| params.get("error"));

  if let Some(e) = ssr_error() {
    let le = serde_json::from_str::<LemmyAppError>(&e[..]);
    match le {
      Ok(e) => match e {
        LemmyAppError { error_type: LemmyAppErrorType::MissingReason, content: c } => {
          let id = format!("{}", post_view.get().post.id);
          if c.eq(&id) {
            report_validation.set("input-error".to_owned());
          }
        }
        _ => {
          report_validation.set("".to_owned());
        }
      },
      Err(_) => {
        error!("error decoding error - log and ignore in UI?");
      }
    }
  }

  let reason = RwSignal::new(String::new());

  let _on_report_submit = move |e: MouseEvent| {
    e.prevent_default();
    Resource::new(
      move || (),
      move |()| async move {
        let form = CreatePostReport { post_id: post_view.get().post.id, reason: reason.get() };
        let result = try_report(form).await;
        match result {
          Ok(_o) => {}
          Err(e) => {
            let _id = format!("{}", post_view.get().post.id);
            match e {
              LemmyAppError { error_type: LemmyAppErrorType::MissingReason, content: _id } => {
                report_validation.set("input-error".to_owned());
              }
              _ => {
                report_validation.set("".to_owned());
              }
            }
          }
        }
      },
    );
  };

  let title = post_view.get().post.name.clone();
  let title_encoded_mut = markdown_to_html(&title, false);
  let title_encoded = Memo::new(move |_| title_encoded_mut.clone());
  let community_title = community_title_with_federation(
    &post_view.get().community.name,
    post_view.get().community.local,
    post_view.get().community.actor_id.inner().host().map(|host| host.to_string()),
  );
  let community_title_encoded = Memo::new(move |_| html_escape::encode_safe(&community_title).to_string());
  let a = post_view.get().creator.actor_id;
  let u = post_view.get().creator.name;
  let instance = get_instance_cookie.get_untracked();
  let creator_name = federated_creator_handle(&u, a.domain().as_deref(), instance.as_deref());
  let creator_name_encoded = Memo::new(move |_| html_escape::encode_safe(&creator_name).to_string());
  let abbr_duration = Memo::new(move |_| relative_time(post_view.get().post.published.timestamp_millis()));
  let thumbnail = RwSignal::new(String::from(""));

  view! {
    <Transition fallback={|| {}}>
      {move || {
        match ssr_site.get() {
          Some(Ok(s)) => {
            {
              let logged_in = Memo::new(move |_| { s.my_user.is_some() });
              let languages = Memo::new(move |_| if let Some(Ok(GetSiteResponse { all_languages: l, .. })) = ssr_site.get() { l } else { vec![] });

              view! {
                <div class={move || {
                  format!(
                    "grid gap-x-4 px-4 grid-cols-[6rem_1fr] break-inside-avoid {}{}",
                    if post_number != 0 { "grid-rows-[1fr_2rem] pb-6" } else { "grid-rows-[1fr] pb-2" },
                    if hide { " invisible" } else { "" },
                  )
                }}>
                  <div class={move || {
                    format!(
                      "col-span-1 row-span-2 flex items-start pt-2{}",
                      if post_view.get_untracked().post.thumbnail_url.is_none() && post_view.get_untracked().post.url.is_none() {
                        " hidden"
                      } else {
                        ""
                      },
                    )
                  }}>
                    <a
                      class="flex flex-col h-full"
                      target="_blank"
                      href={move || {
                        if let Some(d) = post_view.get_untracked().post.url {
                          d.inner().to_string()
                        } else {
                          format!("/p/{}", post_view.get_untracked().post.id)
                        }
                      }}
                    >
                      {move || {
                        if let Some(t) = post_view.get_untracked().post.thumbnail_url {
                          let h = t.inner().to_string();
                          thumbnail.set(h);
                          view! {
                            <div class="flex shrink grow basis-0 min-h-16">
                              <div class="shrink grow basis-0 truncate">
                                <img
                                  class={move || format!("w-24{}", if thumbnail.get().eq(&"/lemmy.svg".to_owned()) { " h-16" } else { "" })}
                                  src={move || thumbnail.get()}
                                  on:error={move |_e| {
                                    thumbnail.set("/lemmy.svg".into());
                                  }}
                                />
                              </div>
                            </div>
                          }
                            .into_any()
                        } else {
                          view! {
                            <div class="block w-24 truncate">
                              <img class="w-24 h-16" src="/lemmy.svg" />
                            </div>
                          }
                            .into_any()
                        }
                      }}
                    </a>
                  </div>
                  <div class={move || {
                    format!(
                      "col-span-1 row-span-1{}",
                      if post_view.get_untracked().post.thumbnail_url.is_none() && post_view.get_untracked().post.url.is_none() {
                        " col-span-2 sm:col-span-2"
                      } else {
                        ""
                      },
                    )
                  }}>
                    <A href={move || format!("/p/{}", post_view.get_untracked().post.id)} attr:class="block hover:text-accent">
                      <span lang={move || languages.get().get(post_view.get_untracked().post.language_id.0.abs() as usize).map(|l| if l.id.0 == 0 { "".into() } else { l.code.clone() } )} class="overflow-y-auto text-xl wrap-anywhere" inner_html={title_encoded.get_untracked()} />
                    </A>
                    <span class="block mt-1 mb-1 text-sm wrap-anywhere">
                      <span>{abbr_duration}</span>
                      " "
                      <Show when={move || { post_number != 0 }} fallback={|| {}}>
                        "by "
                        <span class="overflow-y-auto" inner_html={creator_name_encoded} />
                      </Show>
                      " in "
                      <span class="overflow-y-auto" inner_html={community_title_encoded} />
                      <span class="overflow-y-auto">
                        {external_host_attribution(
                          post_view.get_untracked().post.url.and_then(|url| url.inner().host_str().map(str::to_owned)),
                          get_instance_cookie.get_untracked(),
                        )}
                      </span>
                    </span>
                  </div>
                  <Show when={move || { post_number != 0 }} fallback={|| {}}>
                    <div class={move || {
                      format!(
                        "row-span-1 flex items-center gap-x-1{}",
                        if post_view.get_untracked().post.thumbnail_url.is_none() && post_view.get_untracked().post.url.is_none() {
                          " col-span-2"
                        } else {
                          " col-span-1"
                        },
                      )
                    }}>
                      <ActionForm action={vote_action} attr:class="flex items-center">
                        <input type="hidden" name="post_id" value={format!("{}", post_view.get_untracked().post.id)} />
                        <input type="hidden" name="score" value={move || if Some(1) == post_view.get().my_vote { 0 } else { 1 }} />
                        <button
                          type="submit"
                          on:click={on_up_vote_submit}
                          class={move || {
                            format!(
                              "{}{}",
                              { if Some(1) == post_view.get().my_vote { "text-secondary" } else { "" } },
                              { if !logged_in.get() || !online.get().0 { " text-base-content/50" } else { " hover:text-secondary/50" } },
                            )
                          }}
                          disabled={move || !logged_in.get() || !online.get().0}
                          title={move || t_string!(i18n, up_vote)}
                        >
                          <Icon icon={Icon::Upvote} />
                        </button>
                      </ActionForm>
                      <span class="block text-sm">{move || post_view.get().counts.score}</span>
                      <span
                        class="flex items-center pl-1"
                        title={move || {
                          format!(
                            "{} comments{}",
                            post_view.get_untracked().counts.comments,
                            if post_view.get_untracked().unread_comments != post_view.get_untracked().counts.comments
                              && post_view.get_untracked().unread_comments > 0
                            {
                              format!(" ({} unread)", post_view.get_untracked().unread_comments)
                            } else {
                              "".to_owned()
                            },
                          )
                        }}
                      >
                        <A href={move || format!("/p/{}", post_view.get_untracked().post.id)} attr:class="inline hover:text-accent">
                          <Icon icon={Icon::Comments} class="inline" />
                        </A>
                        {post_view.get_untracked().counts.comments}
                      </span>
                      <span class="flex items-center ml-auto text-base-content/25">
                        <a
                          class={format!(
                            "{}",
                            {
                              if let Some(d) = post_view.get_untracked().post.url {
                                if let Some(f) = d.inner().host_str() {
                                  if f.to_string().ne(&get_instance_cookie.get_untracked().unwrap_or("".into())) { "" } else { " hidden" }
                                } else {
                                  " hidden"
                                }
                              } else {
                                " hidden"
                              }
                            },
                          )}
                          title={move || t_string!(i18n, archive)}
                          target="_blank"
                          href={format!(
                            "https://archive.ph/submit/?url={}",
                            { if let Some(d) = post_view.get_untracked().post.url { d.inner().to_string() } else { "".to_owned() } },
                          )}
                        >
                          <Icon icon={Icon::History} />
                        </a>
                      </span>
                      <span class="flex items-center text-base-content/25">
                        {if post_number != 0 { format!("{}", post_number) } else { "".into() }}
                      </span>
                    </div>
                  </Show>
                </div>
                {move || {
                  view! { <Info loading={loading.get()} /> }
                }}
                <Show when={move || error.get()} fallback={|| {}}>
                  <Error description={t_string!(i18n, error_upvoting_post).to_string()} error={latest_error.get()} />
                </Show>
              }
            }
              .into_any()
          }
          _ => view! {}.into_any(),
        }
      }}
    </Transition>
  }
}
