#[cfg(not(feature = "ssr"))]
pub mod csr_indexed_db {

  use rexie::{ObjectStore, Result, Rexie, TransactionMode};
  use serde::{Deserialize, Serialize};
  use tsify::JsValueSerdeExt;
  use wasm_bindgen::JsValue;

  pub async fn build_indexed_database() -> Result<Rexie> {
    let rexie = Rexie::builder("cache_v3")
      .version(1)
      .add_object_store(ObjectStore::new("post_hidden_comments").key_path("key"))
      .add_object_store(ObjectStore::new("comment_draft"))
      .add_object_store(ObjectStore::new("post_listing_cache"))
      .build()
      .await?;
    Ok(rexie)
  }

  #[derive(Debug, Serialize, Deserialize)]
  pub struct PostHiddenComments {
    pub key: i32,
    pub hidden_comment_ids: Option<Vec<i32>>,
  }

  pub async fn set_hidden_comments(rexie: &Rexie, post_id: i32, hidden_comment_ids: Vec<i32>) -> Result<i32> {
    let transaction = rexie.transaction(&["post_hidden_comments"], TransactionMode::ReadWrite)?;
    let posts = transaction.store("post_hidden_comments")?;
    let cr = PostHiddenComments {
      key: post_id,
      hidden_comment_ids: Some(hidden_comment_ids),
    };
    let post_meta_value = serde_wasm_bindgen::to_value(&cr).unwrap();
    let post_id = posts.put(&post_meta_value, None).await?;
    transaction.done().await?;
    Ok(serde_wasm_bindgen::from_value(post_id).unwrap())
  }

  pub async fn get_hidden_comments(rexie: &Rexie, post_id: i32) -> Result<Vec<i32>> {
    let transaction = rexie.transaction(&["post_hidden_comments"], TransactionMode::ReadOnly)?;
    let posts = transaction.store("post_hidden_comments")?;
    if let Some(post_meta_value) = posts.get(post_id.into()).await? {
      if let Ok(PostHiddenComments {
        hidden_comment_ids: Some(hidden_comment_ids),
        ..
      }) = serde_wasm_bindgen::from_value::<PostHiddenComments>(post_meta_value)
      {
        Ok(hidden_comment_ids)
      } else {
        Ok(vec![])
      }
    } else {
      Ok(vec![])
    }
  }

  #[derive(Debug, Serialize, Deserialize)]
  pub struct CommentDraft {
    // pub key: CommentDraftKey,
    pub value: Option<String>,
  }

  #[derive(Clone, Debug, Serialize, Deserialize)]
  pub enum Draft {
    Edit,
    Reply,
    Post,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct CommentDraftKey {
    pub comment_id: i32,
    pub draft: Draft,
  }

  pub async fn get_draft(rexie: &Rexie, comment_id: i32, draft: Draft) -> Result<String> {
    let transaction = rexie.transaction(&["comment_draft"], TransactionMode::ReadOnly)?;
    let comments = transaction.store("comment_draft")?;
    let key = CommentDraftKey { comment_id, draft };
    if let Some(comment_meta_value) = comments
      .get(serde_wasm_bindgen::to_value(&serde_json::to_string(&key).ok().unwrap()).unwrap())
      .await?
    {
      if let Ok(CommentDraft { value: Some(comment), .. }) = serde_wasm_bindgen::from_value::<CommentDraft>(comment_meta_value) {
        Ok(comment)
      } else {
        Ok("".into())
      }
    } else {
      Ok("".into())
    }
  }

  pub async fn set_draft(rexie: &Rexie, comment_id: i32, draft: Draft, comment: String) -> Result<()> {
    let transaction = rexie.transaction(&["comment_draft"], TransactionMode::ReadWrite)?;
    let posts = transaction.store("comment_draft")?;
    let k = CommentDraftKey { comment_id, draft };
    let cr = CommentDraft {
      // key: k.clone(),
      value: Some(comment),
    };
    let post_meta_value = serde_wasm_bindgen::to_value(&cr).unwrap();
    let post_meta_key = serde_wasm_bindgen::to_value(&serde_json::to_string(&k).ok().unwrap()).unwrap();
    let post_id = posts.put(&post_meta_value, Some(&post_meta_key)).await?;
    transaction.done().await?;
    Ok(())
  }

  pub async fn del_draft(rexie: &Rexie, comment_id: i32, draft: Draft) -> Result<()> {
    let transaction = rexie.transaction(&["comment_draft"], TransactionMode::ReadWrite)?;
    let posts = transaction.store("comment_draft")?;
    let k = CommentDraftKey { comment_id, draft };
    let post_meta_key = serde_wasm_bindgen::to_value(&serde_json::to_string(&k).ok().unwrap()).unwrap();
    let post_id = posts.delete(post_meta_key).await?;
    transaction.done().await?;
    Ok(())
  }

  pub async fn get_post_listings<Form>(rexie: &Rexie, key: &Form) -> Result<String>
  where
    Form: serde::Serialize,
  {
    let transaction = rexie.transaction(&["post_listing_cache"], TransactionMode::ReadOnly)?;
    let comments = transaction.store("post_listing_cache")?;
    if let Some(comment_meta_value) = comments
      .get(serde_wasm_bindgen::to_value(&serde_json::to_string(key).ok().unwrap()).unwrap())
      .await?
    {
      if let Ok(CommentDraft { value: Some(comment), .. }) = serde_wasm_bindgen::from_value::<CommentDraft>(comment_meta_value) {
        Ok(comment)
      } else {
        Ok("".into())
      }
    } else {
      Ok("".into())
    }
  }

  pub async fn set_post_listings<Form>(rexie: &Rexie, key: &Form, t: &String) -> Result<()>
  where
    Form: serde::Serialize,
  {
    let transaction = rexie.transaction(&["post_listing_cache"], TransactionMode::ReadWrite)?;
    let posts = transaction.store("post_listing_cache")?;
    let post_meta_value = serde_wasm_bindgen::to_value(t).unwrap();
    let post_meta_key = serde_wasm_bindgen::to_value(&serde_json::to_string(key).ok().unwrap()).unwrap();
    let post_id = posts.put(&post_meta_value, Some(&post_meta_key)).await?;
    transaction.done().await?;
    Ok(())
  }

  pub async fn del_post_listings<Form>(rexie: &Rexie, key: &Form) -> Result<()>
  where
    Form: serde::Serialize,
  {
    let transaction = rexie.transaction(&["post_listing_cache"], TransactionMode::ReadWrite)?;
    let posts = transaction.store("post_listing_cache")?;
    let post_meta_key = serde_wasm_bindgen::to_value(&serde_json::to_string(key).ok().unwrap()).unwrap();
    let post_id = posts.delete(post_meta_key).await?;
    transaction.done().await?;
    Ok(())
  }
}
