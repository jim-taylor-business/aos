// use crate::{errors::LemmyAppError, i18n::*, lemmy_client::*};
// use lemmy_api_common::{
//   community::*,
//   lemmy_db_schema::{ListingType, SortType},
//   lemmy_db_views_actor::structs::CommunityView,
// };
// use leptos::*;
// use leptos_router::*;

// #[component]
// pub fn Test() -> impl IntoView {
//   let _i18n = use_i18n();

//   let error = expect_context::<RwSignal<Option<(LemmyAppError, Option<RwSignal<bool>>)>>>();

//   let ssr_unread = Resource::new(
//     move || (/* authenticated.get() */),
//     move |()/* user */| async move {
//       logging::log!("oeoe");

//       let result =
//       // if user == Some(false) {
//       //   LemmyClient.unread_count().await
//       // } else {
//         LemmyClient.unread_count().await;
//       // };

//       logging::log!("{:#?}", result);

//       match result {
//         Ok(o) => {
//           logging::log!("wewe");
//           Ok(o)
//         },
//         Err(e) => {
//           // error.set(Some((e.clone(), None)));
//           Err(e)
//         }
//       }
//     },
//   );

//   // let ssr_unread = Resource::new(
//   //   move || (),
//   //   move |()| async move {
//   //     let form = ListCommunities {
//   //       type_: Some(ListingType::Local),
//   //       sort: Some(SortType::Hot),
//   //       limit: Some(6),
//   //       show_nsfw: None,
//   //       page: None,
//   //     };

//   //     let result = LemmyClient.list_communities(form).await;

//   //     match result {
//   //       Ok(o) => {
//   //         logging::log!("wewe");
//   //         Ok(o)
//   //       },
//   //       Err(e) => {
//   //         // error.set(Some((e.clone(), None)));
//   //         Err(e)
//   //       }
//   //     }
//   //     // match result {
//   //     //   Ok(o) => Some(o),
//   //     //   Err(e) => {
//   //     //     error.set(Some((e, None)));
//   //     //     None
//   //     //   }
//   //     // }
//   //   },
//   // );

//   view! {
//     <Transition fallback=|| {}>
//             <div class="badge badge-info badge-xs"> "TEST" </div>
//       {move || {
//         // format!("lalal {:#?}", ssr_unread.get().map(|x| x))
//         // ssr_unread.get().map(|x| view! { <div class="badge badge-error badge-xs"></div> })
//           ssr_unread
//               .get()
//               .map(|u| {
//                   // view! {
//                   //   <div class="badge badge-error badge-xs"></div>
//                   // }

//                   if let Ok(c) = u {
//                     // logging::log!("lalal {:#?}", c);
//                     // view! {
//                     //   <div class="badge badge-error badge-xs"> { format!("lalal {:#?}", c) } </div>
//                     // }
//                     view! {
//                       <div class="badge badge-error badge-xs"> { c.replies + c.mentions + c.private_messages } </div>
//                     }
//                   } else {
//                     view! {
//                       <div class="hidden"> "SO HIDDEN RIGHT NOW" </div>
//                     }
//                   }
//              }
//             )
//       }}

//     </Transition>

//   }
// }
