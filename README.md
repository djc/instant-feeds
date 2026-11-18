# instant-feeds: generate and parse Atom feeds

[![Documentation](https://docs.rs/instant-feeds/badge.svg)](https://docs.rs/instant-feeds)
[![Crates.io](https://img.shields.io/crates/v/instant-feeds.svg)](https://crates.io/crates/instant-feeds)
[![Build status](https://github.com/djc/instant-feeds/workflows/CI/badge.svg)](https://github.com/djc/instant-feeds/actions?query=workflow%3ACI)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)

instant-feeds maps the [Atom Syndication Format](https://www.rfc-editor.org/rfc/rfc4287) to
plain Rust types. It is built on [instant-xml](https://github.com/djc/instant-xml) for
namespace-aware XML handling and [jiff](https://github.com/BurntSushi/jiff) for timestamps.

## Features

* Simple API that allows borrowing field data
* Parsing respects XML namespaces, accepts a prefixed Atom namespace, and skips extension elements
* Minimum supported Rust version is 1.85

## Limitations

instant-feeds is early in its lifecycle, so expect semver-incompatible releases as the API
grows. Currently:

* Only Atom is supported, not RSS
* Text and content are limited to plain text and HTML: parsing XHTML or other media types
  returns an error, and out-of-line content (`src`) is not supported
* `atom:source` and `xml:base` are not supported, and `xml:lang` is only supported on text
* Extension elements are ignored when parsing and cannot be written

All feedback welcome!
