pub use crate::db::{CommentDraft, CommentDraftKey, Draft, GetCacheKey, GetPostsCacheKey, IndexedDb, IndexedDbError, ScrollPositionKey, Store};
pub use crate::i18n::*;
pub use crate::icon::Icon;
pub use crate::{App, html_template};
pub use crate::{
  client, client::*, comment, comment::*, db, errors, errors::*, hero, hero::*, icon, icon::*, listing, listing::*, login, login::*, nav, nav::*,
  overview, overview::*, post, post::*, search, search::*, toolbar, toolbar::*, user, user::*,
};
pub use codee::string::FromToStringCodec;
pub use lemmy_api_common::{
  LemmyErrorType, SuccessResponse, comment::*, community::*, lemmy_db_schema::aggregates::structs::*, lemmy_db_schema::newtypes::*,
  lemmy_db_schema::*, lemmy_db_views::structs::*, lemmy_db_views_actor::structs::*, person::*, post::*, private_message::*, site::*,
};
pub use leptos::config::LeptosOptions;
pub use leptos::{
  html::{Div, Img, Textarea},
  logging::*,
  prelude::*,
  task::*,
  *,
};
pub use leptos_dom::helpers::TimeoutHandle;
pub use leptos_meta::*;
pub use leptos_router::{components::*, hooks, hooks::*, location::State, *};
pub use leptos_use_cookie::*;
pub use send_wrapper::SendWrapper;
pub use serde::{Deserialize, Serialize, de::DeserializeOwned};
pub use std::collections::BTreeMap;
use std::marker::PhantomData;
pub use web_sys::{
  Event, HtmlAnchorElement, HtmlImageElement, KeyboardEvent, MouseEvent, SubmitEvent, TouchEvent, VisibilityState, WheelEvent, wasm_bindgen::JsCast,
};

#[derive(Clone)]
pub struct OnlineSetter(pub bool);
#[derive(Clone, Debug, PartialEq)]
pub struct NotificationsRefresh(pub bool);
#[derive(Clone, Debug, Default)]
pub struct PassedUrl(pub Option<InternalEndpointData>);
#[derive(Clone, Debug, Default)]
pub struct SelectedText(pub Option<String>);

const ROOT_SEGMENT: &str = "";
const POST_SEGMENT: &str = "p";
const COMMUNITY_SEGMENT: &str = "c";
const USER_SEGMENT: &str = "u";
const SEARCH_SEGMENT: &str = "s";
const LOGIN_SEGMENT: &str = "l";

const ID_PARAM: &str = "id";
const NAME_PARAM: &str = "name";

