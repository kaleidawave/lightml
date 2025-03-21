pub struct Lexer<'a> {
    on: &'a str,
    head: u32,
}

impl<'a> Lexer<'a> {
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

    pub fn starts_with_str(&mut self, slice: &str) -> bool {
        self.skip();
        self.current().starts_with(slice)
    }

    // TODO after method
    pub fn parse_until(&mut self, slice: &str, advance: bool) -> Result<(&'a str, ()), ()> {
        // TODO pass as argument
        let quote_escape = true;

        let current = self.current();
        let mut consumed: usize = 0;
        let mut in_code_block = false;

        for (idx, chr) in current.char_indices() {
            if quote_escape && '`' == chr {
                in_code_block = !in_code_block;
            }

            if !in_code_block && current[idx..].starts_with(slice) {
                self.head += consumed as u32;
                if advance {
                    self.head += slice.len() as u32;
                }
                return Ok((&current[..consumed], ()));
            } else {
                consumed += chr.len_utf8();
            }
        }
        dbg!("parse until", slice);
        Err(())
    }

    // Above modified to allow
    pub fn parse_until_postfix(&mut self, slice: &str, then: &str) -> Result<(&'a str, ()), ()> {
        let current = self.current();
        for (idx, _chr) in current.char_indices() {
            if current[idx..].starts_with(slice) && current[(idx + slice.len())..].starts_with(then)
            {
                self.head += idx as u32 + slice.len() as u32;
                return Ok((&current[..idx], ()));
            }
        }
        dbg!("parse until");
        Err(())
    }

    pub fn current(&self) -> &'a str {
        self.current_with_offset(0)
    }

    pub fn consumed(self) -> u32 {
        self.head
    }

    pub fn current_with_offset(&self, offset: u32) -> &'a str {
        unsafe { self.on.get_unchecked((self.head + offset) as usize..) }
        // &self.on[self.head as usize..]
    }

    pub fn parse_string_literal(&mut self) -> Result<(&'a str, ()), ()> {
        let mut chars = self.current().chars();
        let start = if let Some(chr) = chars.next() {
            if let '"' | '\'' = chr {
                chr
            } else {
                return Err(());
            }
        } else {
            return Err(());
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
            if chr == start {
                let slice = &self.on[(self.head as usize + 1)..(self.head as usize + consumed)];
                self.head += consumed as u32 + 1;
                return Ok((slice, ()));
            } else {
                escaped = matches!(chr, '\\');
            }
        }
        Err(())
    }

    pub fn parse_identifier(&mut self, position: &str) -> Result<&'a str, ()> {
        let mut chars = self.current().chars();
        let first = if let Some(chr) = chars.next() {
            let valid = chr.is_alphabetic(); // || matches!(chr, '-' | '_' | '$' | ':');
            if !valid {
                dbg!(chr, position, self.head);
                return Err(());
            } else {
                chr.len_utf8()
            }
        } else {
            return Err(());
        };
        let mut consumed = first;
        for chr in chars {
            // WIP
            let is_part_of_value = if let "Attribute value" = position {
                !(chr.is_whitespace() || matches!(chr, '>'))
            } else {
                chr.is_alphanumeric() || matches!(chr, '-' | '_' | '$' | ':')
            };
            if is_part_of_value {
                consumed += chr.len_utf8();
            } else {
                break;
            }
        }
        let slice = &self.on[(self.head as usize)..(self.head as usize + consumed)];
        self.head += consumed as u32;
        Ok(slice)
    }

    pub fn skip(&mut self) {
        let mut bytes = self.current().bytes().enumerate();
        while let Some((idx, byte)) = bytes.next() {
            if !byte.is_ascii_whitespace() {
                self.head += idx as u32;
                break;
            }
        }
    }

    pub fn expect(&mut self, chr: char) -> Result<(), ()> {
        self.skip();
        if self.current().starts_with(chr) {
            self.head += chr.len_utf8() as u32;
            Ok(())
        } else {
            dbg!(self.current().get(..10), chr);
            Err(())
        }
    }

    pub fn advance(&mut self, distance: u32) {
        self.head += distance;
    }
}
