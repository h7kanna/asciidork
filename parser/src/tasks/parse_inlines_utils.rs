use lazy_static::lazy_static;
use regex::Regex;

use crate::internal::*;

impl Substitutions {
  /// https://docs.asciidoctor.org/asciidoc/latest/pass/pass-macro/#custom-substitutions
  pub fn from_pass_macro_target(target: &Option<SourceString>) -> Self {
    let Some(target) = target else {
      return Substitutions::none();
    };
    let mut subs = Self::none();
    target.split(',').for_each(|value| match value {
      "c" => subs.special_chars = true,
      "a" => subs.attr_refs = true,
      "r" => subs.char_replacement = true,
      "m" => subs.macros = true,
      "p" => subs.post_replacement = true,
      "q" => subs.inline_formatting = true,
      "v" => subs.special_chars = true, // verbatim =  only special chars
      "n" => subs = Substitutions::all(), // normal = all
      _ => {}
    });
    subs
  }
}

pub fn extend(loc: &mut SourceLocation, nodes: &[InlineNode<'_>], adding: usize) {
  loc.end = nodes.last().map(|node| node.loc.end).unwrap_or(loc.end) + adding;
}

pub fn starts_constrained(
  stop_tokens: &[TokenKind],
  token: &Token,
  line: &Line,
  lines: &mut ContiguousLines,
) -> bool {
  debug_assert!(!stop_tokens.is_empty());
  token.is(*stop_tokens.last().expect("non-empty stop tokens"))
    && (line.terminates_constrained(stop_tokens) || lines.terminates_constrained(stop_tokens))
}

pub fn starts_unconstrained(
  kind: TokenKind,
  token: &Token,
  line: &Line,
  lines: &ContiguousLines,
) -> bool {
  token.is(kind) && line.current_is(kind) && contains_seq(&[kind; 2], line, lines)
}

pub fn contains_seq(seq: &[TokenKind], line: &Line, lines: &ContiguousLines) -> bool {
  line.contains_seq(seq) || lines.contains_seq(seq)
}

pub fn node(content: Inline, loc: SourceLocation) -> InlineNode {
  InlineNode::new(content, loc)
}

pub fn finish_macro<'bmp>(
  line: &Line<'bmp, '_>,
  loc: &mut SourceLocation,
  line_end: SourceLocation,
  text: &mut CollectText<'bmp>,
) {
  if let Some(cur_location) = line.loc() {
    loc.extend(cur_location);
    text.loc = loc.clamp_end();
    loc.end -= 1; // parsing attr list moves us one past end of macro
  } else {
    loc.extend(line_end);
    text.loc = loc.clamp_end();
  }
}

lazy_static! {
  pub static ref EMAIL_RE: Regex = Regex::new(
    r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})"
  )
  .unwrap();
}
