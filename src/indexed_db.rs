use cfg_if::cfg_if;

cfg_if! {
  if #[cfg(not(feature = "ssr"))] {
    use rexie::{/*KeyRange, */ ObjectStore, Result, Rexie, TransactionMode};
    use serde::{Deserialize, Serialize};
    // use web_sys::wasm_bindgen::JsValue;

    pub async fn build_comment_database() -> Result<Rexie> {
      let rexie = Rexie::builder("cache")
        .version(1)
        .add_object_store(ObjectStore::new("comment").key_path("post_id"))
        .build()
        .await?;
      Ok(rexie)
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CommentRquest {
      pub post_id: i32,
      pub comment_id: Vec<i32>,
    }

    pub async fn add_array(rexie: &Rexie, post_id: i32, comment_id: Vec<i32>) -> Result<i32> {
      let transaction = rexie.transaction(&["comment"], TransactionMode::ReadWrite)?;
      let comments = transaction.store("comment")?;
      let cr = CommentRquest { post_id, comment_id };
      let comment_meta_value = serde_wasm_bindgen::to_value(&cr).unwrap();
      let comment_id = comments.put(&comment_meta_value, None).await?;
      transaction.done().await?;
      Ok(serde_wasm_bindgen::from_value(comment_id).unwrap())
    }

    pub async fn get_array(rexie: &Rexie, id: i32) -> Result<Option<CommentRquest>> {
      let transaction = rexie.transaction(&["comment"], TransactionMode::ReadOnly)?;
      let employees = transaction.store("comment")?;
      if let Some(employee) = employees.get(id.into()).await? {
        let employee = serde_wasm_bindgen::from_value(employee).unwrap();
        Ok(employee)
      } else {
        Ok(Some(CommentRquest {
          post_id: id,
          comment_id: vec![],
        }))
      }
    }
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
