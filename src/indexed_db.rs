use cfg_if::cfg_if;

cfg_if! {
  if #[cfg(not(feature = "ssr"))] {

  use rexie::{ObjectStore, Result, Rexie, TransactionMode};
  use serde::{Deserialize, Serialize};

  pub async fn build_post_meta_database() -> Result<Rexie> {
    let rexie = Rexie::builder("cache_v2")
      .version(1)
      .add_object_store(ObjectStore::new("post_meta").key_path("post_id"))
      .add_object_store(ObjectStore::new("comment_meta").key_path("comment_id"))
      .build()
      .await?;
    Ok(rexie)
  }

  #[derive(Debug, Serialize, Deserialize)]
  pub struct PostMeta {
    pub post_id: i32,
    pub hidden_comment_ids: Option<Vec<i32>>,
    pub reply_draft: Option<String>,
  }

  #[derive(Debug, Serialize, Deserialize)]
  pub struct CommentMeta {
    pub comment_id: i32,
    pub edit_draft: Option<String>,
    pub reply_draft: Option<String>,
  }

  pub async fn add_comment_array(rexie: &Rexie, post_id: i32, hidden_comment_ids: Vec<i32>) -> Result<i32> {
    let transaction = rexie.transaction(&["post_meta"], TransactionMode::ReadWrite)?;
    let posts = transaction.store("post_meta")?;
    let cr = PostMeta {
      post_id,
      hidden_comment_ids: Some(hidden_comment_ids),
      reply_draft: None,
    };
    let post_meta_value = serde_wasm_bindgen::to_value(&cr).unwrap();
    let post_id = posts.put(&post_meta_value, None).await?;
    transaction.done().await?;
    Ok(serde_wasm_bindgen::from_value(post_id).unwrap())
  }

  pub async fn get_comment_array(rexie: &Rexie, id: i32) -> Result<Vec<i32>> {
    let transaction = rexie.transaction(&["post_meta"], TransactionMode::ReadOnly)?;
    let posts = transaction.store("post_meta")?;
    if let Some(post_meta_value) = posts.get(id.into()).await? {
      if let Ok(PostMeta { hidden_comment_ids: Some(hidden_comment_ids), .. }) = serde_wasm_bindgen::from_value::<PostMeta>(post_meta_value) {
        Ok(hidden_comment_ids)
      } else {
        Ok(vec![])
      }
    } else {
      Ok(vec![])
    }
  }

  pub async fn get_edit_draft(rexie: &Rexie, id: i32) -> Result<String> {
    let transaction = rexie.transaction(&["comment_meta"], TransactionMode::ReadOnly)?;
    let comments = transaction.store("comment_meta")?;
    if let Some(comment_meta_value) = comments.get(id.into()).await? {
      if let Ok(CommentMeta { edit_draft: Some(edit_draft), .. }) = serde_wasm_bindgen::from_value::<CommentMeta>(comment_meta_value) {
        Ok(edit_draft)
      } else {
        Ok("".into())
      }
    } else {
      Ok("".into())
    }
  }

  pub async fn get_reply_draft(rexie: &Rexie, id: i32) -> Result<String> {
    let transaction = rexie.transaction(&["comment_meta"], TransactionMode::ReadOnly)?;
    let comments = transaction.store("comment_meta")?;
    if let Some(comment_meta_value) = comments.get(id.into()).await? {
      if let Ok(CommentMeta { reply_draft: Some(reply_draft), .. }) = serde_wasm_bindgen::from_value::<CommentMeta>(comment_meta_value) {
        Ok(reply_draft)
      } else {
        Ok("".into())
      }
    } else {
      Ok("".into())
    }
  }

// pub async fn get_comments(rexie: &Rexie, post_id: i32) -> Result<Vec<JsValue>> {
//   let transaction = rexie.transaction(&["comment"], TransactionMode::ReadOnly)?;
//   let comments = transaction.store("comment")?;
//   let post_id_value = serde_wasm_bindgen::to_value(&post_id).unwrap();
//   let range = KeyRange::only(&post_id_value)?;
//   let post_id_index = comments.index("post_id")?;
//   let values = post_id_index.get_all(Some(range), None).await?;
//   Ok(values)
// }
  }
}
