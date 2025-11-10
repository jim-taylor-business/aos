use crate::{comment::Comment, db::csr_indexed_db::*};
use lemmy_api_common::lemmy_db_views::structs::CommentView;
use leptos::{prelude::*, task::spawn_local_scoped_with_cancellation};

#[component]
pub fn Comments(comments: Signal<Vec<CommentView>>, post_id: Signal<Option<i32>>) -> impl IntoView {
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
  spawn_local_scoped_with_cancellation(async move {
    if let Some(p) = post_id.get() {
      if let Ok(d) = IndexedDb::new().await {
        if let Ok(Some(comment_ids)) = d.get::<i32, Vec<i32>>(&p).await {
          hidden_comments.set(comment_ids);
        }
      }
    }
  });

  view! {
    <For each={move || com_sig.get()} key={|cv| cv.comment.id} let:cv>
      <Comment
        parent_comment_id=0
        hidden_comments
        comment={cv.into()}
        comments={comments.get().into()}
        level=1
        now_in_millis
        highlight_user_id
        post_id
      />
    </For>
  }
}