const LIST_QUERY: &str = "list";
const SORT_QUERY: &str = "sort";
const PAGE_QUERY: &str = "page";
const BANS_QUERY: &str = "bans";
const TERM_QUERY: &str = "term";
const KIND_QUERY: &str = "kind";
const PERSON_ID_QUERY: &str = "person_id";
const ERROR_QUERY: &str = "error";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ListingQuery {
  pub list: Option<ListingType>,
  pub sort: Option<SortType>,
  pub page: Option<Vec<(usize, String)>>,
  pub bans: Option<Vec<BanType>>,
  pub error: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SearchQuery {
  pub term: Option<String>,
  pub kind: Option<SearchType>,
  pub sort: Option<SortType>,
  pub list: Option<ListingType>,
  pub page: Option<Vec<usize>>,
  pub person_id: Option<i32>,
  pub error: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UserQuery {
  pub page: Option<Vec<usize>>,
  pub person_id: Option<i32>,
  pub error: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PostQuery {
  pub sort: Option<CommentSortType>,
  pub error: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LoginQuery {
  pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InternalEndpointData {
  Home { query: ListingQuery },
  Post { id: String, query: PostQuery },
  Community { name: String, query: ListingQuery },
  User { name: String, query: UserQuery },
  Search { query: SearchQuery },
  PostSearch { id: String, query: SearchQuery },
  CommunitySearch { name: String, query: SearchQuery },
  UserSearch { name: String, query: SearchQuery },
  Login { query: LoginQuery },
  Other { path: String },
}

fn decode_query_value<T: DeserializeOwned>(value: Option<String>) -> Option<T> {
  value.and_then(|value| serde_json::from_str(&value).ok())
}

fn append_json_param<T: Serialize>(params: &mut Vec<(String, String)>, key: &str, value: &T) {
  params.push((key.to_owned(), serde_json::to_string(value).expect("internal query values are serializable")));
}

impl InternalEndpointData {
  pub fn listing_query(&self) -> Option<&ListingQuery> {
    match self {
      Self::Home { query } | Self::Community { query, .. } => Some(query),
      _ => None,
    }
  }

  pub fn search_query(&self) -> Option<&SearchQuery> {
    match self {
      Self::Search { query } | Self::PostSearch { query, .. } | Self::CommunitySearch { query, .. } | Self::UserSearch { query, .. } => Some(query),
      _ => None,
    }
  }

  pub fn post_query(&self) -> Option<&PostQuery> {
    match self {
      Self::Post { query, .. } => Some(query),
      _ => None,
    }
  }

  pub fn user_query(&self) -> Option<&UserQuery> {
    match self {
      Self::User { query, .. } => Some(query),
      _ => None,
    }
  }

  pub fn community_name(&self) -> Option<&str> {
    match self {
      Self::Community { name, .. } | Self::CommunitySearch { name, .. } => Some(name),
      _ => None,
    }
  }

  pub fn user_name(&self) -> Option<&str> {
    match self {
      Self::User { name, .. } | Self::UserSearch { name, .. } => Some(name),
      _ => None,
    }
  }

  pub fn post_id(&self) -> Option<&str> {
    match self {
      Self::Post { id, .. } | Self::PostSearch { id, .. } => Some(id),
      _ => None,
    }
  }

  pub fn list_type(&self) -> Option<ListingType> {
    self.listing_query().and_then(|query| query.list.clone()).or_else(|| self.search_query().and_then(|query| query.list))
  }

  pub fn sort_type(&self) -> Option<SortType> {
    self.listing_query().and_then(|query| query.sort.clone()).or_else(|| self.search_query().and_then(|query| query.sort))
  }

  pub fn search_term(&self) -> Option<String> {
    self.search_query().and_then(|query| query.term.clone())
  }

  pub fn search_kind(&self) -> Option<SearchType> {
    self.search_query().and_then(|query| query.kind)
  }

  pub fn bans(&self) -> Option<Vec<BanType>> {
    self.listing_query().and_then(|query| query.bans.clone())
  }

  pub fn person_id(&self) -> Option<i32> {
    self.search_query().and_then(|query| query.person_id).or_else(|| self.user_query().and_then(|query| query.person_id))
  }

  pub fn error_query(&self) -> Option<&str> {
    match self {
      Self::Home { query } | Self::Community { query, .. } => query.error.as_deref(),
      Self::Post { query, .. } => query.error.as_deref(),
      Self::User { query, .. } => query.error.as_deref(),
      Self::Search { query } | Self::PostSearch { query, .. } | Self::CommunitySearch { query, .. } | Self::UserSearch { query, .. } => {
        query.error.as_deref()
      }
      Self::Login { query } => query.error.as_deref(),
      Self::Other { .. } => None,
    }
  }

  pub fn root_route() -> impl leptos_router::PossibleRouteMatch + Clone + Send + 'static {
    StaticSegment(ROOT_SEGMENT)
  }

  pub fn home_route() -> impl leptos_router::PossibleRouteMatch + Clone + Send + 'static {
    StaticSegment(ROOT_SEGMENT)
  }

  pub fn post_route() -> impl leptos_router::PossibleRouteMatch + Clone + Send + 'static {
    (StaticSegment(POST_SEGMENT), ParamSegment(ID_PARAM))
  }

  pub fn community_route() -> impl leptos_router::PossibleRouteMatch + Clone + Send + 'static {
    (StaticSegment(COMMUNITY_SEGMENT), ParamSegment(NAME_PARAM))
  }

  pub fn user_route() -> impl leptos_router::PossibleRouteMatch + Clone + Send + 'static {
    (StaticSegment(USER_SEGMENT), ParamSegment(NAME_PARAM))
  }

  pub fn search_route() -> impl leptos_router::PossibleRouteMatch + Clone + Send + 'static {
    StaticSegment(SEARCH_SEGMENT)
  }

  pub fn post_search_route() -> impl leptos_router::PossibleRouteMatch + Clone + Send + 'static {
    (StaticSegment(POST_SEGMENT), ParamSegment(ID_PARAM), StaticSegment(SEARCH_SEGMENT))
  }

  pub fn community_search_route() -> impl leptos_router::PossibleRouteMatch + Clone + Send + 'static {
    (StaticSegment(COMMUNITY_SEGMENT), ParamSegment(NAME_PARAM), StaticSegment(SEARCH_SEGMENT))
  }

  pub fn user_search_route() -> impl leptos_router::PossibleRouteMatch + Clone + Send + 'static {
    (StaticSegment(USER_SEGMENT), ParamSegment(NAME_PARAM), StaticSegment(SEARCH_SEGMENT))
  }

  pub fn login_route() -> impl leptos_router::PossibleRouteMatch + Clone + Send + 'static {
    StaticSegment(LOGIN_SEGMENT)
  }

  pub fn path(&self) -> String {
    match self {
      Self::Home { .. } => "/".to_owned(),
      Self::Post { id, .. } => format!("/{POST_SEGMENT}/{id}"),
      Self::Community { name, .. } => format!("/{COMMUNITY_SEGMENT}/{name}"),
      Self::User { name, .. } => format!("/{USER_SEGMENT}/{name}"),
      Self::Search { .. } => format!("/{SEARCH_SEGMENT}"),
      Self::PostSearch { id, .. } => format!("/{POST_SEGMENT}/{id}/{SEARCH_SEGMENT}"),
      Self::CommunitySearch { name, .. } => format!("/{COMMUNITY_SEGMENT}/{name}/{SEARCH_SEGMENT}"),
      Self::UserSearch { name, .. } => format!("/{USER_SEGMENT}/{name}/{SEARCH_SEGMENT}"),
      Self::Login { .. } => format!("/{LOGIN_SEGMENT}"),
      Self::Other { path } => path.clone(),
    }
  }

  pub fn search_endpoint(&self) -> Self {
    match self {
      Self::Post { id, query } => Self::PostSearch { id: id.clone(), query: SearchQuery { error: query.error.clone(), ..SearchQuery::default() } },
      Self::PostSearch { id, query } => Self::PostSearch { id: id.clone(), query: query.clone() },
      Self::Community { name, query } => Self::CommunitySearch {
        name: name.clone(),
        query: SearchQuery { sort: query.sort, list: query.list, error: query.error.clone(), ..SearchQuery::default() },
      },
      Self::CommunitySearch { name, query } => Self::CommunitySearch { name: name.clone(), query: query.clone() },
      Self::User { name, query } => {
        Self::UserSearch { name: name.clone(), query: SearchQuery { error: query.error.clone(), ..SearchQuery::default() } }
      }
      Self::UserSearch { name, query } => Self::UserSearch { name: name.clone(), query: query.clone() },
      Self::Search { query } => Self::Search { query: query.clone() },
      _ => Self::Search { query: SearchQuery::default() },
    }
  }

  pub fn set_list(&mut self, list: Option<ListingType>) {
    match self {
      Self::Home { query } | Self::Community { query, .. } => query.list = list,
      Self::Search { query } | Self::PostSearch { query, .. } | Self::CommunitySearch { query, .. } | Self::UserSearch { query, .. } => {
        query.list = list
      }
      _ => {}
    }
  }

  pub fn set_sort(&mut self, sort: Option<SortType>) {
    match self {
      Self::Home { query } | Self::Community { query, .. } => query.sort = sort,
      Self::Search { query } | Self::PostSearch { query, .. } | Self::CommunitySearch { query, .. } | Self::UserSearch { query, .. } => {
        query.sort = sort
      }
      _ => {}
    }
  }

  pub fn set_bans(&mut self, bans: Option<Vec<BanType>>) {
    if let Self::Home { query } | Self::Community { query, .. } = self {
      query.bans = bans;
    }
  }

  pub fn set_search_kind(&mut self, kind: Option<SearchType>) {
    match self {
      Self::Search { query } | Self::PostSearch { query, .. } | Self::CommunitySearch { query, .. } | Self::UserSearch { query, .. } => {
        query.kind = kind
      }
      _ => {}
    }
  }

  pub fn set_search_term(&mut self, term: Option<String>) {
    match self {
      Self::Search { query } | Self::PostSearch { query, .. } | Self::CommunitySearch { query, .. } | Self::UserSearch { query, .. } => {
        query.term = term
      }
      _ => {}
    }
  }

  pub fn with_page(mut self, page: Vec<usize>) -> Self {
    match &mut self {
      Self::Search { query } | Self::PostSearch { query, .. } | Self::CommunitySearch { query, .. } | Self::UserSearch { query, .. } => {
        query.page = Some(page)
      }
      Self::User { query, .. } => query.page = Some(page),
      _ => {}
    }
    self
  }

  pub fn with_listing_page(mut self, page: Vec<(usize, String)>) -> Self {
    if let Self::Home { query } | Self::Community { query, .. } = &mut self {
      query.page = Some(page);
    }
    self
  }

  pub fn clear_page(&mut self) {
    match self {
      Self::Home { query } | Self::Community { query, .. } => query.page = None,
      Self::Search { query } | Self::PostSearch { query, .. } | Self::CommunitySearch { query, .. } | Self::UserSearch { query, .. } => {
        query.page = None
      }
      Self::User { query, .. } => query.page = None,
      _ => {}
    }
  }

  pub fn query_params(&self) -> Vec<(String, String)> {
    let mut params = Vec::new();
    match self {
      Self::Home { query } | Self::Community { query, .. } => {
        if let Some(value) = &query.list {
          append_json_param(&mut params, LIST_QUERY, value);
        }
        if let Some(value) = &query.sort {
          append_json_param(&mut params, SORT_QUERY, value);
        }
        if let Some(value) = &query.page {
          append_json_param(&mut params, PAGE_QUERY, value);
        }
        if let Some(value) = &query.bans {
          append_json_param(&mut params, BANS_QUERY, value);
        }
        if let Some(value) = &query.error {
          params.push((ERROR_QUERY.into(), value.clone()));
        }
      }
      Self::Search { query } | Self::PostSearch { query, .. } | Self::CommunitySearch { query, .. } | Self::UserSearch { query, .. } => {
        if let Some(value) = &query.term {
          params.push((TERM_QUERY.into(), value.clone()));
        }
        if let Some(value) = &query.kind {
          append_json_param(&mut params, KIND_QUERY, value);
        }
        if let Some(value) = &query.sort {
          append_json_param(&mut params, SORT_QUERY, value);
        }
        if let Some(value) = &query.list {
          append_json_param(&mut params, LIST_QUERY, value);
        }
        if let Some(value) = &query.page {
          append_json_param(&mut params, PAGE_QUERY, value);
        }
        if let Some(value) = query.person_id {
          params.push((PERSON_ID_QUERY.into(), value.to_string()));
        }
        if let Some(value) = &query.error {
          params.push((ERROR_QUERY.into(), value.clone()));
        }
      }
      Self::User { query, .. } => {
        if let Some(value) = &query.page {
          append_json_param(&mut params, PAGE_QUERY, value);
        }
        if let Some(value) = query.person_id {
          params.push((PERSON_ID_QUERY.into(), value.to_string()));
        }
        if let Some(value) = &query.error {
          params.push((ERROR_QUERY.into(), value.clone()));
        }
      }
      Self::Post { query, .. } => {
        if let Some(value) = &query.sort {
          append_json_param(&mut params, SORT_QUERY, value);
        }
        if let Some(value) = &query.error {
          params.push((ERROR_QUERY.into(), value.clone()));
        }
      }
      Self::Login { query } => {
        if let Some(value) = &query.error {
          params.push((ERROR_QUERY.into(), value.clone()));
        }
      }
      Self::Other { .. } => {}
    }
    params
  }

  pub fn url(&self) -> String {
    let path = self.path();
    let params = self.query_params();
    if params.is_empty() {
      path
    } else {
      format!("{path}?{}", serde_urlencoded::to_string(params).expect("internal query parameters are serializable"))
    }
  }

  pub fn query_string(&self) -> String {
    serde_urlencoded::to_string(self.query_params()).expect("internal query parameters are serializable")
  }
}

