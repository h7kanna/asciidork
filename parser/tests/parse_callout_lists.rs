use smallvec::smallvec as svec;

use asciidork_ast::prelude::*;
use asciidork_ast::short::block::*;
use asciidork_parser::Parser;
use test_utils::{assert_eq, *};

#[test]
fn test_parse_callout_list() {
  let input = adoc! {r#"
    ----
    int x; <1>
    int y; <2>
    ----
    <1> foo
    <2> bar
  "#};
  assert_callout_list(
    input,
    Context::CalloutList,
    &[
      ListItem {
        marker: ListMarker::Callout(Some(1)),
        marker_src: src("<1>", 32..35),
        principle: just("foo", 36..39),
        type_meta: ListItemTypeMeta::Callout(svec![Callout::new(0, 0, 1)]),
        ..empty_list_item()
      },
      ListItem {
        marker: ListMarker::Callout(Some(2)),
        marker_src: src("<2>", 40..43),
        principle: just("bar", 44..47),
        type_meta: ListItemTypeMeta::Callout(svec![Callout::new(0, 1, 2)]),
        ..empty_list_item()
      },
    ],
  );
}

#[test]
fn test_callout_list_double_ref() {
  let input = adoc! {r#"
    [source, ruby]
    ----
    require 'asciidoctor' # <1>
    doc = Asciidoctor::Document.new('Hello, World!') # <2>
    puts doc.convert # <2>
    ----
    <1> Import the library
    <2> Where the magic happens
  "#};
  assert_callout_list(
    input,
    Context::CalloutList,
    &[
      ListItem {
        marker: ListMarker::Callout(Some(1)),
        marker_src: src("<1>", 131..134),
        principle: just("Import the library", 135..153),
        type_meta: ListItemTypeMeta::Callout(svec![Callout::new(0, 0, 1)]),
        ..empty_list_item()
      },
      ListItem {
        marker: ListMarker::Callout(Some(2)),
        marker_src: src("<2>", 154..157),
        principle: just("Where the magic happens", 158..181),
        type_meta: ListItemTypeMeta::Callout(svec![Callout::new(0, 1, 2), Callout::new(0, 2, 2),]),
        ..empty_list_item()
      },
    ],
  );
}

#[test]
fn test_nonsequential_callouts() {
  let input = adoc! {r#"
    [source,ruby]
    ----
    require 'asciidoctor' # <2>
    doc = Asciidoctor::Document.new('Hello, World!') # <3>
    puts doc.convert # <1>
    ----
    <1> Describe the first line
    <2> Describe the second line
    <3> Describe the third line
  "#};
  assert_callout_list(
    input,
    Context::CalloutList,
    &[
      ListItem {
        marker: ListMarker::Callout(Some(1)),
        marker_src: src("<1>", 130..133),
        principle: just("Describe the first line", 134..157),
        type_meta: ListItemTypeMeta::Callout(svec![Callout::new(0, 2, 1)]),
        ..empty_list_item()
      },
      ListItem {
        marker: ListMarker::Callout(Some(2)),
        marker_src: src("<2>", 158..161),
        principle: just("Describe the second line", 162..186),
        type_meta: ListItemTypeMeta::Callout(svec![Callout::new(0, 0, 2)]),
        ..empty_list_item()
      },
      ListItem {
        marker: ListMarker::Callout(Some(3)),
        marker_src: src("<3>", 187..190),
        principle: just("Describe the third line", 191..214),
        type_meta: ListItemTypeMeta::Callout(svec![Callout::new(0, 1, 3)]),
        ..empty_list_item()
      },
    ],
  );
}

#[test]
fn test_two_listing_blocks_one_callout_list() {
  let input = adoc! {r#"
    .Import library
    [source, ruby]
    ----
    require 'asciidoctor' # <1>
    ----

    .Use library
    [source, ruby]
    ----
    doc = Asciidoctor::Document.new('Hello, World!') # <2>
    puts doc.convert # <3>
    ----

    <1> Describe the first line
    <2> Describe the second line
    <3> Describe the third line
  "#};
  assert_callout_list(
    input,
    Context::CalloutList,
    &[
      ListItem {
        marker: ListMarker::Callout(Some(1)),
        marker_src: src("<1>", 187..190),
        principle: just("Describe the first line", 191..214),
        type_meta: ListItemTypeMeta::Callout(svec![Callout::new(0, 0, 1)]),
        ..empty_list_item()
      },
      ListItem {
        marker: ListMarker::Callout(Some(2)),
        marker_src: src("<2>", 215..218),
        principle: just("Describe the second line", 219..243),
        type_meta: ListItemTypeMeta::Callout(svec![Callout::new(0, 1, 2)]),
        ..empty_list_item()
      },
      ListItem {
        marker: ListMarker::Callout(Some(3)),
        marker_src: src("<3>", 244..247),
        principle: just("Describe the third line", 248..271),
        type_meta: ListItemTypeMeta::Callout(svec![Callout::new(0, 2, 3)]),
        ..empty_list_item()
      },
    ],
  );
}

// helpers

fn assert_callout_list(
  input: &'static str,
  expected_context: Context,
  expected_items: &[ListItem],
) {
  let mut blocks = parse_blocks!(input);
  // dbg!(&blocks);
  let (context, items, ..) =
    list_block_data(blocks.pop().unwrap()).expect("expected list block data");
  assert_eq!(context, expected_context, from: input);
  assert_eq!(items, expected_items, from: input);
}
