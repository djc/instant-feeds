//! Generate and parse [Atom] feeds
//!
//! This crate maps the Atom Syndication Format to Rust types using [`instant_xml`], with
//! timestamps represented as [`jiff::Timestamp`]. String values are stored as [`Cow<'a, str>`],
//! so constructing a feed accepts both borrowed and owned strings, and parsing a feed borrows
//! from the input where possible.
//!
//! All feed and entry metadata defined by RFC 4287 is supported, except for `atom:source` and
//! `xml:base`. Text and content are limited to plain text and HTML: XHTML, other media types,
//! and content referenced by `src` are not supported. Parsing returns an error for text or
//! content with an unsupported `type`, and ignores other unsupported elements and attributes.
//!
//! # Examples
//!
//! ```
//! use std::str::FromStr;
//!
//! use instant_feeds::{Entry, Feed, Link, Person, Text};
//! use jiff::Timestamp;
//!
//! let updated = Timestamp::from_str("2024-06-01T12:00:00Z").unwrap();
//! let mut feed = Feed::new("https://example.com/feed.xml", "Example Feed", updated);
//! feed.authors.push(Person::new("Example Author"));
//! feed.links.push(Link::new("https://example.com/feed.xml").with_rel("self"));
//!
//! let mut entry = Entry::new("https://example.com/posts/1", "First post", updated);
//! entry.summary = Some(Text::plain("A short summary"));
//! feed.entries.push(entry);
//!
//! let xml = feed.to_string();
//! assert_eq!(Feed::parse(&xml).unwrap(), feed);
//! ```
//!
//! [Atom]: https://www.rfc-editor.org/rfc/rfc4287
//! [`Cow<'a, str>`]: std::borrow::Cow

use std::borrow::Cow;
use std::fmt;

pub use instant_xml::Error;
use instant_xml::de::Node;
use instant_xml::ser::Context;
use instant_xml::{Deserializer, FromXml, Id, Kind, Serializer, ToXml};
use jiff::Timestamp;

/// An Atom feed document
///
/// Use the [`Display`] implementation (for example, via [`ToString::to_string()`]) to
/// serialize the feed as an XML document.
///
/// [`Display`]: std::fmt::Display
#[derive(Clone, Debug, Eq, FromXml, PartialEq, ToXml)]
#[xml(rename = "feed", ns(ATOM))]
#[non_exhaustive]
pub struct Feed<'a> {
    /// Human-readable title of the feed
    #[xml(borrow)]
    pub title: Text<'a>,
    /// Permanent, universally unique identifier of the feed
    #[xml(borrow)]
    pub id: Cow<'a, str>,
    /// Most recent instant in time when the feed was modified
    pub updated: Timestamp,
    /// Authors of the feed
    #[xml(borrow, rename = "author")]
    pub authors: Vec<Person<'a>>,
    /// Categories associated with the feed
    #[xml(borrow, rename = "category")]
    pub categories: Vec<Category<'a>>,
    /// Contributors to the feed
    #[xml(borrow, rename = "contributor")]
    pub contributors: Vec<Person<'a>>,
    /// Agent used to generate the feed
    #[xml(borrow)]
    pub generator: Option<Generator<'a>>,
    /// IRI of an image that provides iconic visual identification for the feed
    #[xml(borrow)]
    pub icon: Option<Cow<'a, str>>,
    /// References from the feed to Web resources
    #[xml(borrow, rename = "link")]
    pub links: Vec<Link<'a>>,
    /// IRI of an image that provides visual identification for the feed
    ///
    /// Unlike the `icon`, the logo is expected to have a 2:1 aspect ratio.
    #[xml(borrow)]
    pub logo: Option<Cow<'a, str>>,
    /// Information about rights held in and over the feed
    #[xml(borrow)]
    pub rights: Option<Text<'a>>,
    /// Human-readable description or subtitle of the feed
    #[xml(borrow)]
    pub subtitle: Option<Text<'a>>,
    /// Entries in the feed
    #[xml(borrow, rename = "entry")]
    pub entries: Vec<Entry<'a>>,
}

impl<'a> Feed<'a> {
    /// Creates a feed with the given metadata and no entries
    ///
    /// The `title` is interpreted as plain text.
    pub fn new(
        id: impl Into<Cow<'a, str>>,
        title: impl Into<Cow<'a, str>>,
        updated: Timestamp,
    ) -> Self {
        Self {
            title: Text::plain(title),
            id: id.into(),
            updated,
            authors: Vec::new(),
            categories: Vec::new(),
            contributors: Vec::new(),
            generator: None,
            icon: None,
            links: Vec::new(),
            logo: None,
            rights: None,
            subtitle: None,
            entries: Vec::new(),
        }
    }