#[derive(Clone, Copy)]
pub enum InternalEndpointRoute {
  Home,
  Post,
  Community,
  User,
  Search,
  PostSearch,
  CommunitySearch,
  UserSearch,
  Login,
  Other,
}

#[derive(Clone)]
struct InternalEndpointContext(Memo<InternalEndpointData>);

pub struct InternalEndpoint<C> {
  endpoint: Memo<InternalEndpointData>,
  marker: PhantomData<fn() -> C>,
}

impl<C> InternalEndpoint<C> {
  pub fn new(_component: C, route: InternalEndpointRoute) -> Self {
    let params = use_params_map();
    let query = use_query_map();
    let pathname = use_location().pathname;
    let endpoint = Memo::new(move |_| {
      let params = params.get();
      let query = query.get();
      let get = |key: &str| query.get(key);
      let error = get(ERROR_QUERY);
      let listing_query = || ListingQuery {
        list: decode_query_value(get(LIST_QUERY)),
        sort: decode_query_value(get(SORT_QUERY)),
        page: decode_query_value(get(PAGE_QUERY)),
        bans: decode_query_value(get(BANS_QUERY)),
        error: error.clone(),
      };
      let search_query = || SearchQuery {
        term: get(TERM_QUERY),
        kind: decode_query_value(get(KIND_QUERY)),
        sort: decode_query_value(get(SORT_QUERY)),
        list: decode_query_value(get(LIST_QUERY)),
        page: decode_query_value(get(PAGE_QUERY)),
        person_id: get(PERSON_ID_QUERY).and_then(|value| value.parse().ok()),
        error: error.clone(),
      };
      match route {
        InternalEndpointRoute::Home => InternalEndpointData::Home { query: listing_query() },
        InternalEndpointRoute::Post => InternalEndpointData::Post {
          id: params.get(ID_PARAM).unwrap_or_default(),
          query: PostQuery { sort: decode_query_value(get(SORT_QUERY)), error },
        },
        InternalEndpointRoute::Community => {
          InternalEndpointData::Community { name: params.get(NAME_PARAM).unwrap_or_default(), query: listing_query() }
        }
        InternalEndpointRoute::User => InternalEndpointData::User {
          name: params.get(NAME_PARAM).unwrap_or_default(),
          query: UserQuery { page: decode_query_value(get(PAGE_QUERY)), person_id: get(PERSON_ID_QUERY).and_then(|value| value.parse().ok()), error },
        },
        InternalEndpointRoute::Search => InternalEndpointData::Search { query: search_query() },
        InternalEndpointRoute::PostSearch => InternalEndpointData::PostSearch { id: params.get(ID_PARAM).unwrap_or_default(), query: search_query() },
        InternalEndpointRoute::CommunitySearch => {
          InternalEndpointData::CommunitySearch { name: params.get(NAME_PARAM).unwrap_or_default(), query: search_query() }
        }
        InternalEndpointRoute::UserSearch => {
          InternalEndpointData::UserSearch { name: params.get(NAME_PARAM).unwrap_or_default(), query: search_query() }
        }
        InternalEndpointRoute::Login => InternalEndpointData::Login { query: LoginQuery { error } },
        InternalEndpointRoute::Other => InternalEndpointData::Other { path: pathname.get() },
      }
    });
    provide_context(InternalEndpointContext(endpoint.clone()));
    Self { endpoint, marker: PhantomData }
  }

