#[cfg(not(feature = "ssr"))]
use crate::indexed_db::csr_indexed_db::*;
use crate::{errors::AosAppError, ui::components::comment::comment_node::CommentNode};
use lemmy_api_common::{lemmy_db_views::structs::CommentView, site::GetSiteResponse};
use leptos::prelude::*;

#[component]
pub fn CommentNodes(
  ssr_site: Resource<Result<GetSiteResponse, AosAppError>>,
  comments: MaybeSignal<Vec<CommentView>>,
  _post_id: MaybeSignal<Option<i32>>,
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

  // #[cfg(not(feature = "ssr"))]
  let _hidden_comments_resource = Resource::new(
    move || (),
    move |()| async move {
      #[cfg(not(feature = "ssr"))]
      if let Some(p) = _post_id.get() {
        // if let Ok(d) = build_post_meta_database().await {
        //   if let Ok(comment_ids) = get_comment_array(d, p).await {
        //     hidden_comments.set(comment_ids);
        //   }
        // }
      }
    },
  );

  let on_hide_show = move |i: i32| {
    if hidden_comments.get().contains(&i) {
      hidden_comments.update(|hc| hc.retain(|c| i != *c));
    } else {
      hidden_comments.update(|hc| hc.push(i));
    }
    // #[cfg(not(feature = "ssr"))]
    let _add_comment_resource = Resource::new(
      move || (),
      move |()| async move {
        // #[cfg(not(feature = "ssr"))]
        if let Some(p) = _post_id.get() {
          // if let Ok(d) = build_post_meta_database().await {
          //   if let Ok(_) = add_comment_array(d, p, hidden_comments.get()).await {}
          // }
        }
      },
    );
  };

  view! {
    <For each={move || com_sig.get()} key={|cv| cv.comment.id} let:cv>
      <CommentNode
        ssr_site
        parent_comment_id=0
        hidden_comments
        // on_toggle={on_hide_show}
        comment={cv.into()}
        comments={comments.get().into()}
        level=1
        now_in_millis
        highlight_user_id
      />
    </For>
  }
  .into_any()
}