    /// Parses a feed from an XML document
    ///
    /// Elements and attributes that this crate does not support are ignored.
    ///
    /// # Errors
    ///
    /// Returns an error if `xml` is not well-formed, if the root element is not an Atom `feed`,
    /// if a required element is missing, or if a value cannot be parsed.
    pub fn parse(xml: &'a str) -> Result<Self, Error> {
        instant_xml::from_str(xml)
    }
}

impl fmt::Display for Feed<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(XML_DECLARATION)?;
        instant_xml::to_writer(self, f).map_err(|_| fmt::Error)
    }
}

/// An entry in an Atom feed
#[derive(Clone, Debug, Eq, FromXml, PartialEq, ToXml)]
#[xml(rename = "entry", ns(ATOM))]
#[non_exhaustive]
pub struct Entry<'a> {
    /// Human-readable title of the entry
    #[xml(borrow)]
    pub title: Text<'a>,
    /// Permanent, universally unique identifier of the entry
    #[xml(borrow)]
    pub id: Cow<'a, str>,
    /// Most recent instant in time when the entry was modified in a significant way
    pub updated: Timestamp,
    /// Authors of the entry
    ///
    /// Entries without authors inherit the authors of the feed.
    #[xml(borrow, rename = "author")]
    pub authors: Vec<Person<'a>>,
    /// Categories associated with the entry
    #[xml(borrow, rename = "category")]
    pub categories: Vec<Category<'a>>,
    /// Contributors to the entry
    #[xml(borrow, rename = "contributor")]
    pub contributors: Vec<Person<'a>>,
    /// References from the entry to Web resources
    #[xml(borrow, rename = "link")]
    pub links: Vec<Link<'a>>,
    /// Instant in time associated with an early event in the life cycle of the entry
    pub published: Option<Timestamp>,
    /// Information about rights held in and over the entry
    #[xml(borrow)]
    pub rights: Option<Text<'a>>,
    /// Short summary, abstract, or excerpt of the entry
    #[xml(borrow)]
    pub summary: Option<Text<'a>>,
    /// Content of the entry
    ///
    /// Only plain text and HTML content are supported.
    #[xml(borrow)]
    pub content: Option<Text<'a>>,
}

impl<'a> Entry<'a> {
    /// Creates an entry with the given metadata
    ///
    /// The `title` is interpreted as plain text.
    pub fn new(
        id: impl Into<Cow<'a, str>>,
        title: impl Into<Cow<'a, str>>,
        updated: Timestamp,
    ) -> Self {
        Self {
            title: Text::plain(title),
            id: id.into(),
            updated,
            authors: Vec::new(),
            categories: Vec::new(),
            contributors: Vec::new(),
            links: Vec::new(),
            published: None,
            rights: None,
            summary: None,
            content: None,
        }
    }
}

/// Human-readable text, such as a title or summary
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct Text<'a> {
    /// The text, or HTML markup if `kind` is [`TextKind::Html`]
    pub value: Cow<'a, str>,
    /// How the `value` should be interpreted
    pub kind: TextKind,
    /// Natural language of the text, serialized as `xml:lang`
    pub lang: Option<Cow<'a, str>>,
}

impl<'a> Text<'a> {
    /// Creates plain text
    pub fn plain(value: impl Into<Cow<'a, str>>) -> Self {
        Self {
            value: value.into(),
            kind: TextKind::Text,
            lang: None,
        }
    }

    /// Creates text that contains HTML markup
    ///
    /// The markup is escaped during serialization, as the Atom specification requires.
    pub fn html(value: impl Into<Cow<'a, str>>) -> Self {
        Self {
            value: value.into(),
            kind: TextKind::Html,
            lang: None,
        }
    }

    /// Sets the natural language of the text, as a language tag like `en`
    pub fn with_lang(mut self, lang: impl Into<Cow<'a, str>>) -> Self {
        self.lang = Some(lang.into());
        self
    }
}

impl ToXml for Text<'_> {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        field: Option<Id<'_>>,
        serializer: &mut Serializer<'_, W>,
    ) -> Result<(), Error> {
        let Some(field) = field else {
            return Err(Error::UnexpectedState("text requires an element name"));
        };

        let Self { value, kind, lang } = self;
        let element = serializer.write_start(field.name, field.ns, None::<Context<0>>)?;
        if let Some(lang) = lang {
            serializer.write_attr("xml:lang", "", lang)?;
        }

        match kind {
            TextKind::Text => {}
            TextKind::Html => serializer.write_attr("type", "", "html")?,
        }

        serializer.end_start()?;
        value.serialize(None, serializer)?;
        serializer.write_close(element)
    }
}