  pub fn get(&self) -> InternalEndpointData {
    self.endpoint.get()
  }
}

pub fn use_internal_endpoint() -> Memo<InternalEndpointData> {
  expect_context::<InternalEndpointContext>().0
}

#[derive(Clone, Debug)]
pub struct InternalNavigation {
  pub endpoint: InternalEndpointData,
  pub cache_entry: Option<InternalCacheEntry>,
}

#[derive(Clone, Debug)]
pub enum InternalCacheEntry {
  Posts(usize, GetPostsCacheKey),
  Post(GetCacheKey<GetPost>),
  Comments(GetCacheKey<GetComments>),
  Search(usize, GetCacheKey<Search>),
  User(usize, GetCacheKey<GetPersonDetails>),
}

pub fn internal_posts_cache_entry(
  endpoint: &InternalEndpointData,
  list: ListingType,
  sort: SortType,
  bans: &[BanType],
  auth: Option<String>,
  instance: Option<String>,
) -> Option<InternalCacheEntry> {
  let community_name = match endpoint {
    InternalEndpointData::Home { .. } => None,
    InternalEndpointData::Community { name, .. } => Some(name.clone()),
    _ => return None,
  };
  Some(InternalCacheEntry::Posts(
    0,
    GetPostsCacheKey {
      form: GetPosts {
        type_: Some(list),
        sort: Some(sort),
        page: None,
        limit: Some(50),
        community_id: None,
        community_name,
        saved_only: None,
        liked_only: None,
        disliked_only: None,
        show_hidden: Some(true),
        show_read: Some(true),
        show_nsfw: Some(bans.contains(&BanType::Nsfw)),
        page_cursor: None,
      },
      auth,
      instance,
      ban: bans.contains(&BanType::Ban),
    },
  ))
}

