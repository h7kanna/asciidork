use super::source_lexer::SourceLexer;
use crate::internal::*;

#[derive(Debug)]
pub struct RootLexer<'arena> {
  pub bump: &'arena Bump,
  idx: u16,
  next_idx: Option<u16>,
  source_stack: Vec<u16>,
  sources: BumpVec<'arena, SourceLexer<'arena>>,
  tmp_buf: Option<(SourceLexer<'arena>, BufLoc)>,
}

#[derive(Debug)]
pub enum BufLoc {
  Repeat(SourceLocation),
  Offset(u32),
}

impl<'arena> RootLexer<'arena> {
  pub fn new(src: BumpVec<'arena, u8>, file: SourceFile, bump: &'arena Bump) -> Self {
    Self {
      bump,
      idx: 0,
      next_idx: None,
      source_stack: Vec::new(),
      sources: bvec![in bump; SourceLexer::new(src, file, 0, None, bump)],
      tmp_buf: None,
    }
  }

  pub fn from_str(bump: &'arena Bump, file: SourceFile, src: &str) -> Self {
    Self {
      bump,
      idx: 0,
      next_idx: None,
      source_stack: Vec::new(),
      sources: bvec![in bump; SourceLexer::from_str(src, file, bump)],
      tmp_buf: None,
    }
  }

  pub fn from_byte_slice(bytes: &[u8], file: SourceFile, bump: &'arena Bump) -> Self {
    Self {
      bump,
      idx: 0,
      next_idx: None,
      source_stack: Vec::new(),
      sources: bvec![in bump; SourceLexer::from_byte_slice(bytes, file, bump)],
      tmp_buf: None,
    }
  }

  pub fn push_source(
    &mut self,
    src_file: SourceFile,
    leveloffset: i8,
    max_include_depth: Option<u16>,
    mut src_bytes: BumpVec<'arena, u8>,
  ) {
    // match asciidoctor - its include processor returns an array of lines
    // so even if the source has no trailing newline, it's inserted as a full line
    // NB: the include resolver has taken care of reserving space for the newline
    if src_bytes.last() != Some(&b'\n') {
      src_bytes.push(b'\n');
    }
    self.sources.push(SourceLexer::new(
      src_bytes,
      src_file,
      leveloffset,
      max_include_depth,
      self.bump,
    ));
    let next_idx = self.sources.len() as u16 - 1;
    self.next_idx = Some(next_idx);
  }

  pub fn set_tmp_buf(&mut self, buf: &str, loc: BufLoc) {
    self.tmp_buf = Some((SourceLexer::from_str(buf, SourceFile::Tmp, self.bump), loc));
  }

  pub fn adjust_offset(&mut self, offset_adjustment: u32) {
    self.sources[self.idx as usize].offset = offset_adjustment;
  }

  pub fn source_file(&self) -> &SourceFile {
    &self.sources[self.idx as usize].file
  }

  pub fn source_file_at(&self, idx: u16) -> &SourceFile {
    &self.sources[idx as usize].file
  }

  pub const fn source_is_primary(&self) -> bool {
    self.idx == 0
  }

  pub const fn source_idx(&self) -> u16 {
    self.idx
  }

  pub fn source_idx_of_xref_target(&self, xref_target: &str) -> Option<u16> {
    self
      .sources
      .iter()
      .map(|src| &src.file)
      .enumerate()
      .find(|(_, file)| file.matches_xref_target(xref_target))
      .map(|(i, _)| i as u16)
  }

  pub fn leveloffset(&self, idx: u16) -> i8 {
    self.sources[idx as usize].leveloffset
  }