impl<'xml: 'a, 'a> FromXml<'xml> for Text<'a> {
    #[inline]
    fn matches(id: Id<'_>, field: Option<Id<'_>>) -> bool {
        match field {
            Some(field) => id == field,
            None => false,
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut Deserializer<'cx, 'xml>,
    ) -> Result<(), Error> {
        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        let mut value = Cow::Borrowed("");
        let mut kind = TextKind::Text;
        let mut lang = None;
        while let Some(node) = deserializer.next() {
            match node? {
                Node::Attribute(attr) => match deserializer.attribute_id(&attr)? {
                    Id {
                        ns: "",
                        name: "type",
                    } => kind = TextKind::from_attribute(&attr.value)?,
                    Id {
                        ns: XML,
                        name: "lang",
                    } => lang = Some(attr.value),
                    _ => {}
                },
                Node::Text(text) => match value.is_empty() {
                    true => value = text,
                    false => value.to_mut().push_str(&text),
                },
                node @ (Node::Open(_) | Node::AttributeValue(_) | Node::Close { .. }) => {
                    return Err(Error::UnexpectedNode(format!("{node:?} in {field}")));
                }
            }
        }

        *into = Some(Self { value, kind, lang });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: Kind = Kind::Element;
}

/// How the value of a [`Text`] should be interpreted
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TextKind {
    /// Plain text without markup
    Text,
    /// HTML markup
    Html,
}

impl TextKind {
    fn from_attribute(value: &str) -> Result<Self, Error> {
        match value {
            "text" => Ok(Self::Text),
            "html" => Ok(Self::Html),
            _ => Err(Error::UnexpectedValue(format!(
                "unsupported text type {value:?}"
            ))),
        }
    }
}

/// A reference from a feed or entry to a Web resource
#[derive(Clone, Debug, Eq, FromXml, PartialEq, ToXml)]
#[xml(rename = "link", ns(ATOM))]
#[non_exhaustive]
pub struct Link<'a> {
    /// IRI of the referenced resource
    #[xml(attribute, borrow)]
    pub href: Cow<'a, str>,
    /// Link relation type, such as `alternate` or `self`
    ///
    /// Consumers interpret a link without a relation type as `alternate`.
    #[xml(attribute, borrow)]
    pub rel: Option<Cow<'a, str>>,
    /// Advisory media type of the referenced resource
    #[xml(attribute, borrow, rename = "type")]
    pub mime_type: Option<Cow<'a, str>>,
    /// Language of the referenced resource, as a language tag like `en`
    #[xml(attribute, borrow)]
    pub hreflang: Option<Cow<'a, str>>,
    /// Human-readable information about the link
    #[xml(attribute, borrow)]
    pub title: Option<Cow<'a, str>>,
    /// Advisory length of the referenced resource in octets
    #[xml(attribute)]
    pub length: Option<u64>,
}

impl<'a> Link<'a> {
    /// Creates a link to the given IRI
    pub fn new(href: impl Into<Cow<'a, str>>) -> Self {
        Self {
            href: href.into(),
            rel: None,
            mime_type: None,
            hreflang: None,
            title: None,
            length: None,
        }
    }

    /// Sets the link relation type
    pub fn with_rel(mut self, rel: impl Into<Cow<'a, str>>) -> Self {
        self.rel = Some(rel.into());
        self
    }

    /// Sets the advisory media type of the referenced resource
    pub fn with_mime_type(mut self, mime_type: impl Into<Cow<'a, str>>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    /// Sets the language of the referenced resource
    pub fn with_hreflang(mut self, hreflang: impl Into<Cow<'a, str>>) -> Self {
        self.hreflang = Some(hreflang.into());
        self
    }

    /// Sets the human-readable information about the link
    pub fn with_title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the advisory length of the referenced resource in octets
    pub fn with_length(mut self, length: u64) -> Self {
        self.length = Some(length);
        self
    }
}

/// A category associated with a feed or entry
#[derive(Clone, Debug, Eq, FromXml, PartialEq, ToXml)]
#[xml(rename = "category", ns(ATOM))]
#[non_exhaustive]
pub struct Category<'a> {
    /// Identifier of the category
    #[xml(attribute, borrow)]
    pub term: Cow<'a, str>,
    /// IRI that identifies the categorization scheme
    #[xml(attribute, borrow)]
    pub scheme: Option<Cow<'a, str>>,
    /// Human-readable label for display purposes
    #[xml(attribute, borrow)]
    pub label: Option<Cow<'a, str>>,
}

impl<'a> Category<'a> {
    /// Creates a category with the given identifier
    pub fn new(term: impl Into<Cow<'a, str>>) -> Self {
        Self {
            term: term.into(),
            scheme: None,
            label: None,
        }
    }

