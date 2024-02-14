use crate::internal::*;

#[derive(Debug, Clone)]
pub struct ContiguousLines<'bmp, 'src> {
  // NB: lines kept in reverse, as there is no VeqDeque in bumpalo
  // and we almost always want to consume from the front, so fake it
  reversed_lines: BumpVec<'bmp, Line<'bmp, 'src>>,
}

impl<'bmp, 'src> ContiguousLines<'bmp, 'src> {
  pub fn new(mut lines: BumpVec<'bmp, Line<'bmp, 'src>>) -> Self {
    lines.reverse();
    ContiguousLines { reversed_lines: lines }
  }

  pub fn current(&self) -> Option<&Line<'bmp, 'src>> {
    self.reversed_lines.last()
  }

  pub fn current_mut(&mut self) -> Option<&mut Line<'bmp, 'src>> {
    self.reversed_lines.last_mut()
  }

  pub fn first(&self) -> Option<&Line<'bmp, 'src>> {
    self.reversed_lines.last()
  }

  pub fn last(&self) -> Option<&Line<'bmp, 'src>> {
    self.reversed_lines.first()
  }

  pub fn last_mut(&mut self) -> Option<&mut Line<'bmp, 'src>> {
    self.reversed_lines.first_mut()
  }

  pub fn remove_last_unchecked(&mut self) -> Line<'bmp, 'src> {
    self.reversed_lines.remove(0)
  }

  pub fn nth(&self, n: usize) -> Option<&Line<'bmp, 'src>> {
    self.reversed_lines.get(self.reversed_lines.len() - n - 1)
  }

  pub fn current_token(&self) -> Option<&Token<'src>> {
    self.current().and_then(|line| line.current_token())
  }

  pub fn is_empty(&self) -> bool {
    self.reversed_lines.is_empty()
  }

  pub fn consume_current(&mut self) -> Option<Line<'bmp, 'src>> {
    self.reversed_lines.pop()
  }

  pub fn consume_current_token(&mut self) -> Option<Token<'src>> {
    self
      .consume_current()
      .and_then(|mut line| line.consume_current())
  }

  pub fn restore_if_nonempty(&mut self, line: Line<'bmp, 'src>) {
    if !line.is_empty() {
      self.reversed_lines.push(line);
    }
  }

  pub fn retain(&mut self, f: impl FnMut(&Line<'bmp, 'src>) -> bool) {
    self.reversed_lines.retain(f);
  }

  pub fn any(&self, f: impl FnMut(&Line<'bmp, 'src>) -> bool) -> bool {
    self.reversed_lines.iter().any(f)
  }

  pub fn contains_seq(&self, kinds: &[TokenKind]) -> bool {
    self
      .reversed_lines
      .iter()
      .any(|line| line.contains_seq(kinds))
  }

  pub fn terminates_constrained(&self, stop_tokens: &[TokenKind]) -> bool {
    self
      .reversed_lines
      .iter()
      .any(|line| line.terminates_constrained(stop_tokens))
  }

  pub fn is_block_macro(&self) -> bool {
    self.reversed_lines.len() == 1 && self.current().unwrap().is_block_macro()
  }

  pub fn current_starts_with(&self, kind: TokenKind) -> bool {
    match self.current() {
      Some(line) => line
        .current_token()
        .map(|token| token.is(kind))
        .unwrap_or(false),
      None => false,
    }
  }

  pub fn loc(&self) -> Option<SourceLocation> {
    if let Some(line) = self.reversed_lines.last() {
      line.loc()
    } else {
      None
    }
  }

  pub fn is_quoted_paragraph(&self) -> bool {
    use TokenKind::*;
    if self.reversed_lines.len() < 2 {
      return false;
    }
    let last_line = self.last().unwrap();
    if !last_line.starts_with_seq(&[Dashes, Whitespace])
      || !last_line.src.starts_with("-- ")
      || last_line.num_tokens() < 3
    {
      return false;
    }
    let first_line = self.current().unwrap();
    if !first_line.starts(DoubleQuote) {
      return false;
    }
    let penult = self.nth(self.reversed_lines.len() - 2).unwrap();
    penult.ends(DoubleQuote)
  }

  pub fn starts_list(&self) -> bool {
    self
      .first()
      .map(|line| line.starts_list_item())
      .unwrap_or(false)
  }

  pub fn starts(&self, kind: TokenKind) -> bool {
    self.first().map(|line| line.starts(kind)).unwrap_or(false)
  }

  pub fn discard_leading_comment_lines(&mut self) {
    while self
      .current()
      .map(|line| line.starts(TokenKind::CommentLine))
      .unwrap_or(false)
    {
      self.consume_current();
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_is_quoted_paragraph() {
    let cases = vec![
      ("\"foo bar\nso baz\"\n-- me", true),
      ("foo bar\nso baz\"\n-- me", false),
      ("\"foo bar\nso baz\n-- me", false),
      ("\"foo bar\nso baz\"\n-- ", false),
      ("\"foo bar\nso baz\"\nme -- too", false),
    ];
    let bump = &Bump::new();
    for (input, expected) in cases {
      let mut parser = Parser::new(bump, input);
      let lines = parser.read_lines().unwrap();
      assert_eq!(
        lines.is_quoted_paragraph(),
        expected,
        "input was:\n\n```\n{}\n```\n",
        input
      );
    }
  }
}
