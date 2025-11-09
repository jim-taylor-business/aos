pub mod csr_indexed_db {
  use lemmy_api_common::{
    comment::*,
    community::*,
    person::*,
    post::*,
    private_message::GetPrivateMessages,
    site::*,
    LemmyErrorType,
    SuccessResponse,
  };
  use leptos::logging::log;
  use serde::{de::DeserializeOwned, Deserialize, Serialize};
  use strum_macros::Display;
  use thiserror::Error;
  use tsify::JsValueSerdeExt;

  #[derive(Debug, Serialize, Deserialize)]
  pub struct CommentDraft {
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

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ScrollPositionKey {
    pub path: String,
    pub query: String,
  }

  impl Store for ScrollPositionKey {
    fn store_name(&self) -> &'static str {
      "scroll_positions"
    }
  }

  impl Store for i32 {
    fn store_name(&self) -> &'static str {
      "post_closed_comments"
    }
  }

  impl Store for CommentDraftKey {
    fn store_name(&self) -> &'static str {
      "comment_drafts"
    }
  }

  pub trait Store {
    fn store_name(&self) -> &'static str;
  }

  impl Store for Login {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for () {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for ListCommunities {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for CreatePostReport {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for SavePost {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for BlockPerson {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for CreatePostLike {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for CreateCommentLike {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for SaveComment {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for GetReplies {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for GetPersonMentions {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for GetPrivateMessages {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for MarkCommentReplyAsRead {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for CreateComment {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for EditComment {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for Search {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for GetComment {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for GetModlog {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for GetPost {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for GetPosts {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for GetComments {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  #[cfg(not(feature = "ssr"))]
  use rexie::{ObjectStore, Rexie, Transaction, TransactionMode};
  #[cfg(not(feature = "ssr"))]
  use wasm_bindgen::JsValue;

  #[cfg(not(feature = "ssr"))]
  #[derive(Debug, Error)]
  pub enum Error {
    #[error("rexie error: {0}")]
    Rexie(#[from] rexie::Error),
    #[error("serde wasm bindgen error: {0}")]
    SerdeWasmBindgen(#[from] serde_wasm_bindgen::Error),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
  }

  #[cfg(not(feature = "ssr"))]
  #[derive(Clone)]
  pub struct IndexedDb {
    pub rexie: Rexie,
  }

  #[cfg(not(feature = "ssr"))]
  impl IndexedDb {
    pub async fn new() -> Result<Self, Error> {
      let rexie = Rexie::builder("cache_v5")
        .version(1)
        .add_object_store(ObjectStore::new("post_closed_comments"))
        .add_object_store(ObjectStore::new("comment_drafts"))
        .add_object_store(ObjectStore::new("query_gets"))
        .add_object_store(ObjectStore::new("scroll_positions"))
        .build()
        .await?;
      Ok(Self { rexie })
    }
  }

  #[cfg(not(feature = "ssr"))]
  impl IndexedDb {
    pub async fn get<Form, Response>(&self, key: &Form) -> Result<Option<Response>, Error>
    where
      Form: Serialize + Store,
      Response: DeserializeOwned,
    {
      let transaction = self.rexie.transaction(&[key.store_name()], TransactionMode::ReadOnly)?;
      let comments = transaction.store(key.store_name())?;
      if let Some(comment_meta_value) = comments.get(serde_wasm_bindgen::to_value(&serde_json::to_string(key)?)?).await? {
        Ok(Some(serde_wasm_bindgen::from_value::<Response>(comment_meta_value)?))
      } else {
        Ok(None)
      }
    }

    pub async fn set<Form, Response>(&self, key: &Form, t: &Response) -> Result<(), Error>
    where
      Form: Serialize + Store,
      Response: Serialize,
    {
      let transaction = self.rexie.transaction(&[key.store_name()], TransactionMode::ReadWrite)?;
      let posts = transaction.store(key.store_name())?;
      let post_meta_value = serde_wasm_bindgen::to_value(t)?;
      let post_meta_key = serde_wasm_bindgen::to_value(&serde_json::to_string(key)?)?;
      let post_id = posts.put(&post_meta_value, Some(&post_meta_key)).await?;
      transaction.done().await?;
      Ok(())
    }

    pub async fn del<Form>(&self, key: &Form) -> Result<(), Error>
    where
      Form: Serialize + Store,
    {
      let transaction = self.rexie.transaction(&[key.store_name()], TransactionMode::ReadWrite)?;
      let posts = transaction.store(key.store_name())?;
      let post_meta_key = serde_wasm_bindgen::to_value(&serde_json::to_string(key)?)?;
      let post_id = posts.delete(post_meta_key).await?;
      transaction.done().await?;
      Ok(())
    }
  }
}