    /// Sets the IRI that identifies the categorization scheme
    pub fn with_scheme(mut self, scheme: impl Into<Cow<'a, str>>) -> Self {
        self.scheme = Some(scheme.into());
        self
    }

    /// Sets the human-readable label for display purposes
    pub fn with_label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// The agent used to generate a feed
#[derive(Clone, Debug, Eq, FromXml, PartialEq, ToXml)]
#[xml(rename = "generator", ns(ATOM))]
#[non_exhaustive]
pub struct Generator<'a> {
    /// IRI relevant to the agent
    #[xml(attribute, borrow)]
    pub uri: Option<Cow<'a, str>>,
    /// Version of the agent
    #[xml(attribute, borrow)]
    pub version: Option<Cow<'a, str>>,
    /// Human-readable name of the agent
    #[xml(direct)]
    pub name: Cow<'a, str>,
}

impl<'a> Generator<'a> {
    /// Creates a generator with the given name
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            uri: None,
            version: None,
            name: name.into(),
        }
    }

    /// Sets the IRI relevant to the agent
    pub fn with_uri(mut self, uri: impl Into<Cow<'a, str>>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Sets the version of the agent
    pub fn with_version(mut self, version: impl Into<Cow<'a, str>>) -> Self {
        self.version = Some(version.into());
        self
    }
}

/// A person, corporation, or similar entity, such as an author
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct Person<'a> {
    /// Human-readable name of the person
    pub name: Cow<'a, str>,
    /// IRI associated with the person
    pub uri: Option<Cow<'a, str>>,
    /// Email address associated with the person
    pub email: Option<Cow<'a, str>>,
}

impl<'a> Person<'a> {
    /// Creates a person with the given name
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: name.into(),
            uri: None,
            email: None,
        }
    }

    /// Sets the IRI associated with the person
    pub fn with_uri(mut self, uri: impl Into<Cow<'a, str>>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Sets the email address associated with the person
    pub fn with_email(mut self, email: impl Into<Cow<'a, str>>) -> Self {
        self.email = Some(email.into());
        self
    }
}

impl ToXml for Person<'_> {
    fn serialize<W: fmt::Write + ?Sized>(
        &self,
        field: Option<Id<'_>>,
        serializer: &mut Serializer<'_, W>,
    ) -> Result<(), Error> {
        let Some(field) = field else {
            return Err(Error::UnexpectedState("person requires an element name"));
        };

        let Self { name, uri, email } = self;
        let element = serializer.write_start(field.name, field.ns, None::<Context<0>>)?;
        serializer.end_start()?;
        name.serialize(
            Some(Id {
                ns: ATOM,
                name: "name",
            }),
            serializer,
        )?;
        uri.serialize(
            Some(Id {
                ns: ATOM,
                name: "uri",
            }),
            serializer,
        )?;
        email.serialize(
            Some(Id {
                ns: ATOM,
                name: "email",
            }),
            serializer,
        )?;
        serializer.write_close(element)
    }
}

impl<'xml: 'a, 'a> FromXml<'xml> for Person<'a> {
    #[inline]
    fn matches(id: Id<'_>, field: Option<Id<'_>>) -> bool {
        match field {
            Some(field) => id == field,
            None => false,
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut Deserializer<'cx, 'xml>,
    ) -> Result<(), Error> {
        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        let mut name = None;
        let mut uri = None;
        let mut email = None;
        while let Some(node) = deserializer.next() {
            let element = match node? {
                Node::Open(element) => element,
                Node::Attribute(_) | Node::Text(_) => continue,
                node @ (Node::AttributeValue(_) | Node::Close { .. }) => {
                    return Err(Error::UnexpectedNode(format!("{node:?} in {field}")));
                }
            };

            let id = deserializer.element_id(&element)?;
            let mut nested = deserializer.nested(element);
            let (value, path) = match id {
                Id {
                    ns: ATOM,
                    name: "name",
                } => (&mut name, "Person::name"),
                Id {
                    ns: ATOM,
                    name: "uri",
                } => (&mut uri, "Person::uri"),
                Id {
                    ns: ATOM,
                    name: "email",
                } => (&mut email, "Person::email"),
                _ => {
                    nested.ignore()?;
                    continue;
                }
            };

            if value.is_some() {
                return Err(Error::DuplicateValue(path));
            }

            *value = Some(nested.take_str()?.unwrap_or_default());
            nested.ignore()?;
        }

        let Some(name) = name else {
            return Err(Error::MissingValue("Person::name"));
        };

        *into = Some(Self { name, uri, email });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: Kind = Kind::Element;
}

const XML_DECLARATION: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n";
const ATOM: &str = "http://www.w3.org/2005/Atom";
const XML: &str = "http://www.w3.org/XML/1998/namespace";
