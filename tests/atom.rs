use std::borrow::Cow;
use std::str::FromStr;

use instant_feeds::{Category, Entry, Feed, Generator, Link, Person, Text};
use jiff::Timestamp;

#[test]
fn write_feed() {
    let expected = concat!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n",
        r#"<feed xmlns="http://www.w3.org/2005/Atom">"#,
        "<title>Example Blog</title>",
        "<id>https://example.com/feed.xml</id>",
        "<updated>2024-06-03T12:00:00Z</updated>",
        "<author><name>Example Author</name></author>",
        "<icon>https://example.com/favicon.ico</icon>",
        r#"<link href="https://example.com/feed.xml" rel="self" type="application/atom+xml" />"#,
        r#"<link href="https://example.com/" rel="alternate" type="text/html" />"#,
        r#"<subtitle xml:lang="en">Notes on Rust and XML</subtitle>"#,
        "<entry>",
        "<title>Parsing XML in Rust</title>",
        "<id>https://example.com/posts/parsing-xml.html</id>",
        "<updated>2024-06-03T12:00:00Z</updated>",
        r#"<category term="rust" />"#,
        r#"<category term="xml" />"#,
        r#"<link href="https://example.com/posts/parsing-xml.html" rel="alternate" "#,
        r#"type="text/html" title="Parsing XML in Rust" />"#,
        "<published>2024-06-01T12:00:00Z</published>",
        r#"<summary xml:lang="en">How to map XML documents to Rust types</summary>"#,
        r#"<content xml:lang="en" type="html">"#,
        "&lt;p&gt;Use &lt;code&gt;FromXml&lt;/code&gt; to parse documents.&lt;/p&gt;",
        "</content>",
        "</entry>",
        "<entry>",
        "<title>Generating Atom feeds</title>",
        "<id>https://example.com/posts/atom-feeds.html</id>",
        "<updated>2024-05-20T12:00:00Z</updated>",
        r#"<link href="https://example.com/posts/atom-feeds.html" rel="alternate" "#,
        r#"type="text/html" title="Generating Atom feeds" />"#,
        "<published>2024-05-20T12:00:00Z</published>",
        r#"<summary xml:lang="en">Publishing a feed for your site</summary>"#,
        r#"<content xml:lang="en" type="html">"#,
        "&lt;p&gt;See &lt;a href=&quot;https://www.rfc-editor.org/rfc/rfc4287&quot;&gt;",
        "RFC 4287&lt;/a&gt; for details.&lt;/p&gt;",
        "</content>",
        "</entry>",
        "</feed>",
    );

    assert_eq!(example_feed().to_string(), expected);
}

#[test]
fn round_trip() {
    let feed = example_feed();
    let xml = feed.to_string();
    assert_eq!(Feed::parse(&xml).unwrap(), feed);
}

