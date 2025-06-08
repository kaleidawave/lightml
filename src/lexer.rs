pub struct Lexer<'a> {
    on: &'a str,
    head: u32,
}

impl<'a> Lexer<'a> {
    #[must_use]
    pub fn new(on: &'a str) -> Self {
        Self { on, head: 0 }
    }

    pub fn is_operator_advance(&mut self, operator: &str) -> bool {
        self.skip();
        if self.current().starts_with(operator) {
            self.head += operator.len() as u32;
            true
        } else {
            false
        }
    }

    pub fn starts_with_string_delimeter(&mut self) -> bool {
        self.skip();
        self.current().starts_with(['"', '\''])
    }

    pub fn starts_with_no_advance(&mut self, slice: &str) -> bool {
        self.current().starts_with(slice)
    }

    /// # Errors
    ///
    /// Will return `Err` if reached end of parse buffer without finding `slice`
    pub fn parse_until(&mut self, slice: &str, advance: bool) -> Result<&'a str, u32> {
        let current = self.current();
        let start = self.head;

        // None = None, Some(true) = ```, Some(false) = `
        #[cfg(feature = "markdown_code_blocks_in_html")]
        let (mut in_code_block, mut escaped) = (None, false);

        let mut chars = current.char_indices();

        #[allow(unused, clippy::while_let_on_iterator)]
        while let Some((idx, _chr)) = chars.next() {
            #[cfg(feature = "markdown_code_blocks_in_html")]
            {
                // dbg!(idx, &current[idx..], escaped, &in_code_block);

                if escaped {
                    escaped = false;
                    continue;
                } else {
                    escaped = '\\' == _chr;
                }

                if let '`' = _chr {
                    if let Some(false) = in_code_block {
                        in_code_block = None;
                        continue;
                    }

                    let in_block = current[idx..].starts_with("```");
                    if in_block {
                        "``".chars().for_each(|_| {
                            chars.next();
                        });

                        if let Some(true) = in_code_block {
                            in_code_block = None;
                        } else {
                            in_code_block = Some(true);
                        }
                    } else if let None = in_code_block {
                        in_code_block = Some(false);
                    }
                }

                if in_code_block.is_some() {
                    continue;
                }
            }

            if current[idx..].starts_with(slice) {
                self.head += idx as u32;
                if advance {
                    self.head += slice.len() as u32;
                }
                return Ok(&current[..idx]);
            }
        }