pub fn internal_search_cache_entry(
  endpoint: &InternalEndpointData,
  term: String,
  kind: SearchType,
  sort: SortType,
  list: Option<ListingType>,
  person_id: Option<i32>,
  auth: Option<String>,
  instance: Option<String>,
) -> Option<InternalCacheEntry> {
  let community_name = match endpoint {
    InternalEndpointData::CommunitySearch { name, .. } => Some(name.clone()),
    InternalEndpointData::Search { .. } | InternalEndpointData::PostSearch { .. } | InternalEndpointData::UserSearch { .. } => None,
    _ => return None,
  };
  Some(InternalCacheEntry::Search(
    1,
    GetCacheKey {
      form: Search {
        q: term,
        type_: Some(kind),
        sort: Some(sort),
        community_name: if person_id.is_some() { None } else { community_name },
        community_id: None,
        page: Some(1),
        limit: Some(50),
        creator_id: person_id.map(PersonId),
        listing_type: list,
        post_title_only: None,
      },
      auth,
      instance,
      ban: false,
    },
  ))
}

impl InternalNavigation {
  pub fn new(endpoint: InternalEndpointData) -> Self {
    Self { endpoint, cache_entry: None }
  }

  pub fn with_cache_entry(mut self, cache_entry: InternalCacheEntry) -> Self {
    self.cache_entry = Some(cache_entry);
    self
  }

  pub fn url(&self) -> String {
    self.endpoint.url()
  }
}

pub fn internal_navigation_url(navigation: &InternalNavigation) -> String {
  navigation.url()
}