#[test]
fn optional_elements() {
    let updated = timestamp("2024-06-01T12:00:00Z");
    let mut feed = Feed::new("urn:example", "Example", updated);
    feed.categories.push(
        Category::new("rust")
            .with_scheme("https://example.com/tags")
            .with_label("Rust"),
    );
    feed.contributors.push(
        Person::new("Jane Doe")
            .with_uri("https://example.com/jane")
            .with_email("jane@example.com"),
    );
    feed.generator = Some(
        Generator::new("Example Generator")
            .with_uri("https://example.com/generator")
            .with_version("1.0"),
    );
    feed.logo = Some(Cow::Borrowed("https://example.com/logo.png"));
    feed.rights = Some(Text::plain("© 2024 Example"));

    let mut entry = Entry::new("urn:example:1", "Episode 1", updated);
    entry.authors.push(Person::new("John Doe"));
    entry.contributors.push(Person::new("Jane Doe"));
    entry.links.push(
        Link::new("https://example.com/episode-1.mp3")
            .with_rel("enclosure")
            .with_mime_type("audio/mpeg")
            .with_hreflang("en")
            .with_length(1337),
    );
    entry.rights = Some(Text::html("<b>CC BY 4.0</b>"));
    feed.entries.push(entry);

    let expected = concat!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n",
        r#"<feed xmlns="http://www.w3.org/2005/Atom">"#,
        "<title>Example</title>",
        "<id>urn:example</id>",
        "<updated>2024-06-01T12:00:00Z</updated>",
        r#"<category term="rust" scheme="https://example.com/tags" label="Rust" />"#,
        "<contributor><name>Jane Doe</name><uri>https://example.com/jane</uri>",
        "<email>jane@example.com</email></contributor>",
        r#"<generator uri="https://example.com/generator" version="1.0">"#,
        "Example Generator</generator>",
        "<logo>https://example.com/logo.png</logo>",
        "<rights>© 2024 Example</rights>",
        "<entry>",
        "<title>Episode 1</title>",
        "<id>urn:example:1</id>",
        "<updated>2024-06-01T12:00:00Z</updated>",
        "<author><name>John Doe</name></author>",
        "<contributor><name>Jane Doe</name></contributor>",
        r#"<link href="https://example.com/episode-1.mp3" rel="enclosure" type="audio/mpeg" "#,
        r#"hreflang="en" length="1337" />"#,
        r#"<rights type="html">&lt;b&gt;CC BY 4.0&lt;/b&gt;</rights>"#,
        "</entry>",
        "</feed>",
    );

    let xml = feed.to_string();
    assert_eq!(xml, expected);
    assert_eq!(Feed::parse(&xml).unwrap(), feed);
}

#[test]
fn escaping() {
    let title = r#"Use-after-free in <Parser> & "friends""#;
    let mut feed = Feed::new("urn:example", title, timestamp("2024-06-01T12:00:00Z"));
    feed.links
        .push(Link::new("https://example.com/?a=1&b=2").with_title(title));

    let xml = feed.to_string();
    assert!(
        xml.contains("<title>Use-after-free in &lt;Parser&gt; &amp; &quot;friends&quot;</title>")
    );
    assert!(xml.contains(concat!(
        r#"<link href="https://example.com/?a=1&amp;b=2" "#,
        r#"title="Use-after-free in &lt;Parser&gt; &amp; &quot;friends&quot;" />"#,
    )));
    assert_eq!(Feed::parse(&xml).unwrap(), feed);
}

#[test]
fn parse_rfc4287_example() {
    let mut expected = Feed::new(
        "urn:uuid:60a76c80-d399-11d9-b93c-0003939e0af6",
        "Example Feed",
        timestamp("2003-12-13T18:30:02Z"),
    );
    expected.links.push(Link::new("http://example.org/"));
    expected.authors.push(Person::new("John Doe"));

    let mut entry = Entry::new(
        "urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a",
        "Atom-Powered Robots Run Amok",
        timestamp("2003-12-13T18:30:02Z"),
    );
    entry
        .links
        .push(Link::new("http://example.org/2003/12/13/atom03"));
    entry.summary = Some(Text::plain("Some text."));
    expected.entries.push(entry);

    assert_eq!(Feed::parse(RFC4287_EXAMPLE).unwrap(), expected);
}

#[test]
fn parse_ignores_extensions() {
    let feed = Feed::parse(EXTENDED).unwrap();
    assert_eq!(feed.title, Text::plain("Example"));
    assert_eq!(feed.updated, timestamp("2024-06-01T12:00:00Z"));
    assert_eq!(
        feed.authors,
        [Person::new("Jane Doe").with_email("jane@example.com")]
    );
    assert_eq!(
        feed.generator,
        Some(
            Generator::new("Example Generator")
                .with_uri("https://example.com/")
                .with_version("1.0")
        )
    );

    let [entry] = feed.entries.as_slice() else {
        panic!("expected exactly one entry, found {}", feed.entries.len());
    };

    assert_eq!(entry.title, Text::html("<b>Bold</b> news"));
    assert_eq!(
        entry.categories,
        [Category::new("news")
            .with_scheme("https://example.com/categories")
            .with_label("News")]
    );
    assert_eq!(entry.content, Some(Text::html("<p>Hello & welcome</p>")));
}

