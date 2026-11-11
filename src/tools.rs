pub fn markdown_to_html(content: &str, linkify: bool) -> String {
  let mut options = pulldown_cmark::Options::empty();
  options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
  options.insert(pulldown_cmark::Options::ENABLE_TABLES);
  options.insert(pulldown_cmark::Options::ENABLE_SUPERSCRIPT);
  options.insert(pulldown_cmark::Options::ENABLE_SUBSCRIPT);
  options.insert(pulldown_cmark::Options::ENABLE_CONTAINER_EXTENSIONS);
  if linkify {
    options.insert(pulldown_cmark::Options::ENABLE_LINKIFY_LEMMY);
    options.insert(pulldown_cmark::Options::ENABLE_LINKIFY_HTTP);
  }
  markdown_to_html_with_options(content, options)
}

pub fn markdown_to_html_with_options(content: &str, options: pulldown_cmark::Options) -> String {
  let parser = pulldown_cmark::Parser::new_ext(content, options);

  let custom = parser.map(|event| match event {
    pulldown_cmark::Event::Html(text) => {
      let er = format!("<p>{}</p>", html_escape::encode_safe(&text).to_string());
      pulldown_cmark::Event::Html(er.into())
    }
    pulldown_cmark::Event::InlineHtml(text) => {
      let er = html_escape::encode_safe(&text).to_string();
      pulldown_cmark::Event::InlineHtml(er.into())
    }
    _ => event,
  });
  let mut safe_html = String::new();
  pulldown_cmark::html::push_html(&mut safe_html, custom);

  safe_html
}
