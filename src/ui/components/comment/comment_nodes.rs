#[cfg(not(feature = "ssr"))]
use crate::indexed_db::*;
use crate::{errors::LemmyAppError, ui::components::comment::comment_node::CommentNode};
use lemmy_api_common::{lemmy_db_views::structs::CommentView, site::GetSiteResponse};
use leptos::*;

#[component]
pub fn CommentNodes(
  ssr_site: Resource<Option<bool>, Result<GetSiteResponse, LemmyAppError>>,
  comments: MaybeSignal<Vec<CommentView>>,
  _post_id: MaybeSignal<Option<i32>>,
) -> impl IntoView {
  let mut comments_clone = comments.get().clone();
  comments_clone.retain(|ct| ct.comment.path.chars().filter(|c| *c == '.').count() == 1);
  let com_sig = RwSignal::new(comments_clone);

  let now_in_millis = {
    #[cfg(not(feature = "ssr"))]
    {
      chrono::offset::Utc::now().timestamp_millis() as u64
    }
    #[cfg(feature = "ssr")]
    {
      std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64
    }
  };

  let hidden_comments: RwSignal<Vec<i32>> = RwSignal::new(vec![]);

  #[cfg(not(feature = "ssr"))]
  let _hidden_comments_resource = create_local_resource(
    move || (),
    move |()| async move {
      if let Some(p) = _post_id.get() {
        if let Ok(d) = build_comment_database().await {
          if let Ok(Some(CommentRquest { comment_id, .. })) = get_array(&d, p).await {
            hidden_comments.set(comment_id);
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
        if let Some(p) = _post_id.get() {
          if let Ok(d) = build_comment_database().await {
            if let Ok(_) = add_array(&d, p, hidden_comments.get()).await {}
          }
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
        on_toggle={on_hide_show}
        comment_view={cv.into()}
        comments={comments.get().into()}
        level=1
        now_in_millis
      />
    </For>
  }
}