#[test]
fn parse_prefixed_namespace() {
    let xml = concat!(
        r#"<atom:feed xmlns:atom="http://www.w3.org/2005/Atom">"#,
        "<atom:title>Prefixed</atom:title>",
        "<atom:id>urn:example</atom:id>",
        "<atom:updated>2024-06-01T12:00:00Z</atom:updated>",
        "</atom:feed>",
    );

    let expected = Feed::new("urn:example", "Prefixed", timestamp("2024-06-01T12:00:00Z"));
    assert_eq!(Feed::parse(xml).unwrap(), expected);
}

#[test]
fn parse_borrows_from_input() {
    let xml = concat!(
        r#"<feed xmlns="http://www.w3.org/2005/Atom">"#,
        "<title>Example</title>",
        "<id>urn:example</id>",
        "<updated>2024-06-01T12:00:00Z</updated>",
        r#"<link href="https://example.com/" rel="alternate" type="text/html" title="Home" />"#,
        "<entry>",
        "<title>Post</title>",
        "<id>urn:example:1</id>",
        "<updated>2024-06-01T12:00:00Z</updated>",
        r#"<category term="news" />"#,
        "</entry>",
        "</feed>",
    );

    let feed = Feed::parse(xml).unwrap();
    assert!(matches!(feed.id, Cow::Borrowed("urn:example")));
    assert!(matches!(feed.title.value, Cow::Borrowed("Example")));

    let [link] = feed.links.as_slice() else {
        panic!("expected exactly one link, found {}", feed.links.len());
    };
    assert!(matches!(link.href, Cow::Borrowed("https://example.com/")));
    assert!(matches!(link.rel, Some(Cow::Borrowed("alternate"))));
    assert!(matches!(link.mime_type, Some(Cow::Borrowed("text/html"))));
    assert!(matches!(link.title, Some(Cow::Borrowed("Home"))));

    let [entry] = feed.entries.as_slice() else {
        panic!("expected exactly one entry, found {}", feed.entries.len());
    };
    let [category] = entry.categories.as_slice() else {
        panic!("expected one category, found {}", entry.categories.len());
    };
    assert!(matches!(category.term, Cow::Borrowed("news")));
}

#[test]
fn parse_errors() {
    let no_namespace = concat!(
        "<feed>",
        "<title>Example</title>",
        "<id>urn:example</id>",
        "<updated>2024-06-01T12:00:00Z</updated>",
        "</feed>",
    );
    assert!(Feed::parse(no_namespace).is_err());

    let missing_id = concat!(
        r#"<feed xmlns="http://www.w3.org/2005/Atom">"#,
        "<title>Example</title>",
        "<updated>2024-06-01T12:00:00Z</updated>",
        "</feed>",
    );
    assert!(Feed::parse(missing_id).is_err());

    let invalid_timestamp = concat!(
        r#"<feed xmlns="http://www.w3.org/2005/Atom">"#,
        "<title>Example</title>",
        "<id>urn:example</id>",
        "<updated>2024-06-01</updated>",
        "</feed>",
    );
    assert!(Feed::parse(invalid_timestamp).is_err());

    let xhtml = concat!(
        r#"<feed xmlns="http://www.w3.org/2005/Atom">"#,
        r#"<title type="xhtml"><div xmlns="http://www.w3.org/1999/xhtml">Example</div></title>"#,
        "<id>urn:example</id>",
        "<updated>2024-06-01T12:00:00Z</updated>",
        "</feed>",
    );
    assert!(Feed::parse(xhtml).is_err());
}