  pub fn peek(&self) -> Option<u8> {
    // case: we're in the middle of a temporary buffer
    if let Some((tmp_buf, _)) = &self.tmp_buf {
      return tmp_buf.peek();
    }
    // case: we're about to switch to a new source
    if let Some(next_idx) = self.next_idx {
      return self.sources[next_idx as usize].peek();
    }
    // case: normal path: we're peeking at the current source, and have bytes
    if let Some(c) = self.sources[self.idx as usize].peek() {
      return Some(c);
    }
    // case: we're out of bytes, check if we're returning to a previous source
    if let Some(next_idx) = self.source_stack.last() {
      return self.sources[*next_idx as usize].peek();
    }
    // case: nothing left in any source, document EOF
    None
  }

  pub fn skip_byte(&mut self) {
    if let Some((tmp_lexer, _)) = &mut self.tmp_buf {
      tmp_lexer.pos += 1;
      if tmp_lexer.is_eof() {
        self.tmp_buf = None;
      }
      return;
    }
    self.maybe_advance_source();
    if self.sources[self.idx as usize].peek().is_some() {
      self.sources[self.idx as usize].pos += 1;
    } else if let Some(prev_idx) = self.source_stack.pop() {
      self.idx = prev_idx;
      self.skip_byte();
    }
  }

  fn maybe_advance_source(&mut self) {
    if let Some(next_idx) = self.next_idx.take() {
      self.source_stack.push(self.idx);
      self.idx = next_idx;
    }
  }

  pub fn consume_empty_lines(&mut self) {
    self.maybe_advance_source();
    self.sources[self.idx as usize].consume_empty_lines();
    if self.sources[self.idx as usize].is_eof() {
      if let Some(prev_idx) = self.source_stack.pop() {
        self.idx = prev_idx;
        self.consume_empty_lines();
      }
    }
  }

  pub fn raw_lines(&'arena self) -> impl Iterator<Item = &'arena str> {
    self.sources[self.idx as usize].raw_lines()
  }

  pub fn loc(&self) -> SourceLocation {
    SourceLocation::from(self.sources[self.idx as usize].pos)
  }

  pub fn at_delimiter_line(&self) -> Option<(u32, u8)> {
    self.sources[self.idx as usize].at_delimiter_line()
  }

  pub fn is_eof(&self) -> bool {
    self.peek().is_none()
  }

  pub fn peek_is(&self, c: u8) -> bool {
    self.peek() == Some(c)
  }

  pub fn line_of(&self, location: u32) -> BumpString<'arena> {
    self.sources[self.idx as usize].line_of(location)
  }

  pub fn line_number(&self, location: u32) -> u32 {
    let (line_number, _) = self.line_number_with_offset(location);
    line_number
  }

  pub fn line_number_with_offset(&self, location: u32) -> (u32, u32) {
    self.sources[self.idx as usize].line_number_with_offset(location)
  }

  pub fn next_token(&mut self) -> Token<'arena> {
    if let Some((ref mut buf_lexer, ref buf_loc)) = self.tmp_buf {
      if let Some(mut token) = buf_lexer.next_token() {
        match buf_loc {
          BufLoc::Repeat(loc) => token.loc = *loc,
          BufLoc::Offset(offset) => token.loc = token.loc.offset(*offset),
        }
        if buf_lexer.is_eof() {
          self.tmp_buf = None
        }
        return token;
      }
    }
    self.maybe_advance_source();
    match self.sources[self.idx as usize].next_token() {
      Some(mut token) => {
        token.loc.include_depth = self.idx;
        token
      }
      None => {
        let Some(prev_idx) = self.source_stack.pop() else {
          return self.token(TokenKind::Eof, "", self.loc());
        };
        self.idx = prev_idx;
        self.next_token()
      }
    }
  }

  pub fn truncate(&mut self) {
    self.sources[self.idx as usize].truncate();
  }

  pub const fn include_depth(&self) -> u16 {
    self.idx
  }

  pub fn max_include_depth(&self) -> Option<(u16, u16)> {
    self
      .sources
      .iter()
      .enumerate()
      .fold(None, |current, (i, src)| {
        src.max_include_depth.map(|d| (d, i as u16)).or(current)
      })
  }
}

