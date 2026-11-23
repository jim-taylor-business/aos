# A.O.S

**A Lemmy client written in Rust, built to be read like a newspaper.**

A.O.S renders on the server for the fastest time to rendered page possible, hydrates as WebAssembly for instant
interaction, and talks to
whichever Lemmy instance you point it at. Posts and comments flow into columns, your place is remembered, and reading,
navigating and voting all work before a single byte of WebAssembly has loaded.

[![License: MIT](https://img.shields.io/badge/license-MIT-yellow.svg)](LICENSE)
[![Rust edition 2024](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org/)
[![Leptos 0.8](https://img.shields.io/badge/leptos-0.8-blue.svg)](https://leptos.dev/)
[![Axum](https://img.shields.io/badge/server-axum-5c6bc0.svg)](https://github.com/tokio-rs/axum)
[![Tailwind 4 + daisyUI 5](https://img.shields.io/badge/style-tailwind%204%20%2B%20daisyUI%205-38bdf8.svg)](https://daisyui.com/)
[![Lemmy API v3](https://img.shields.io/badge/lemmy-api%20v3-00a86b.svg)](https://join-lemmy.org/)

[Lemmy support](#lemmy-support) &middot;
[Technical design](#technical-design) &middot;
[Visual and interactive design](#visual-and-interactive-design) &middot;
[How A.O.S compares](#how-aos-compares) &middot;
[Contributing](#contributing)

<img src="docs/media/hero.webp" width="100%">

## Features

- <big>**_Lemmy support across your instance_**</big> - Browse feeds, communities and users. Read threaded comments.
  Vote, save, reply, edit, report and block. Search posts, comments, communities and users in context. Sign in to your
  own account.

- <big>**_Cutting egde Rust architecture_**</big> - One Rust crate compiled for the server (Axum SSR) and the browser
  (WebAssembly). Direct-to-instance API calls, a layered memory and IndexedDB cache, forms that work without JavaScript.

- <big>**_Radically reactive reader first design_**</big> - A newspaper-column reading surface with sideways scrolling,
  a hero post with its top comment, press-and-hold action bars, collapsible comment threads, three themes, pagination
  state preserved in the URL.

- <big>**_And more_**</big> - English and French UI translations, offline reading from cache, saved drafts, saved scroll
  position, and a blazingly fast feel (of course).

## Lemmy support

### Supported today

| Feature         | Supported                                                                                                                        |
|-----------------|----------------------------------------------------------------------------------------------------------------------------------|
| **Feeds**       | All, Local and Subscribed feeds, sorted by Active, Hot, New, Old, Top, Controversial, Scaled or New Comments. 50 posts per page. |
| **Communities** | Header with icon, banner, description and rules. Subscribe and unsubscribe.                                                      |
| **Posts**       | Full post with body, image, embed description, link host and language.                                                           |
| **Comments**    | 100% Lemmy compatible Markdown. Full threaded tree up to 128 levels deep. Missing children load on demand.                       |
| **Voting**      | Upvote and downvote posts and comments, with optimistic updates.                                                                 |
| **Saving**      | Save and unsave posts and comments.                                                                                              |
| **Writing**     | Comment on a post, reply to a comment, edit your own comments. Drafts are kept as you type.                                      |
| **Moderation**  | Report a post with a required reason. Block a user.                                                                              |
| **Search**      | Posts, Comments, Communities and Users, scoped to the whole instance, a community, a user or a single post.                      |
| **Users**       | Profile with avatar, banner and bio. Posts and comments merged into one newest-first activity view, grouped by post.             |
| **Mod log**     | Look up why a comment was removed or its author banned.                                                                          |
| **Accounts**    | Username and password login and logout.                                                                                          |
| **Branding**    | Instance name, icon, description and language list shape the UI. Page titles read like `AOS - <instance> - <description>`.       |

### Federation-aware by default

Lemmy is a federated network, and A.O.S shows it honestly instead of hiding it.

- A.O.S is not tied to one Lemmy server. It starts on `lemmy.world`, and you can switch to any other instance by
  pressing and holding the site name in the top bar, typing a domain, and pressing Enter.
- Community names carry their home instance (`rust@programming.dev`) when the community is not local to the instance you
  are on.
- Author handles show a domain only when it differs from your selected instance.
- Posts that link to another site show the host they came from.
- Federated posts include a link to the original on their home instance.
- Posts that carry a link or an image get a one-tap shortcut to archive it on archive.ph.
- Each post and comment sets a `lang` attribute from its Lemmy language, which helps screen readers, hyphenation and
  font selection.

### Filters, safety and moderation

- NSFW content is hidden unless you switch it on.
- Posts by authors who are banned from their community are hidden unless you opt in.
- Removed, deleted, banned and blocked comments render as clear placeholders, with a way to look up the mod log or
  reveal blocked content.
- Reports require a reason, and blocking a user is available from the post menu.

## Technical design

### Architecture

A.O.S is a single [Leptos](https://leptos.dev/) crate. The same components, request types and Lemmy API trait run in
both places, so the server-rendered HTML and the hydrated app are built from the same code and stay in step.

### Using Rust, Leptos and standard HTTP to provide a unique experience

1. You open a URL. The server reads your cookies (`instance`, `jwt`, `theme` and the language
   preference).
2. Axum hands the request to Leptos, which renders the route in async SSR mode. The same components fetch from your
   Lemmy instance through `reqwest`.
3. The finished HTML streams back with title, content, theme and language already in place, along with the WebAssembly
   bundle.
4. The bundle hydrates the page. From here on, links, votes and menus act in place, and requests go from the browser
   straight to your instance.
5. Every successful GET is written to memory and IndexedDB, so revisits are instant and survive reloads.

### Where state lives

| Where                      | Why                                                                                           | What                                                                                                                                                   |
|----------------------------|-----------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Cookies**                | The server can read them, so the first render already uses your instance, theme and language. | Authentication, instance, theme, and language information.                                                                                             |
| **URL**                    | Every view is linkable, and the back button does the right thing.                             | Route, plus the query parameters `list` (All, Local, Subscribed), `sort`, `bans` (NSFW and banned toggles), `kind` (search type) and `page` (cursors). |
| **Memory**                 | Instant revisits within a session.                                                            | Five in-memory caches for listings, posts, comments, search and users.                                                                                 |
| **IndexedDB** (`cache_v5`) | Survives reloads. Powers offline reading.                                                     | API responses, scroll positions, collapsed comments, drafts.                                                                                           |

### Pagination you can bookmark

Lemmy paginates with cursors. A.O.S keeps them in the URL as a JSON list of offset and cursor pairs. Filter and sort sit
beside it as JSON-quoted values, for example `/?list="Local"&sort="Hot"&page=[[50,"P3408409"]]`. The
server renders from that chain, so a deep scroll can be reloaded or shared. Infinite scroll uses an intersection
observer at the end of the columns and navigates with `replace`, so it never floods your history. Without JavaScript, a
plain Next link takes its place.

### Caching and offline

- **Typed request forms are the cache keys**, combined with the login and instance, so one user's data never answers
  another's request.
- **When the browser goes offline**, GET requests are answered from IndexedDB. If there is no cached copy, you get a
  warning with a Retry button instead of a blank page.
- **Writes pause** while offline, and resume when the connection returns.
- **A successful vote, save or reply patches the cached post and listing in place**, so the feed you go back to is
  already up to date.

### Progressive enhancement

Menus are `<details>` elements, navigation is real `<a href>` links, and actions such as voting, saving, reporting,
blocking and login are server functions behind ordinary forms. They work before WebAssembly loads, and Leptos upgrades
them in place afterwards. A dedicated `POST /l` route carries the page you were on through the login form, so signing in
returns you where you started.

### Markdown, errors and i18n

- **Markdown** uses a `pulldown-cmark` fork with strikethrough, tables, superscript, subscript and containers, plus
  Lemmy-style and plain-URL link detection. Raw HTML in content is escaped, never injected.
- **Errors** are typed (`LemmyAppError`), including an explicit offline case. They show as a compact banner with
  expandable details and a retry button.
- **Localisation** uses `leptos_i18n`, with translation keys checked at compile time. English and French ship today, and
  relative times ("15h 53m ago") are localised too.

### Project structure

```text
.
├── src/
│   ├── main.rs       Axum SSR server (and the client-only entry point), POST /l
│   ├── lib.rs        App shell, routes and HTML template
│   ├── prelude.rs    Shared imports, cookies, Markdown, wheel gestures, alerts
│   ├── client.rs     LemmyApi trait and LemmyClient: reqwest on the server, gloo-net in the browser
│   ├── db.rs         IndexedDB storage (cache_v5, via Rexie)
│   ├── errors.rs     Typed errors, including the offline case
│   ├── nav.rs        Top bar: instance switch, filter, sort, search, theme, language
│   ├── overview.rs   Feeds: community header, cursor pagination, infinite scroll
│   ├── listing.rs    Post cards
│   ├── hero.rs       The expanded first post with its top comment
│   ├── toolbar.rs    Vote, save, comment, report, block and archive actions
│   ├── post.rs       Post page and comment columns
│   ├── comment.rs    Comment tree: folding, press-and-hold actions, drafts
│   ├── search.rs     Posts, comments, communities and users, scoped by context
│   ├── user.rs       Profiles and activity
│   ├── login.rs      Lemmy login
│   └── icon.rs       SVG sprite icons
├── locales/          English and French translations
├── public/           Fonts, icon sprite, favicon and web manifest
├── input.css         Tailwind entry point
├── build.rs          Compile-time i18n generation
└── Cargo.toml        Dependencies, features and Leptos metadata
```

### Deployment

The repository includes a Gitea Actions workflow in `.gitea/workflows/` that builds the release server and site, then
uploads the app files to a remote server over SSH. It then runs a command of your choosing, such as one that starts the
server in the background. The workflow is triggered by a push to the `main` branch.

<details>
<summary>Secrets the workflow expects</summary>

- `SSH_HOST`
- `SSH_PORT`
- `SSH_USER`
- `SSH_KEY`
- `WORK_DIR`
- `SSH_CMD`

The deployment command must start the uploaded binary with the Leptos environment variables set (`LEPTOS_SITE_ROOT` and
`LEPTOS_SITE_ADDR`), and serve `target/site` alongside it.

</details>

## Visual and interactive design

A.O.S treats reading as the primary focus. There is very little chrome: one slim top bar, icon-only toolbars, and menus
that stay folded until you need them. The rest of the screen is content.

### A reading surface built from columns

- **Columns, not an endless scroll.** Content flows into newspaper-like columns that fill top to bottom and then
  continue to the right.
- **The mouse wheel just works.** Vertical wheel input is converted to sideways scrolling, with a small gesture
  classifier that leaves deliberate sideways trackpad swipes to the browser. Text boxes opt out, so scrolling inside a
  reply behaves normally.
- **The next page arrives on its own.** Reaching the end of the columns loads the next page of posts.
- **Your place is remembered.** Scroll position is saved per path and query, and restored when you come back.
- **A hero post.** The first post of a feed opens in full, with its thumbnail, toolbar, body and the top comment
  previewed underneath.
- **On a phone, it is just a feed.** Below the mobile breakpoint the same markup becomes an ordinary vertical page.

### Navigation

The top bar holds everything in a handful of folded menus: feed filter, sort, search scope, content toggles (NSFW and
banned), theme, language and account. Menus are native `<details>` elements, so they work without scripting and close
one another. On small screens the instance name collapses to its icon and search expands to full width. Press and hold
the site name to change instance.

Search is contextual. Open it from a community, a user or a post, and it searches within that context, with the scope
shown in the field.

### Comments you can fold, probe and refill

- **Tap or click** a comment to fold its thread. Open and closed state is saved per post, and selecting text never
  triggers it.
- **Press and hold for half a second**, with a finger or a mouse, to reveal the action bar: vote, save, reply, edit your
  own comments, and highlight everything by one author.
- **A replies badge** loads children that were not in the first response.
- **Left-edge colour** shows status at a glance: the original poster, comments you upvoted and comments you downvoted.
- **Indentation stops at level 8**, and top-level threads alternate background tints, so deep conversations stay legible
  on a narrow screen.
- **Drafts are saved as you type**, for replies, edits and top-level comments, and cleared when you send.
- **Placeholders** for removed, banned, deleted and blocked comments carry their own icon, and some of them can be
  revealed.

### Themes

A.O.S ships `dark`, `light` and `retro` through daisyUI's `data-theme`. The choice is stored in a cookie that the server
reads, so pages are themed from the very first paint. Vote colours follow the theme: secondary is upvote, primary is
downvote, and accent marks saved items and the original poster.

### Feedback

Loading, warning and error states are inline soft alerts in the flow of the page, rather than spinners that block it.
Posts show unread comment counts, and times are compact and localised.

### Features the visual and interactive design makes possible

| Design decision                    | What you get                                                                                                        |
|------------------------------------|---------------------------------------------------------------------------------------------------------------------|
| Newspaper-style columns            | More content per screen, with the next item always beside you, and a sideways scroll that feels like turning pages. |
| Wheel-to-horizontal translation    | Mouse users are not forced to hold Shift, and trackpad users keep native sideways swipes.                           |
| Hero post with top comment         | The conversation starter is readable without leaving the feed.                                                      |
| Press-and-hold action bar          | A clean thread by default, with every action one gesture away on touch and mouse alike.                             |
| Tap to fold, state saved           | Long threads become manageable, and they stay the way you left them.                                                |
| Coloured edges and highlight       | Status and authorship are visible without extra labels, and one commenter can be followed through a thread.         |
| Draft autosave                     | A reply survives a reload, a crash or an accidental tab close.                                                      |
| Cookie-driven themes               | Three looks with no flash of the wrong theme.                                                                       |
| Icon sprite and `em` sizing        | Crisp icons that scale with text size.                                                                              |
| Archive shortcut on external links | One tap to archive a link that may disappear or sit behind a paywall.                                               |

## How A.O.S compares

Lemmy has a healthy ecosystem of clients, and many of them are excellent. This is a map of where A.O.S sits, not a
ranking. It is a snapshot from each project's public pages (November 2026), so check their repositories for current
status.

| Client                                                | Runs as                             | Built with                                                 | Known for                                                                                                                                                  |
|-------------------------------------------------------|-------------------------------------|------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------|
| A.O.S                                                 | Web, server-rendered, then hydrated | Rust, Leptos 0.8, Axum, WebAssembly, Tailwind 4, daisyUI 5 | Sideways column reading, one Rust codebase for server and browser, an IndexedDB cache with offline reading                                                 |
| [lemmy-ui](https://github.com/LemmyNet/lemmy-ui)      | Web, server-rendered                | Inferno                                                    | The official web app that ships with Lemmy                                                                                                                 |
| [Photon](https://github.com/Xyphyn/photon)            | Web, self-hostable                  | Svelte 5, SvelteKit                                        | Broad Lemmy feature coverage including mod and admin tools, optional server-side rendering, Lemmy and PieFed                                               |
| [Voyager](https://github.com/aeharding/voyager)       | Web (PWA), iOS, Android             | React, Ionic, Capacitor, TypeScript                        | A mobile-first, Apollo-style interface with native apps and push notifications, for Lemmy and PieFed                                                       |
| [Alexandrite](https://github.com/sheodox/alexandrite) | Web, client-rendered                | Svelte, SvelteKit                                          | A desktop-first UI that can show a post in a second column beside the feed, with infinite scroll and multi-account switching                               |
| [mlmym](https://github.com/rystaf/mlmym)              | Web, server-rendered                | Go templates                                               | An old-Reddit-style desktop interface that works without JavaScript. The original is no longer maintained, and a fork keeps it running for old.lemmy.world |

### What sets A.O.S apart

- **It reads sideways.** Columns, the wheel-to-sideways gesture and the hero post are the central idea of A.O.S, not a
  skin over a familiar feed. Alexandrite also puts a post beside the feed in a second column, while A.O.S makes columns
  the whole reading surface, for feeds and comments alike.
- **One language, end to end.** The server and the browser compile against Lemmy's own request and response types, so an
  API change shows up as a compile error instead of a runtime surprise.
- **Readable before anything loads.** Pages arrive as real HTML with real links and forms, then WebAssembly takes over.
  Client-side apps such as Voyager and Alexandrite render in the browser once their JavaScript has loaded, while A.O.S
  shares its server-rendered approach with lemmy-ui and mlmym.
- **A local-first cache.** Responses, scroll position, collapsed threads and drafts live in IndexedDB, so a flaky
  connection costs you little.
- **A compact codebase.** Seventeen source files and no database dependency.

## Roadmap

There are a few fundamentals still to be done, and every one of them is a good place to contribute, but I would like to
continue and extend the unique user experience in their implementation. There are benefits and compromises to the
current
design that can be improved with careful thought and experimentation. The following are some of the next steps:

- Creating posts and communities.
- Multiple accounts and account switching.
- More locales and themes.
- Two-factor (TOTP) login.
- An inbox for replies, mentions and private messages. The client layer and an icon are already in place.
- Desktop and mobile packaging. A web manifest is already included.

## Contributing

Contributions are welcome, whether that is code, testing or funding.

Fork the repository and create a feature branch. Make the smallest focused change that addresses the issue.

Open an issue with reproduction steps or the proposed behaviour before large or disruptive work.

## License

Released under the [MIT License](LICENSE).

## AI usage in this app

This project is not "vibe-coded" by AI. However, I have no problem using AI autocomplete tools to speed up development
as long as all AI suggestions are completely reviewed and tested by a human before being committed. This promotional
page did use
significant suggested text from an LLM based on the project's human written source and browser rendering.

## Acknowledgements

A.O.S makes extensive use of [Lemmy](https://join-lemmy.org/), [Leptos](https://leptos.dev/),
[leptos-use](https://leptos-use.rs/), [leptos_i18n](https://github.com/Baptistemontan/leptos_i18n),
[Axum](https://github.com/tokio-rs/axum), [Tokio](https://tokio.rs/), [Tailwind CSS](https://tailwindcss.com/),
[daisyUI](https://daisyui.com/), [pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark),
[Rexie](https://github.com/SaifAqqad/rexie) and the Adwaita Sans typeface.