pub fn navigate_internal(navigation: InternalNavigation, scroll_element: Option<NodeRef<Div>>) -> String {
  navigate_internal_with_options(navigation, scroll_element, NavigateOptions::default(), true)
}

pub fn navigate_internal_pagination(navigation: InternalNavigation) -> String {
  navigate_internal_with_options(navigation, None, NavigateOptions { resolve: true, replace: true, scroll: false, state: State::default() }, false)
}

pub fn navigate_internal_with_options(
  navigation: InternalNavigation,
  scroll_element: Option<NodeRef<Div>>,
  options: NavigateOptions,
  reset_destination_scroll: bool,
) -> String {
  let url = navigation.url();

  #[cfg(not(feature = "ssr"))]
  {
    if let Some(cache_entry) = navigation.cache_entry.as_ref() {
      match cache_entry {
        InternalCacheEntry::Posts(page, key) => {
          expect_context::<RwSignal<BTreeMap<(usize, GetPostsCacheKey), (i64, LemmyAppResult<GetPostsResponse>)>>>().update(|cache| {
            cache.remove(&(*page, key.clone()));
          });
        }
        InternalCacheEntry::Post(key) => {
          expect_context::<RwSignal<BTreeMap<GetCacheKey<GetPost>, (i64, LemmyAppResult<GetPostResponse>)>>>().update(|cache| {
            cache.remove(key);
          });
        }
        InternalCacheEntry::Comments(key) => {
          expect_context::<RwSignal<BTreeMap<GetCacheKey<GetComments>, (i64, LemmyAppResult<GetCommentsResponse>)>>>().update(|cache| {
            cache.remove(key);
          });
        }
        InternalCacheEntry::Search(page, key) => {
          expect_context::<RwSignal<BTreeMap<(usize, GetCacheKey<Search>), (i64, LemmyAppResult<SearchResponse>)>>>().update(|cache| {
            cache.remove(&(*page, key.clone()));
          });
        }
        InternalCacheEntry::User(page, key) => {
          expect_context::<RwSignal<BTreeMap<(usize, GetCacheKey<GetPersonDetails>), (i64, LemmyAppResult<GetPersonDetailsResponse>)>>>().update(
            |cache| {
              cache.remove(&(*page, key.clone()));
            },
          );
        }
      }
    }

    let destination_path = navigation.endpoint.path();
    let destination_query = navigation.endpoint.query_string();
    let current_path = use_location().pathname.get_untracked();
    if current_path == destination_path {
      if let Some(element) = scroll_element.and_then(|node_ref| node_ref.get()) {
        element.set_scroll_left(0);
      }
    }

    let navigate = use_navigate();
    let destination_url = url.clone();
    spawn_local_scoped(async move {
      if reset_destination_scroll && let Err(err) = IndexedDb::save_scroll_position(destination_path, destination_query, 0).await {
        error!("failed to reset destination scroll position: {err}");
      }
      navigate(&destination_url, options);
    });
  }

  url
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub enum BanType {
  Nsfw,
  Ban,
}

pub fn markdown_to_html(content: &str, linkify: bool) -> String {
  let mut options = pulldown_cmark::Options::empty();
  options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
  options.insert(pulldown_cmark::Options::ENABLE_TABLES);
  options.insert(pulldown_cmark::Options::ENABLE_SUPERSCRIPT);
  options.insert(pulldown_cmark::Options::ENABLE_SUBSCRIPT);
  options.insert(pulldown_cmark::Options::ENABLE_CONTAINER_EXTENSIONS);
  if linkify {
    options.insert(pulldown_cmark::Options::ENABLE_LINKIFY_LEMMY);
    options.insert(pulldown_cmark::Options::ENABLE_LINKIFY_HTTP);
  }
  markdown_to_html_with_options(content, options)
}

pub fn markdown_to_html_with_options(content: &str, options: pulldown_cmark::Options) -> String {
  let parser = pulldown_cmark::Parser::new_ext(content, options);
  let custom = parser.map(|event| match event {
    pulldown_cmark::Event::Html(text) => {
      let er = format!("<p>{}</p>", html_escape::encode_safe(&text).to_string());
      pulldown_cmark::Event::Html(er.into())
    }
    pulldown_cmark::Event::InlineHtml(text) => {
      let er = html_escape::encode_safe(&text).to_string();
      pulldown_cmark::Event::InlineHtml(er.into())
    }
    _ => event,
  });
  let mut safe_html = String::new();
  pulldown_cmark::html::push_html(&mut safe_html, custom);
  safe_html
}

pub fn relative_time(published_at_millis: i64) -> String {
  let i18n = use_i18n();
  let now_in_millis = jiff::Timestamp::now().as_millisecond();
  let difference = now_in_millis.saturating_sub(published_at_millis);
  let is_future = difference < 0;
  let elapsed_millis = difference.unsigned_abs();
  let duration = pretty_duration::pretty_duration(
    &std::time::Duration::from_millis(elapsed_millis),
    Some(pretty_duration::PrettyDurationOptions {
      output_format: Some(pretty_duration::PrettyDurationOutputFormat::Compact),
      singular_labels: None,
      plural_labels: None,
    }),
  );
  let abbreviated = if let Some((index, _)) = duration.match_indices(' ').nth(1) { duration[..index].to_owned() } else { duration };
  if is_future { t_string!(i18n, time_future, duration = abbreviated) } else { t_string!(i18n, time_past, duration = abbreviated) }
}

pub fn community_title_with_federation(name: &str, local: bool, host: Option<String>) -> String {
  if local { name.to_owned() } else { format!("{}@{}", name, host.unwrap_or_default()) }
}

pub fn federated_creator_handle(name: &str, domain: Option<&str>, instance: Option<&str>) -> String {
  match domain {
    Some(domain) if instance != Some(domain) => format!("{}@{}", name, domain),
    _ => name.to_owned(),
  }
}

pub fn external_host_attribution(host: Option<String>, instance: Option<String>) -> String {
  let i18n = use_i18n();
  match host {
    Some(host) if Some(host.as_str()) != instance.as_deref() => t_string!(i18n, host_attribution, host = host),
    _ => String::new(),
  }
}

#[derive(Default)]
struct WheelGesture {
  horizontal: bool,
  acc_x: f64,
  acc_y: f64,
  last_ts: f64,
}

pub fn handle_horizontal_scroll_wheel(scroll_element: NodeRef<Div>) -> impl FnMut(WheelEvent) + 'static {
  let mut gesture = WheelGesture::default();
  move |event| {
    let unit = match event.delta_mode() {
      1 => 16.0,
      2 => scroll_element.get_untracked().map(|element| element.client_width() as f64).unwrap_or(800.0),
      _ => 1.0,
    };
    let dx = event.delta_x() * unit;
    let dy = event.delta_y() * unit;
    let now = event.time_stamp();
    if now - gesture.last_ts > 150.0 {
      gesture = WheelGesture::default();
    }
    gesture.last_ts = now;

    if !gesture.horizontal {
      gesture.acc_x = gesture.acc_x + dx.abs();
      gesture.acc_y = gesture.acc_y + if dy.abs() < 20.0 { dy.abs() * 3.0 } else { dy.abs() };
      if gesture.acc_x > 256.0 && gesture.acc_x > gesture.acc_y * 2.0 {
        gesture.horizontal = true;
      }
    }
    if gesture.horizontal || (dy == 0.0 && gesture.acc_y == 0.0) {
      return;
    }
    let Some(element) = scroll_element.get_untracked() else {
      return;
    };

    let scroll_left = element.scroll_left() as f64;
    event.prevent_default();
    element.set_scroll_left((scroll_left + dy) as i32);
  }
}

