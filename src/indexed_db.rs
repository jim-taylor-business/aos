use lemmy_api_common::post::{GetPost, GetPostResponse};

#[cfg(not(feature = "ssr"))]
pub mod csr_indexed_db {

  use lemmy_api_common::post::{GetPost, GetPostResponse, GetPosts, GetPostsResponse};
  use leptos::{logging::log, Serializable};
  use rexie::{ObjectStore, Rexie, Transaction, TransactionMode};
  use serde::{Deserialize, Serialize};
  use strum_macros::Display;
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
    let rexie = Rexie::builder("cache_v5")
      .version(1)
      .add_object_store(ObjectStore::new("post_closed_comments"))
      .add_object_store(ObjectStore::new("comment_drafts"))
      .add_object_store(ObjectStore::new("query_gets"))
      .add_object_store(ObjectStore::new("scroll_positions"))
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
      Ok(serde_wasm_bindgen::from_value::<Vec<i32>>(post_meta_value)?)
    } else {
      Ok(vec![])
    }
  }

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

  pub async fn get_draft(rexie: &Rexie, comment_id: i32, draft: Draft) -> Result<Option<String>, Error> {
    let transaction = rexie.transaction(&["comment_drafts"], TransactionMode::ReadOnly)?;
    let comments = transaction.store("comment_drafts")?;
    let key = CommentDraftKey { comment_id, draft };
    if let Some(comment_meta_value) = comments.get(serde_wasm_bindgen::to_value(&serde_json::to_string(&key)?)?).await? {
      if let CommentDraft { value: Some(comment), .. } = serde_wasm_bindgen::from_value::<CommentDraft>(comment_meta_value)? {
        return Ok(Some(comment));
      }
    }
    Ok(None)
  }

  pub async fn set_draft(rexie: &Rexie, comment_id: i32, draft: Draft, comment: String) -> Result<(), Error> {
    let transaction = rexie.transaction(&["comment_drafts"], TransactionMode::ReadWrite)?;
    let posts = transaction.store("comment_drafts")?;
    let k = CommentDraftKey { comment_id, draft };
    let cr = CommentDraft {
      value: Some(comment),
    };
    let post_meta_value = serde_wasm_bindgen::to_value(&cr)?;
    let post_meta_key = serde_wasm_bindgen::to_value(&serde_json::to_string(&k)?)?;
    let post_id = posts.put(&post_meta_value, Some(&post_meta_key)).await?;
    transaction.done().await?;
    Ok(())
  }

  pub async fn del_draft(rexie: &Rexie, comment_id: i32, draft: Draft) -> Result<(), Error> {
    let transaction = rexie.transaction(&["comment_drafts"], TransactionMode::ReadWrite)?;
    let posts = transaction.store("comment_drafts")?;
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
    let transaction = rexie.transaction(&["query_gets"], TransactionMode::ReadOnly)?;
    let comments = transaction.store("query_gets")?;
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
    let transaction = rexie.transaction(&["query_gets"], TransactionMode::ReadWrite)?;
    let posts = transaction.store("query_gets")?;
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
    let transaction = rexie.transaction(&["query_gets"], TransactionMode::ReadWrite)?;
    let posts = transaction.store("query_gets")?;
    let post_meta_key = serde_wasm_bindgen::to_value(&serde_json::to_string(key)?)?;
    let post_id = posts.delete(post_meta_key).await?;
    transaction.done().await?;
    Ok(())
  }

  // pub async fn get_scroll_position_cache<Form, Response>(rexie: &Rexie, key: &Form) -> Result<Option<Response>, Error>
  // where
  //   Form: Serialize,
  //   Response: Serializable + Serialize + for<'de> Deserialize<'de>,
  // {
  //   let transaction = rexie.transaction(&["scroll_positions"], TransactionMode::ReadOnly)?;
  //   let comments = transaction.store("scroll_positions")?;
  //   if let Some(comment_meta_value) = comments.get(serde_wasm_bindgen::to_value(&serde_json::to_string(key)?)?).await? {
  //     Ok(Some(serde_wasm_bindgen::from_value::<Response>(comment_meta_value)?))
  //   } else {
  //     Ok(None)
  //   }
  // }

  // pub async fn set_scroll_position_cache<Form, Response>(rexie: &Rexie, key: &Form, t: &Response) -> Result<(), Error>
  // where
  //   Form: Serialize,
  //   Response: Serializable + Serialize + for<'de> Deserialize<'de>,
  // {
  //   let transaction = rexie.transaction(&["scroll_positions"], TransactionMode::ReadWrite)?;
  //   let posts = transaction.store("scroll_positions")?;
  //   let post_meta_value = serde_wasm_bindgen::to_value(t)?;
  //   let post_meta_key = serde_wasm_bindgen::to_value(&serde_json::to_string(key)?)?;
  //   let post_id = posts.put(&post_meta_value, Some(&post_meta_key)).await?;
  //   transaction.done().await?;
  //   Ok(())
  // }

  // pub async fn del_scroll_position_cache<Form>(rexie: &Rexie, key: &Form) -> Result<(), Error>
  // where
  //   Form: Serialize,
  // {
  //   let transaction = rexie.transaction(&["scroll_positions"], TransactionMode::ReadWrite)?;
  //   let posts = transaction.store("scroll_positions")?;
  //   let post_meta_key = serde_wasm_bindgen::to_value(&serde_json::to_string(key)?)?;
  //   let post_id = posts.delete(post_meta_key).await?;
  //   transaction.done().await?;
  //   Ok(())
  // }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ScrollPositionKey {
    pub path: String,
    pub query: String,
  }


  // #[derive(Debug)]
  // pub struct IndexedDbApi {
  //   rexie: Rexie,
  // }

  // impl IndexedDbApi {
  //     pub async fn new(db_name: &str, version: u32) -> Result<Self, Error> {
  //       let rexie = Rexie::builder(db_name)
  //         .version(version)
  //         .add_object_store(ObjectStore::new("post_closed_comments"))
  //         .add_object_store(ObjectStore::new("comment_drafts"))
  //         .add_object_store(ObjectStore::new("query_gets"))
  //         .add_object_store(ObjectStore::new("scroll_positions"))
  //         .build()
  //         .await?;
  //       // self.rexie = rexie;
  //       Ok(Self { rexie })
  //       // Ok(())
  //     }

  //     fn table_name<T: Serialize>(&self) -> String {
  //       // Use type name as table name
  //       log!("{}",std::any::type_name::<T>().to_string().as_str());
  //       match std::any::type_name::<T>().to_string().as_str() {
  //         "GetPost" => {
  //           "query_gets".to_string()
  //         }
  //         "aos::indexed_db::csr_indexed_db::ScrollPositionKey" => {
  //           "scroll_positions".to_string()
  //         }
  //         _ => {
  //           panic!("no storage");
  //         }
  //       }
  //     }

  //     pub async fn set<Form, Response>(&self, key: &Form, value: &Response) -> Result<(), Error>
  //     where
  //       Form: Serialize,
  //       Response: Serialize + for<'de> Deserialize<'de>,
  //     {
  //       let table = self.table_name::<Form>();
  //       let transaction = self.rexie.transaction(&[&table[..]], TransactionMode::ReadWrite)?;
  //       let table_store = transaction.store(&table)?;
  //       let value_store = serde_wasm_bindgen::to_value(value)?;
  //       let key_store = serde_wasm_bindgen::to_value(&serde_json::to_string(key)?)?;
  //       let store_result = table_store.put(&value_store, Some(&key_store)).await?;
  //       transaction.done().await?;
  //       Ok(())
  //     }

  //     pub async fn get<Form, Response>(&self, key: &Form) -> Result<Option<Response>, Error>
  //     where
  //       Form: Serialize,
  //       Response: Serialize + for<'de> Deserialize<'de>,
  //     {
  //       let table = self.table_name::<Form>();
  //       let transaction = self.rexie.transaction(&[&table[..]], TransactionMode::ReadOnly)?;
  //       let table_store = transaction.store(&table)?;
  //       if let Some(value_store) = table_store.get(serde_wasm_bindgen::to_value(&serde_json::to_string(key)?)?).await? {
  //         Ok(Some(serde_wasm_bindgen::from_value::<Response>(value_store)?))
  //       } else {
  //         Ok(None)
  //       }
  //     }

  //     pub async fn del<Form, Response>(&self, key: &Form) -> Result<(), Error>
  //     where
  //       Form: Serialize,
  //       // Response: Serialize + for<'de> Deserialize<'de>,
  //     {
  //       let table = self.table_name::<Form>();
  //       let transaction = self.rexie.transaction(&[&table[..]], TransactionMode::ReadOnly)?;
  //       let table_store = transaction.store(&table)?;
  //       let store_result = table_store.delete(serde_wasm_bindgen::to_value(&serde_json::to_string(key)?)?).await?;
  //       transaction.done().await?;
  //       Ok(())
  //     }
  // }


        // #[derive(Clone)]
        // pub struct IndexedDb {
        //   pub rexie: Rexie,
        // }

        // impl IndexedDb  {
        //   pub async fn new() -> Result<Self, Error> {
        //     let rexie = Rexie::builder("cache_v5")
        //       .version(1)
        //       .add_object_store(ObjectStore::new("post_closed_comments"))
        //       .add_object_store(ObjectStore::new("comment_drafts"))
        //       .add_object_store(ObjectStore::new("query_gets"))
        //       .add_object_store(ObjectStore::new("scroll_positions"))
        //       .build()
        //       .await?;
        //     Ok(Self { rexie })
        //     // Self { rexie: None }
        //   }

        //   // pub fn build_indexed_database() -> leptos::Resource<(), ()> {
        //   // // pub fn build_indexed_database() -> leptos::Resource<(), Self> {
        //   //   leptos::create_local_resource(
        //   //     move || (),
        //   //     move |()| async move {
        //   //       log!("wd");
        //   //       // let r = Rexie::builder("cache_v5")
        //   //       //   .version(1)
        //   //       //   .add_object_store(ObjectStore::new("post_closed_comments"))
        //   //       //   .add_object_store(ObjectStore::new("comment_drafts"))
        //   //       //   .add_object_store(ObjectStore::new("query_gets"))
        //   //       //   .add_object_store(ObjectStore::new("scroll_positions"))
        //   //       //   .build()
        //   //       //   .await.ok().unwrap();
        //   //       // Self { rexie: r }
        //   //     },
        //   //   )
        //   // }

        //   // pub async fn build(&mut self) -> Result<(), Error> {
        //   //   if let Some(ref r) = self.rexie {
        //   //     Ok(())
        //   //   } else {
        //   //     log!("wd");
        //   //     self.rexie = Some(Rexie::builder("cache_v5")
        //   //       .version(1)
        //   //       .add_object_store(ObjectStore::new("post_closed_comments"))
        //   //       .add_object_store(ObjectStore::new("comment_drafts"))
        //   //       .add_object_store(ObjectStore::new("query_gets"))
        //   //       .add_object_store(ObjectStore::new("scroll_positions"))
        //   //       .build()
        //   //       .await?);
        //   //     Ok(())
        //   //   }
        //   // }


        //   // pub fn get_rexie() -> &Rexie {
        //   //   &rexie
        //   // }
        // }

        // trait Store
        // {
        //   fn store_name(&self) -> &'static str;
        //   // fn get_rexie(&self) -> &Rexie;
        // }

        // impl Store for GetPost {
        //   fn store_name(&self) -> &'static str {
        //     "query_gets"
        //   }
        // }

        // impl Store for ScrollPositionKey {
        //   fn store_name(&self) -> &'static str {
        //     "scroll_positions"
        //   }
        // }

        // // // fn store_name<T: Store>(v: &T) -> &'static str {
        // // //   v.store_name()
        // // // }

        // pub trait Crud<Form, Response>
        // where
        //   Form: Serialize + Store,
        //   Response: Serialize + for<'de> Deserialize<'de>,
        // {
        //   async fn get(&self, key: &Form) -> Result<Option<Response>, Error>;
        //   async fn set(&self, key: &Form, t: &Response) -> Result<(), Error>;
        //   async fn del(&self, key: &Form) -> Result<(), Error>;
        // }

        // impl<Form, Response> Crud<Form, Response> for IndexedDb
        // where
        //   Form: Serialize + Store,
        //   Response: Serialize + for<'de> Deserialize<'de>,
        // {
        //   async fn get(&self, key: &Form) -> Result<Option<Response>, Error>
        //   {
        //     // let r = self.build_indexed_database().await?;

        //     let transaction = self.rexie.transaction(&[key.store_name()], TransactionMode::ReadOnly)?;
        //     let comments = transaction.store(key.store_name())?;
        //     if let Some(comment_meta_value) = comments.get(serde_wasm_bindgen::to_value(&serde_json::to_string(key)?)?).await? {
        //       Ok(Some(serde_wasm_bindgen::from_value::<Response>(comment_meta_value)?))
        //     } else {
        //       Ok(None)
        //     }
        //   }

        //   async fn set(&self, key: &Form, t: &Response) -> Result<(), Error>
        //   {
        //     // let r = self.build_indexed_database().await?;

        //     let transaction = self.rexie.transaction(&[key.store_name()], TransactionMode::ReadWrite)?;
        //     let posts = transaction.store(key.store_name())?;
        //     let post_meta_value = serde_wasm_bindgen::to_value(t)?;
        //     let post_meta_key = serde_wasm_bindgen::to_value(&serde_json::to_string(key)?)?;
        //     // let post_meta_key = serde_wasm_bindgen::to_value(key)?;
        //     let post_id = posts.put(&post_meta_value, Some(&post_meta_key)).await?;
        //     transaction.done().await?;
        //     Ok(())
        //   }

        //   async fn del(&self, key: &Form) -> Result<(), Error>
        //   {
        //     // let r = self.build_indexed_database().await?;

        //     // leptos::spawn_local(async move {
        //     let transaction = self.rexie.transaction(&[key.store_name()], TransactionMode::ReadWrite)?;
        //     let posts = transaction.store(key.store_name())?;
        //     let post_meta_key = serde_wasm_bindgen::to_value(&serde_json::to_string(key)?)?;
        //     let post_id = posts.delete(post_meta_key).await?;
        //     transaction.done().await?;
        //     // });
        //     Ok(())
        //   }
        // }
        //

  #[derive(Clone)]
  pub struct IndexedDb {
    pub rexie: Rexie,
  }

  impl IndexedDb  {
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

  trait Store
  {
    fn store_name(&self) -> &'static str;
  }

  impl Store for GetPost {
    fn store_name(&self) -> &'static str {
      "query_gets"
    }
  }

  impl Store for ScrollPositionKey {
    fn store_name(&self) -> &'static str {
      "scroll_positions"
    }
  }

  // pub trait Crud
  // // where
  // //   Form: Serialize + Store,
  // //   Response: Serialize + for<'de> Deserialize<'de>,
  // {
  //   async fn get<Form, Response>(&self, key: &Form) -> Result<Option<Response>, Error>;
  //   async fn set<Form, Response>(&self, key: &Form, t: &Response) -> Result<(), Error>;
  //   async fn del<Form>(&self, key: &Form) -> Result<(), Error>;
  // }

  impl IndexedDb
  {
    pub async fn get<Form, Response>(&self, key: &Form) -> Result<Option<Response>, Error>
    where
      Form: Serialize + Store,
      Response: Serialize + for<'de> Deserialize<'de>,
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
      Response: Serialize + for<'de> Deserialize<'de>,
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

// use csr_indexed_db::*;

// async fn use_trait<T: Crud<ScrollPositionKey, i32>>(val: T) {
//     val.del(&ScrollPositionKey { path: "".into(), query: "".into() }).await;
// }


// async fn test() -> Result<(), csr_indexed_db::Error> {
//   use csr_indexed_db::*;
//   use rexie::{ObjectStore, Rexie, TransactionMode};

//   let i = IndexedDb::new().await?;
//   // let i = IndexedDb { rexie: Rexie::builder("cache_v5")
//   //   .version(1)
//   //   .add_object_store(ObjectStore::new("post_closed_comments"))
//   //   .add_object_store(ObjectStore::new("comment_drafts"))
//   //   .add_object_store(ObjectStore::new("query_gets"))
//   //   .add_object_store(ObjectStore::new("scroll_positions"))
//   //   .build()
//   //   .await? };
//   //
//   // IndexedDb::get(&self, key)
//   //
//   // let c = i as Crud<ScrollPositionKey, i32>;
//   i.set(&GetPost { id: None, comment_id: None }, &23);
//   let n: i32 = i.get(&GetPost { id: None, comment_id: None }).await.unwrap().unwrap();
//   i.del(&ScrollPositionKey { path: "".into(), query: "".into() }).await;
//   // use_trait(i).await;

//   // let obj: &dyn Crud<ScrollPositionKey, i32> = &i;


//   Ok(())
// }


// // Trait with two generics: A and B
// trait DualOps<A, B> {
//     fn op_a(&self, value: A) -> String;
//     fn op_b(&self, value: B) -> String;
// }

// // Concrete struct (no generics)
// struct MyStruct;

// // Implement DualOps for MyStruct with chosen types for A and B
// impl DualOps<i32, &str> for MyStruct {
//     fn op_a(&self, value: i32) -> String {
//         format!("op_a received: {}", value)
//     }
//     fn op_b(&self, value: &str) -> String {
//         format!("op_b received: {}", value)
//     }
// }

// fn main() {
//     let obj = MyStruct;
//     println!("{}", obj.op_a(42));      // "op_a received: 42"
//     println!("{}", obj.op_b("hello")); // "op_b received: hello"
// }
