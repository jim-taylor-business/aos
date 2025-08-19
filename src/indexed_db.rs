#[cfg(not(feature = "ssr"))]
pub mod csr_indexed_db {

  use leptos::Serializable;
  use rexie::{ObjectStore, Rexie, TransactionMode};
  use serde::{Deserialize, Serialize};
  use thiserror::Error;
  use tsify::JsValueSerdeExt;
  use wasm_bindgen::JsValue;

  #[derive(Debug, Error)]
  pub enum Error {
    #[error("rexie error: {0}")]
    Rexie(#[from] rexie::Error),
    #[error("serde wasm bindgen error: {0}")]
    SerdeWasmBindgen(#[from] serde_wasm_bindgen::Error),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
  }

  pub async fn build_indexed_database() -> Result<Rexie, Error> {
    let rexie = Rexie::builder("cache_v4")
      .version(1)
      .add_object_store(ObjectStore::new("post_closed_comments"))
      .add_object_store(ObjectStore::new("comment_draft"))
      .add_object_store(ObjectStore::new("query_get_cache"))
      .build()
      .await?;
    Ok(rexie)
  }

  pub async fn set_hidden_comments(rexie: &Rexie, post_id: i32, hidden_comment_ids: Vec<i32>) -> Result<i32, Error> {
    let transaction = rexie.transaction(&["post_closed_comments"], TransactionMode::ReadWrite)?;
    let posts = transaction.store("post_closed_comments")?;
    let post_meta_value = serde_wasm_bindgen::to_value(&hidden_comment_ids)?;
    let post_id = posts.put(&post_meta_value, Some(&serde_wasm_bindgen::to_value(&post_id)?)).await?;
    transaction.done().await?;
    Ok(serde_wasm_bindgen::from_value(post_id)?)
  }

  pub async fn get_hidden_comments(rexie: &Rexie, post_id: i32) -> Result<Vec<i32>, Error> {
    let transaction = rexie.transaction(&["post_closed_comments"], TransactionMode::ReadOnly)?;
    let posts = transaction.store("post_closed_comments")?;
    if let Some(post_meta_value) = posts.get(post_id.into()).await? {
      leptos::logging::log!("pood {:#?} {:#?}", post_id, post_meta_value);
      Ok(serde_wasm_bindgen::from_value::<Vec<i32>>(post_meta_value)?)
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

  pub async fn get_draft(rexie: &Rexie, comment_id: i32, draft: Draft) -> Result<Option<String>, Error> {
    let transaction = rexie.transaction(&["comment_draft"], TransactionMode::ReadOnly)?;
    let comments = transaction.store("comment_draft")?;
    let key = CommentDraftKey { comment_id, draft };
    if let Some(comment_meta_value) = comments.get(serde_wasm_bindgen::to_value(&serde_json::to_string(&key)?)?).await? {
      if let CommentDraft { value: Some(comment), .. } = serde_wasm_bindgen::from_value::<CommentDraft>(comment_meta_value)? {
        return Ok(Some(comment));
      }
    }
    Ok(None)
  }

  pub async fn set_draft(rexie: &Rexie, comment_id: i32, draft: Draft, comment: String) -> Result<(), Error> {
    let transaction = rexie.transaction(&["comment_draft"], TransactionMode::ReadWrite)?;
    let posts = transaction.store("comment_draft")?;
    let k = CommentDraftKey { comment_id, draft };
    let cr = CommentDraft {
      // key: k.clone(),
      value: Some(comment),
    };
    let post_meta_value = serde_wasm_bindgen::to_value(&cr)?;
    let post_meta_key = serde_wasm_bindgen::to_value(&serde_json::to_string(&k)?)?;
    let post_id = posts.put(&post_meta_value, Some(&post_meta_key)).await?;
    transaction.done().await?;
    Ok(())
  }

  pub async fn del_draft(rexie: &Rexie, comment_id: i32, draft: Draft) -> Result<(), Error> {
    let transaction = rexie.transaction(&["comment_draft"], TransactionMode::ReadWrite)?;
    let posts = transaction.store("comment_draft")?;
    let k = CommentDraftKey { comment_id, draft };
    let post_meta_key = serde_wasm_bindgen::to_value(&serde_json::to_string(&k)?)?;
    let post_id = posts.delete(post_meta_key).await?;
    transaction.done().await?;
    Ok(())
  }

  pub async fn get_query_get_cache<Form, Response>(rexie: &Rexie, key: &Form) -> Result<Option<Response>, Error>
  where
    Form: Serialize,
    Response: Serializable + Serialize + for<'de> Deserialize<'de>,
  {
    let transaction = rexie.transaction(&["query_get_cache"], TransactionMode::ReadOnly)?;
    let comments = transaction.store("query_get_cache")?;
    if let Some(comment_meta_value) = comments.get(serde_wasm_bindgen::to_value(&serde_json::to_string(key)?)?).await? {
      Ok(Some(serde_wasm_bindgen::from_value::<Response>(comment_meta_value)?))
    } else {
      Ok(None)
    }
  }

  pub async fn set_query_get_cache<Form, Response>(rexie: &Rexie, key: &Form, t: &Response) -> Result<(), Error>
  where
    Form: Serialize,
    Response: Serializable + Serialize + for<'de> Deserialize<'de>,
  {
    let transaction = rexie.transaction(&["query_get_cache"], TransactionMode::ReadWrite)?;
    let posts = transaction.store("query_get_cache")?;
    // let post_meta_value = serde_wasm_bindgen::to_value(&cr).unwrap();
    let post_meta_value = serde_wasm_bindgen::to_value(t)?;
    let post_meta_key = serde_wasm_bindgen::to_value(&serde_json::to_string(key)?)?;
    let post_id = posts.put(&post_meta_value, Some(&post_meta_key)).await?;
    transaction.done().await?;
    Ok(())
  }

  pub async fn del_query_get_cache<Form>(rexie: &Rexie, key: &Form) -> Result<(), Error>
  where
    Form: Serialize,
  {
    let transaction = rexie.transaction(&["query_get_cache"], TransactionMode::ReadWrite)?;
    let posts = transaction.store("query_get_cache")?;
    let post_meta_key = serde_wasm_bindgen::to_value(&serde_json::to_string(key)?)?;
    let post_id = posts.delete(post_meta_key).await?;
    transaction.done().await?;
    Ok(())
  }
}