impl<'arena> HasArena<'arena> for RootLexer<'arena> {
  fn bump(&self) -> &'arena Bump {
    self.bump
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::token::TokenKind;
  use crate::variants::token::*;
  use ast::SourceLocation;
  use test_utils::*;

  macro_rules! assert_token_cases {
    ($cases:expr) => {{
      for (input, expected) in $cases {
        let mut lexer = test_lexer!(input);
        let mut index = 0;
        for (token_type, lexeme) in expected {
          let start = index;
          let end = start + lexeme.len();
          let expected_token = Token {
            kind: token_type,
            loc: SourceLocation::from(start..end),
            lexeme: bstr!(lexeme),
          };
          let actual = lexer.next_token();
          assert_eq!(actual, expected_token);
          index = end;
        }
        let next = lexer.next_token();
        assert_eq!(next.kind, Eof);
      }
    }};
  }

  macro_rules! refute_produces_token {
    ($kind:ident, $cases:expr) => {{
      for input in $cases {
        let mut lexer = test_lexer!(input);
        loop {
          match lexer.next_token().kind {
            Eof => {
              assert_eq!(true, true);
              break;
            }
            $kind => panic!("unexpected TokenKind::{:?} in input `{input}`", $kind),
            _ => {}
          }
        }
      }
    }};
  }

  #[test]
  fn test_tokens() {
    let cases = vec![
      ("|===", vec![(Pipe, "|"), (EqualSigns, "===")]),
      ("////", vec![(DelimiterLine, "////")]),
      ("<.>", vec![(CalloutNumber, "<.>")]),
      ("<1>", vec![(CalloutNumber, "<1>")]),
      ("<255>", vec![(CalloutNumber, "<255>")]),
      ("<!--.-->", vec![(CalloutNumber, "<!--.-->")]),
      ("<!--2-->", vec![(CalloutNumber, "<!--2-->")]),
      ("<!--255-->", vec![(CalloutNumber, "<!--255-->")]),
      (
        "<..>",
        vec![(LessThan, "<"), (Dots, ".."), (GreaterThan, ">")],
      ),
      (
        "<1.1>",
        vec![
          (LessThan, "<"),
          (Digits, "1"),
          (Dots, "."),
          (Digits, "1"),
          (GreaterThan, ">"),
        ],
      ),
      (
        "<!--1x-->",
        vec![
          (LessThan, "<"),
          (Bang, "!"),
          (Dashes, "--"),
          (Digits, "1"),
          (Word, "x"),
          (Dashes, "--"),
          (GreaterThan, ">"),
        ],
      ),
      (
        "<x>",
        vec![(LessThan, "<"), (Word, "x"), (GreaterThan, ">")],
      ),
      ("{}", vec![(OpenBrace, "{"), (CloseBrace, "}")]),
      ("  ", vec![(Whitespace, "  ")]),
      (".", vec![(Dots, ".")]),
      ("..", vec![(Dots, "..")]),
      ("1", vec![(Digits, "1")]),
      ("12345", vec![(Digits, "12345")]),
      ("-", vec![(Dashes, "-")]),
      ("---", vec![(Dashes, "---")]),
      (
        "---- foo",
        vec![(Dashes, "----"), (Whitespace, " "), (Word, "foo")],
      ),
      ("-----", vec![(Dashes, "-----")]),
      ("--", vec![(DelimiterLine, "--")]),
      ("--\n", vec![(DelimiterLine, "--"), (Newline, "\n")]),
      ("****", vec![(DelimiterLine, "****")]),
      ("====", vec![(DelimiterLine, "====")]),
      ("____", vec![(DelimiterLine, "____")]),
      ("----", vec![(DelimiterLine, "----")]),
      ("....", vec![(DelimiterLine, "....")]),
      ("++++", vec![(DelimiterLine, "++++")]),
      (
        "****\nfoo",
        vec![(DelimiterLine, "****"), (Newline, "\n"), (Word, "foo")],
      ),
      (
        "foo****",
        vec![
          (Word, "foo"),
          (Star, "*"),
          (Star, "*"),
          (Star, "*"),
          (Star, "*"),
        ],
      ),
      ("foo@bar", vec![(Word, "foo@bar")]),
      (
        "foo@bar.com@",
        vec![(Word, "foo@bar"), (Dots, "."), (Word, "com@")],
      ),
      ("@foo@bar", vec![(Word, "@foo@bar")]),
      ("foo@", vec![(Word, "foo@")]),
      ("foo@.a", vec![(Word, "foo@"), (Dots, "."), (Word, "a")]),
      ("foo.bar", vec![(Word, "foo"), (Dots, "."), (Word, "bar")]),
      (
        "roflfootnote:",
        vec![(Word, "rofl"), (MacroName, "footnote:")],
      ),
      ("footnote:", vec![(MacroName, "footnote:")]),
      ("==", vec![(EqualSigns, "==")]),
      ("===", vec![(EqualSigns, "===")]),
      (
        "// foo",
        vec![(ForwardSlashes, "//"), (Whitespace, " "), (Word, "foo")],
      ),
      (
        "foo\n//-\nbar",
        vec![
          (Word, "foo"),
          (Newline, "\n"),
          (ForwardSlashes, "//"),
          (Dashes, "-"),
          (Newline, "\n"),
          (Word, "bar"),
        ],
      ),
      (
        "foo     ;", // whitespace is grouped
        vec![(Word, "foo"), (Whitespace, "     "), (SemiColon, ";")],
      ),
      (
        "foo=;",
        vec![(Word, "foo"), (EqualSigns, "="), (SemiColon, ";")],
      ),
      (
        "foo;=",
        vec![(Word, "foo"), (SemiColon, ";"), (EqualSigns, "=")],
      ),
      (
        "foo;image:&bar,lol^~_*.!`+[]#'\"\\%",
        vec![
          (Word, "foo"),
          (SemiColon, ";"),
          (MacroName, "image:"),
          (Ampersand, "&"),
          (Word, "bar"),
          (Comma, ","),
          (Word, "lol"),
          (Caret, "^"),
          (Tilde, "~"),
          (Underscore, "_"),
          (Star, "*"),
          (Dots, "."),
          (Bang, "!"),
          (Backtick, "`"),
          (Plus, "+"),
          (OpenBracket, "["),
          (CloseBracket, "]"),
          (Hash, "#"),
          (SingleQuote, "'"),
          (DoubleQuote, "\""),
          (Backslash, "\\"),
          (Percent, "%"),
        ],
      ),
      (
        "Foobar\n\n",
        vec![(Word, "Foobar"), (Newline, "\n"), (Newline, "\n")],
      ),
      (
        "== Title",
        vec![(EqualSigns, "=="), (Whitespace, " "), (Word, "Title")],
      ),
      (
        adoc! { "
          // ignored
          = Document Title
          Kismet R. Lee <kismet@asciidoctor.org>
          :description: The document's description.
          :sectanchors:
          :url-repo: https://my-git-repo.com

          The document body starts here.
        "},
        vec![
          (ForwardSlashes, "//"),
          (Whitespace, " "),
          (Word, "ignored"),
          (Newline, "\n"),
          (EqualSigns, "="),
          (Whitespace, " "),
          (Word, "Document"),
          (Whitespace, " "),
          (Word, "Title"),
          (Newline, "\n"),
          (Word, "Kismet"),
          (Whitespace, " "),
          (Word, "R"),
          (Dots, "."),
          (Whitespace, " "),
          (Word, "Lee"),
          (Whitespace, " "),
          (LessThan, "<"),
          (MaybeEmail, "kismet@asciidoctor.org"),
          (GreaterThan, ">"),
          (Newline, "\n"),
          (Colon, ":"),
          (Word, "description"),
          (Colon, ":"),
          (Whitespace, " "),
          (Word, "The"),
          (Whitespace, " "),
          (Word, "document"),
          (SingleQuote, "'"),
          (Word, "s"),
          (Whitespace, " "),
          (Word, "description"),
          (Dots, "."),
          (Newline, "\n"),
          (Colon, ":"),
          (Word, "sectanchors"),
          (Colon, ":"),
          (Newline, "\n"),
          (Colon, ":"),
          (Word, "url"),
          (Dashes, "-"),
          (Word, "repo"),
          (Colon, ":"),
          (Whitespace, " "),
          (UriScheme, "https://"),
          (Word, "my"),
          (Dashes, "-"),
          (Word, "git"),
          (Dashes, "-"),
          (Word, "repo"),
          (Dots, "."),
          (Word, "com"),
          (Newline, "\n"),
          (Newline, "\n"),
          (Word, "The"),
          (Whitespace, " "),
          (Word, "document"),
          (Whitespace, " "),
          (Word, "body"),
          (Whitespace, " "),
          (Word, "starts"),
          (Whitespace, " "),
          (Word, "here"),
          (Dots, "."),
          (Newline, "\n"),
        ],
      ),
    ];
    assert_token_cases!(cases);
  }

  #[test]
  fn test_entity_tokens() {
    assert_token_cases!([
      ("&amp;", vec![(Entity, "&amp;")]),
      ("&sect;", vec![(Entity, "&sect;")]),
      (
        "&CounterClockwiseContourIntegral;",
        vec![(Entity, "&CounterClockwiseContourIntegral;")]
      ),
      ("&#169;", vec![(Entity, "&#169;")]),
      ("&#10004;", vec![(Entity, "&#10004;")]),
      ("&#128512;", vec![(Entity, "&#128512;")]),
      ("&#x2022;", vec![(Entity, "&#x2022;")]),
      ("&#x1f600;", vec![(Entity, "&#x1f600;")]),
    ]);
    refute_produces_token!(
      Entity,
      ["&a;", "&foo", "&#;", "&#x;", "&#XFFF;", "&#x05T;", "&sect 1;"]
    );
  }

  #[test]
  fn test_accented_words() {
    assert_token_cases!([
      ("él", vec![(Word, "él")]),
      (
        "foo él",
        vec![(Word, "foo"), (Whitespace, " "), (Word, "él")],
      ),
      ("diplomático", vec![(Word, "diplomático")]),
    ]);
  }

  #[test]
  fn test_attr_refs() {
    assert_token_cases!([
      ("{foo}", vec![(AttrRef, "{foo}")]),
      ("{foo-bar}", vec![(AttrRef, "{foo-bar}")]),
      ("{foo123}", vec![(AttrRef, "{foo123}")]),
    ]);

    refute_produces_token!(
      AttrRef,
      [
        "\\{foo}",    // escaped
        "foo {}",     // must be one char long
        "foo {a\nb}", // newline
        "foo {hi@}",  // only a-z,A-Z,0-9,-,_ allowed
      ]
    );
  }

  #[test]
  fn test_uri_schemes() {
    assert_token_cases!([
      ("http://", vec![(UriScheme, "http://")]),
      ("https://", vec![(UriScheme, "https://")]),
      ("file:///", vec![(UriScheme, "file:///")]),
      (
        "(http://foo)",
        vec![
          (OpenParens, "("),
          (UriScheme, "http://"),
          (Word, "foo"),
          (CloseParens, ")")
        ]
      ),
    ]);

    refute_produces_token!(AttrRef, ["file://"]);
  }

  #[test]
  fn test_directives() {
    assert_token_cases!([
      (
        "include::foo",
        vec![(Directive, "include::"), (Word, "foo")],
      ),
      (
        // not valid, but should lex as Directive
        // parser will reject it as a match
        "include::not-include []",
        vec![
          (Directive, "include::"),
          (Word, "not"),
          (Dashes, "-"),
          (Word, "include"),
          (Whitespace, " "),
          (OpenBracket, "["),
          (CloseBracket, "]")
        ],
      )
    ]);

    refute_produces_token!(
      Directive,
      [
        "foo include::foo",        // not at start of line
        "include:: foo",           // space after ::
        "include:: not-include[]", // space after ::
        "include:: []",            // space after ::
        "include::[]",             // empty target
      ]
    );
  }

  #[test]
  fn test_term_delimiters() {
    let foo = (Word, "foo");
    let space = (Whitespace, " ");
    let cases = vec![
      ("foo:: foo", vec![foo, (TermDelimiter, "::"), space, foo]),
      ("foo::", vec![foo, (TermDelimiter, "::")]),
      ("foo;;", vec![foo, (TermDelimiter, ";;")]),
      ("foo:::", vec![foo, (TermDelimiter, ":::")]),
      ("foo::::", vec![foo, (TermDelimiter, "::::")]),
      // doesn't trip up on macros
      (
        "image:: foo",
        vec![(Word, "image"), (TermDelimiter, "::"), space, foo],
      ),
      (
        "xfootnote:: foo",
        vec![(Word, "xfootnote"), (TermDelimiter, "::"), space, foo],
      ),
      (
        "kbd::: foo",
        vec![(Word, "kbd"), (TermDelimiter, ":::"), space, foo],
      ),
      (
        "footnote:::: foo",
        vec![(Word, "footnote"), (TermDelimiter, "::::"), space, foo],
      ),
    ];
    assert_token_cases!(cases);

    refute_produces_token!(
      TermDelimiter,
      ["foo::foo", "foo;;;", "foo:::::", "foo:::::foo", ":: foo"]
    );
  }

  #[test]
  fn test_tokens_full() {
    let input = "&^foobar[//";
    let mut lexer = test_lexer!(input);
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Ampersand,
        loc: SourceLocation::new(0, 1),
        lexeme: bstr!("&"),
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Caret,
        loc: SourceLocation::new(1, 2),
        lexeme: bstr!("^"),
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Word,
        loc: SourceLocation::new(2, 8),
        lexeme: bstr!("foobar"),
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::OpenBracket,
        loc: SourceLocation::new(8, 9),
        lexeme: bstr!("["),
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::ForwardSlashes,
        loc: SourceLocation::new(9, 11),
        lexeme: bstr!("//"),
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Eof,
        loc: SourceLocation::new(12, 12),
        lexeme: bstr!(""),
      }
    );
  }

  #[test]
  fn test_line_of() {
    let lexer = test_lexer!("foo\nbar\n\nbaz\n");
    assert_eq!(lexer.line_of(1), "foo");
    assert_eq!(lexer.line_of(2), "foo");
    assert_eq!(lexer.line_of(3), "foo"); // newline

    assert_eq!(lexer.line_of(4), "bar");
    assert_eq!(lexer.line_of(7), "bar");
    assert_eq!(lexer.line_of(8), ""); // empty line
    assert_eq!(lexer.line_of(9), "baz");
  }

  #[test]
  fn test_line_num() {
    let input = adoc! {
      "= :
      foo

      ;"
    };
    let mut lexer = test_lexer!(input);
    assert_next_token_line(&mut lexer, 1, EqualSigns);
    assert_next_token_line(&mut lexer, 1, Whitespace);
    assert_next_token_line(&mut lexer, 1, Colon);
    assert_next_token_line(&mut lexer, 1, Newline);
    assert_next_token_line(&mut lexer, 2, Word);
    assert_next_token_line(&mut lexer, 2, Newline);
    assert_next_token_line(&mut lexer, 3, Newline);
  }

  fn assert_next_token_line(lexer: &mut RootLexer, line: u32, expected_kind: TokenKind) {
    let token = lexer.next_token();
    assert_eq!(token.kind, expected_kind);
    assert_eq!(lexer.line_number(token.loc.start), line);
  }

  #[test]
  fn test_consume_empty_lines() {
    let mut lexer = test_lexer!("\n\n\n\n\n");
    lexer.consume_empty_lines();
    assert!(lexer.is_eof());
  }
}