#[component]
pub fn Warning(
  #[prop(optional, into)] description: Option<String>,
  #[prop(optional, into)] on_retry_click: Option<Callback<MouseEvent>>,
) -> impl IntoView {
  let i18n = use_i18n();
  view! {
    <div class="py-4 px-8 break-inside-avoid">
      <div class="flex justify-between items-center alert alert-warning alert-soft">
        <span>{move || description.clone().unwrap_or_else(|| t_string!(i18n, offline).to_string())}</span>
        {if let Some(o) = on_retry_click {
          view! {
            <span
              on:click={move |e: MouseEvent| {
                o.run(e);
              }}
              class="btn btn-sm"
            >
              {t!(i18n, retry)}
            </span>
          }
            .into_any()
        } else {
          view! {}.into_any()
        }}
      </div>
    </div>
  }
  .into_any()
}

#[component]
pub fn Error(
  #[prop(optional, into)] get_url: Option<bool>,
  #[prop(optional, into)] description: Option<String>,
  error: LemmyAppError,
  #[prop(optional, into)] on_retry_click: Option<Callback<MouseEvent>>,
) -> impl IntoView {
  let i18n = use_i18n();
  let location = use_location();
  let current_url = move || {
    let pathname = location.pathname.get();
    let search = location.search.get();
    if search.is_empty() { pathname } else { format!("{pathname}?{search}") }
  };
  error!("{:#?}", error);
  view! {
    <div class="py-4 px-8 break-inside-avoid">
      <div class="flex alert alert-error alert-soft">
        <details class="w-full min-w-0">
          <summary class="flex justify-between items-center list-none">
            <span>{move || description.clone().unwrap_or_else(|| t_string!(i18n, error).to_string())}</span>
            {if let Some(o) = on_retry_click {
              {
                if let Some(true) = get_url {
                  view! {
                    <a
                      href={current_url}
                      on:click={move |e: MouseEvent| {
                        e.prevent_default();
                        o.run(e);
                      }}
                      class="btn btn-sm"
                    >
                      {t!(i18n, retry)}
                    </a>
                  }
                    .into_any()
                } else {
                  view! {
                    <span
                      on:click={move |e: MouseEvent| {
                        e.prevent_default();
                        o.run(e);
                      }}
                      class="btn btn-sm"
                    >
                      {t!(i18n, retry)}
                    </span>
                  }
                    .into_any()
                }
              }
            } else {
              {
                if let Some(true) = get_url {
                  view! {
                    <a
                      href={current_url}
                      class="btn btn-sm"
                    >
                      {t!(i18n, retry)}
                    </a>
                  }
                    .into_any()
                } else {
                  view! {}.into_any()
                }
              }
            }}
          </summary>
          <div class="overflow-auto">
            <pre>{format!("{:#?}", error.error_type)}</pre>
            <pre>{error.content}</pre>
          </div>
        </details>
      </div>
    </div>
  }
  .into_any()
}