fn example_feed() -> Feed<'static> {
    let mut feed = Feed::new(
        "https://example.com/feed.xml",
        "Example Blog",
        timestamp("2024-06-03T12:00:00Z"),
    );
    feed.subtitle = Some(Text::plain("Notes on Rust and XML").with_lang("en"));
    feed.links.push(
        Link::new("https://example.com/feed.xml")
            .with_rel("self")
            .with_mime_type("application/atom+xml"),
    );
    feed.links.push(
        Link::new("https://example.com/")
            .with_rel("alternate")
            .with_mime_type("text/html"),
    );
    feed.icon = Some("https://example.com/favicon.ico".into());
    feed.authors.push(Person::new("Example Author"));

    let mut entry = post_entry(
        "parsing-xml",
        "Parsing XML in Rust",
        timestamp("2024-06-01T12:00:00Z"),
        timestamp("2024-06-03T12:00:00Z"),
    );
    entry.categories.push(Category::new("rust"));
    entry.categories.push(Category::new("xml"));
    entry.summary = Some(Text::plain("How to map XML documents to Rust types").with_lang("en"));
    entry.content =
        Some(Text::html("<p>Use <code>FromXml</code> to parse documents.</p>").with_lang("en"));
    feed.entries.push(entry);

    let mut entry = post_entry(
        "atom-feeds",
        "Generating Atom feeds",
        timestamp("2024-05-20T12:00:00Z"),
        timestamp("2024-05-20T12:00:00Z"),
    );
    entry.summary = Some(Text::plain("Publishing a feed for your site").with_lang("en"));
    entry.content = Some(
        Text::html(
            r#"<p>See <a href="https://www.rfc-editor.org/rfc/rfc4287">RFC 4287</a> for details.</p>"#,
        )
        .with_lang("en"),
    );
    feed.entries.push(entry);

    feed
}

fn post_entry(
    slug: &str,
    title: &'static str,
    published: Timestamp,
    updated: Timestamp,
) -> Entry<'static> {
    let url = format!("https://example.com/posts/{slug}.html");
    let mut entry = Entry::new(url.clone(), title, updated);
    entry.links.push(
        Link::new(url)
            .with_rel("alternate")
            .with_mime_type("text/html")
            .with_title(title),
    );
    entry.published = Some(published);
    entry
}

fn timestamp(s: &str) -> Timestamp {
    Timestamp::from_str(s).unwrap()
}

const RFC4287_EXAMPLE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">

  <title>Example Feed</title>
  <link href="http://example.org/"/>
  <updated>2003-12-13T18:30:02Z</updated>
  <author>
    <name>John Doe</name>
  </author>
  <id>urn:uuid:60a76c80-d399-11d9-b93c-0003939e0af6</id>

  <entry>
    <title>Atom-Powered Robots Run Amok</title>
    <link href="http://example.org/2003/12/13/atom03"/>
    <id>urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a</id>
    <updated>2003-12-13T18:30:02Z</updated>
    <summary>Some text.</summary>
  </entry>

</feed>
"#;

const EXTENDED: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom" xmlns:dc="http://purl.org/dc/elements/1.1/" xml:lang="en">
  <title type="text">Example</title>
  <dc:title>Not the Atom title</dc:title>
  <id>urn:example</id>
  <updated>2024-06-01T14:00:00+02:00</updated>
  <generator uri="https://example.com/" version="1.0">Example Generator</generator>
  <author>
    <name>Jane Doe</name>
    <email>jane@example.com</email>
  </author>
  <entry>
    <title type="html">&lt;b&gt;Bold&lt;/b&gt; news</title>
    <id>urn:example:1</id>
    <updated>2024-06-01T12:00:00Z</updated>
    <category term="news" scheme="https://example.com/categories" label="News"/>
    <dc:creator>Jane Doe</dc:creator>
    <content type="html"><![CDATA[<p>Hello & welcome</p>]]></content>
  </entry>
</feed>
"#;
