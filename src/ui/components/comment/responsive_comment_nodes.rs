#[cfg(not(feature = "ssr"))]
use crate::indexed_db::csr_indexed_db::*;
use crate::{
  errors::LemmyAppError,
  ui::components::comment::{comment_node::CommentNode, responsive_comment_node::ResponsiveCommentNode},
};
use lemmy_api_common::{lemmy_db_views::structs::CommentView, site::GetSiteResponse};
use leptos::*;

#[component]
pub fn ResponsiveCommentNodes(
  ssr_site: Resource<Option<bool>, Result<GetSiteResponse, LemmyAppError>>,
  comments: MaybeSignal<Vec<CommentView>>,
  post_id: Signal<Option<i32>>,
) -> impl IntoView {
  let mut comments_clone = comments.get().clone();
  comments_clone.retain(|ct| ct.comment.path.chars().filter(|c| *c == '.').count() == 1);
  let com_sig = RwSignal::new(comments_clone);
  let highlight_user_id = RwSignal::new(None);

  let now_in_millis = RwSignal::new({
    #[cfg(not(feature = "ssr"))]
    {
      chrono::offset::Utc::now().timestamp_millis() as u64
    }
    #[cfg(feature = "ssr")]
    {
      std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64
    }
  });

  let hidden_comments: RwSignal<Vec<i32>> = RwSignal::new(vec![]);

  #[cfg(not(feature = "ssr"))]
  let _hidden_comments_resource = create_local_resource(
    move || (),
    move |()| async move {
      if let Some(p) = post_id.get() {
        if let Ok(d) = build_indexed_database().await {
          if let Ok(comment_ids) = get_hidden_comments(&d, p).await {
            logging::log!("dood {:#?}", comment_ids);
            hidden_comments.set(comment_ids);
          }
        }
      }
    },
  );

  let on_hide_show = move |i: i32| {
    if hidden_comments.get().contains(&i) {
      hidden_comments.update(|hc| hc.retain(|c| i != *c));
    } else {
      hidden_comments.update(|hc| hc.push(i));
    }
    let _hidden_comments_resource = create_local_resource(
      move || (),
      move |()| async move {
        #[cfg(not(feature = "ssr"))]
        if let Some(p) = post_id.get() {
          if let Ok(d) = build_indexed_database().await {
            if let Ok(_) = set_hidden_comments(&d, p, hidden_comments.get()).await {}
          }
        }
      },
    );
  };

  view! {
    <For each={move || com_sig.get()} key={|cv| cv.comment.id} let:cv>
      <ResponsiveCommentNode
        ssr_site
        parent_comment_id=0
        hidden_comments
        on_toggle={on_hide_show}
        comment={cv.into()}
        comments={comments.get().into()}
        level=1
        now_in_millis
        highlight_user_id
      />
    </For>
  }
}