#[component]
pub fn Info(loading: bool, #[prop(optional, into)] description: Option<String>) -> impl IntoView {
  let i18n = use_i18n();
  if loading {
    view! {
      <div class="overflow-hidden break-inside-avoid animate-[popdown_1s_step-end_1]">
        <div class="py-4 px-8">
          <div class="alert alert-info alert-soft">
            <span>{move || description.clone().unwrap_or_else(|| t_string!(i18n, loading).to_string())}</span>
          </div>
        </div>
      </div>
    }
    .into_any()
  } else {
    view! {}.into_any()
  }
}

#[derive(Clone)]
pub struct ReadAuthCookie(pub Signal<Option<String>>);
#[derive(Clone)]
pub struct WriteAuthCookie(pub WriteSignal<Option<String>>);
#[derive(Clone)]
pub struct ReadInstanceCookie(pub Signal<Option<String>>);
#[derive(Clone)]
pub struct WriteInstanceCookie(pub WriteSignal<Option<String>>);
#[derive(Clone)]
pub struct ReadThemeCookie(pub Signal<Option<String>>);
#[derive(Clone)]
pub struct WriteThemeCookie(pub WriteSignal<Option<String>>);

pub fn default_cookie_options<T, E, D>() -> UseCookieOptions<T, E, D> {
  UseCookieOptions::default().max_age(691200000).path("/").same_site(SameSite::Lax)
}

pub fn provide_auth_cookie() -> (Signal<Option<String>>, WriteSignal<Option<String>>) {
  let (get_auth_cookie, set_auth_cookie) = use_cookie_with_options::<String, FromToStringCodec>("jwt", default_cookie_options());
  provide_context(ReadAuthCookie(get_auth_cookie));
  provide_context(WriteAuthCookie(set_auth_cookie));
  (get_auth_cookie, set_auth_cookie)
}

pub fn provide_instance_cookie() -> (Signal<Option<String>>, WriteSignal<Option<String>>) {
  let (get_instance_cookie, set_instance_cookie) = use_cookie_with_options::<String, FromToStringCodec>("instance", default_cookie_options());
  provide_context(ReadInstanceCookie(get_instance_cookie));
  provide_context(WriteInstanceCookie(set_instance_cookie));
  (get_instance_cookie, set_instance_cookie)
}

pub fn provide_theme_cookie() -> (Signal<Option<String>>, WriteSignal<Option<String>>) {
  let (get_theme_cookie, set_theme_cookie) = use_cookie_with_options::<String, FromToStringCodec>("theme", default_cookie_options());
  provide_context(ReadThemeCookie(get_theme_cookie));
  provide_context(WriteThemeCookie(set_theme_cookie));
  (get_theme_cookie, set_theme_cookie)
}

pub fn provide_cookies() {
  provide_auth_cookie();
  provide_instance_cookie();
}

pub fn provide_auth_and_instance_cookies() {
  provide_cookies();
}

#[cfg(feature = "ssr")]
pub fn set_response_cookie(
  response: &leptos_axum::ResponseOptions,
  name: &str,
  value: impl Into<String>,
) -> Result<(), http::header::InvalidHeaderValue> {
  let cookie = cookie::Cookie::build(cookie::Cookie::new(name, value.into()))
    .path("/")
    .same_site(cookie::SameSite::Lax)
    .max_age(cookie::time::SignedDuration::seconds(691200))
    .to_string();
  response.insert_header(http::header::SET_COOKIE, http::HeaderValue::from_str(&cookie)?);
  Ok(())
}