        Err(start)
    }

    /// Above modified to check `then`
    /// # Errors
    ///
    /// Will return `Err` if reached end of parse buffer without finding `slice`
    pub fn parse_until_postfix(&mut self, slice: &str, then: &str) -> Result<&'a str, u32> {
        let current = self.current();
        let start = self.head;
        for (idx, _chr) in current.char_indices() {
            if current[idx..].starts_with(slice) && current[(idx + slice.len())..].starts_with(then)
            {
                self.head += idx as u32 + slice.len() as u32;
                return Ok(&current[..idx]);
            }
        }
        Err(start)
    }

    #[must_use]
    pub fn current(&self) -> &'a str {
        self.current_with_offset(0)
    }

    #[must_use]
    pub fn consumed(&self) -> u32 {
        self.head
    }

    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.head as usize == self.on.len()
    }

    #[must_use]
    pub fn current_with_offset(&self, offset: u32) -> &'a str {
        unsafe { self.on.get_unchecked((self.head + offset) as usize..) }
        // &self.on[self.head as usize..]
    }

    /// # Errors
    ///
    /// Will return `Err` if reached end of parse buffer without finding string delimeter
    pub fn parse_string_literal(&mut self) -> Result<&'a str, u32> {
        let mut chars = self.current().chars();
        let start = self.head;

        let delimeter = if let Some(chr) = chars.next() {
            if let '"' | '\'' = chr {
                chr
            } else {
                return Err(start);
            }
        } else {
            return Err(start);
        };

        let mut consumed: usize = 0;
        let mut escaped = false;
        for chr in chars {
            consumed += chr.len_utf8();

            // TODO temp
            if escaped {
                escaped = false;
                continue;
            }

            if chr == delimeter {
                let slice = &self.on[(self.head as usize + 1)..(self.head as usize + consumed)];
                self.head += consumed as u32 + 1;
                return Ok(slice);
            }
            escaped = matches!(chr, '\\');
        }
        Err(start)
    }

    /// # Errors
    ///
    /// Will return `Err` if reached end of parse buffer without finding identifier terminator
    pub fn parse_identifier(&mut self, position: &str) -> Result<&'a str, u32> {
        let current = self.current();
        let chars = current.char_indices();
        let start = self.head;

        for (idx, chr) in chars {
            // WIP
            let is_part_of_value = if let "Attribute key" = position {
                // See https://html.spec.whatwg.org/multipage/syntax.html#attributes-2
                // TODO non-characters
                !matches!(chr, ' ' | '\'' | '"' | '>' | '/' | '=')
            } else if let "Attribute value" = position {
                // See https://html.spec.whatwg.org/multipage/syntax.html#attributes-2
                !(chr.is_ascii_whitespace() || matches!(chr, '`' | '"' | '\'' | '>' | '<' | '='))
            } else {
                chr.is_alphanumeric() || matches!(chr, '-' | '_' | '$' | ':')
            };

            if !is_part_of_value {
                let slice = &current[..idx];
                self.head += idx as u32;
                return if let 0 = idx { Err(start) } else { Ok(slice) };
            }
        }
        Err(start)
    }

    #[must_use]
    pub fn parse_opening_tag_no_advance(&self) -> Option<&'a str> {
        let current = self.current().trim_start();
        if let Some(rest) = current.strip_prefix('<') {
            for (idx, chr) in rest.char_indices() {
                if let '>' | ' ' = chr {
                    return Some(&rest[..idx]);
                }
            }
            None
        } else {
            None
        }
    }

    #[must_use]
    pub fn parse_closing_tag_no_advance(&self) -> Option<&'a str> {
        let current = self.current().trim_start();
        if let Some(rest) = current.strip_prefix("</") {
            for (idx, chr) in rest.char_indices() {
                if let '>' | ' ' = chr {
                    return Some(&rest[..idx]);
                }
            }
            None
        } else {
            None
        }
    }

    pub fn skip(&mut self) {
        let chars = self.current().char_indices();
        for (idx, chr) in chars {
            if !chr.is_whitespace() {
                self.head += idx as u32;
                break;
            }
        }
    }

    pub fn parse_whitespace(&mut self) -> Option<&'a str> {
        let current = self.current();
        let chars = current.char_indices();
        for (idx, chr) in chars {
            if !chr.is_whitespace() {
                self.head += idx as u32;
                return if idx == 0 {
                    None
                } else {
                    Some(&current[..idx])
                };
            }
        }
        None
    }

    /// # Errors
    ///
    /// Will return `Err` if chr is not at head of parse buffer
    pub fn expect(&mut self, chr: char) -> Result<(), u32> {
        self.skip();
        if self.current().starts_with(chr) {
            self.head += chr.len_utf8() as u32;
            Ok(())
        } else {
            Err(self.head)
        }
    }

    pub fn advance(&mut self, distance: u32) {
        self.head += distance;
    }

    #[must_use]
    pub fn after_comments(&self) -> &str {
        todo!()
    }

    pub fn parse_one_or_more_whitespace(&mut self) -> bool {
        if self
            .current()
            .starts_with(|c: char| c.is_ascii_whitespace())
        {
            self.skip();
            true
        } else {
            false
        }
    }

    pub fn starts_with_ascii_case_ignore(&mut self, expected: &str) -> bool {
        let current = self.current().get(..expected.len());
        if current.is_some_and(|slice| slice.eq_ignore_ascii_case(expected)) {
            self.advance(expected.len() as u32);
            true
        } else {
            false
        }
    }
}
