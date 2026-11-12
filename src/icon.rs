use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
  Eye,
  EyeSlash,
  Notifications,
  Donate,
  Search,
  Upvote,
  Downvote,
  Crosspost,
  VerticalDots,
  Report,
  Comments,
  Block,
  Save,
  Reply,
  External,
  Link,
  Translate,
  Palette,
  Pencil,
  Highlighter,
  Filter,
  Sort,
  Community,
  User,
  SignIn,
  SignOut,
  Eraser,
  Hammer,
  Archive,
  History,
  Subscribe,
  Rules,
  Ban,
  Cube,
}

impl Icon {
  pub fn as_str(&self) -> &'static str {
    match self {
      Icon::Block => "block",
      Icon::Comments => "comments",
      Icon::Crosspost => "crosspost",
      Icon::Donate => "donate",
      Icon::Downvote => "downvote",
      Icon::Eye => "eye",
      Icon::EyeSlash => "eye-slash",
      Icon::Notifications => "notifications",
      Icon::Report => "report",
      Icon::Save => "save",
      Icon::Search => "search",
      Icon::Upvote => "upvote",
      Icon::VerticalDots => "vertical-dots",
      Icon::Reply => "reply",
      Icon::External => "external",
      Icon::Link => "link",
      Icon::Translate => "translate",
      Icon::Palette => "palette",
      Icon::Pencil => "pencil",
      Icon::Highlighter => "highlighter",
      Icon::Filter => "filter",
      Icon::Sort => "sort",
      Icon::Community => "community",
      Icon::User => "user",
      Icon::SignIn => "signin",
      Icon::SignOut => "signout",
      Icon::Eraser => "eraser",
      Icon::Hammer => "hammer",
      Icon::Archive => "archive",
      Icon::History => "history",
      Icon::Subscribe => "subscribe",
      Icon::Rules => "rules",
      Icon::Ban => "ban",
      Icon::Cube => "cube",
    }
  }
}

#[component]
pub fn Icon(#[prop(into)] icon: Signal<Icon>, #[prop(optional)] class: Option<TextProp>) -> impl IntoView {
  let href = Signal::derive(move || format!("/icons.svg#{}", icon.get().as_str()));
  view! {
    <svg attr:class={class.unwrap_or("".into())} width="1.5em" height="1.5em">
      <use_ href={href} xlink:href={href} />
    </svg>
  }
}
