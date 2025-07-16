use std::str::Chars;
use std::{iter::Peekable, str::FromStr};

use bigdecimal::num_bigint::Sign;
use bigdecimal::{BigDecimal, Num};
use napi_derive_ohos::napi;
use napi_ohos::bindgen_prelude::ToNapiValue;
use napi_ohos::{Env, Error, JsString, JsUnknown, NapiValue, Result, Status};

use crate::bignumber::BigNumber;
use crate::error::ParseError;
use crate::options::Options;

#[napi]
#[allow(dead_code)]
pub fn parse(env: Env, s: String, options: Option<Options>) -> Result<JsUnknown> {
    let opts = options.unwrap_or_default();
    let mut parser = JsonParser::new(&s, opts, env);
    parser.parse()
}

pub struct JsonParser<'a> {
    chars: Peekable<Chars<'a>>,
    opts: Options,
    env: Env,
}

impl<'a> JsonParser<'a> {
    pub fn new(input: &'a str, opts: Options, env: Env) -> Self {
        JsonParser {
            chars: input.chars().peekable(),
            opts,
            env,
        }
    }

    pub fn parse(&mut self) -> Result<JsUnknown> {
        let value = self.parse_value()?;
        self.skip_whitespace();
        if self.chars.peek().is_some() {
            return Err(ParseError::TrailingCharacters.into());
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsUnknown> {
        self.skip_whitespace();
        match self.chars.peek() {
            Some('n') => self.parse_null(),
            Some('t') => self.parse_true(),
            Some('f') => self.parse_false(),
            Some('"') => {
                let s = self.parse_string()?;
                let s = self.to_js_string(s)?;
                Ok(s.into_unknown())
            }
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_ascii_digit() || *c == '-' => self.parse_number(),
            Some(c) => Err(ParseError::UnexpectedCharacter(*c).into()),
            None => Err(ParseError::UnexpectedEndOfInput.into()),
        }
    }

    fn parse_null(&mut self) -> Result<JsUnknown> {
        self.expect_str("null")?;
        let null = self.env.get_null()?;
        Ok(null.into_unknown())
    }

    fn parse_true(&mut self) -> Result<JsUnknown> {
        self.expect_str("true")?;
        let t = self.env.get_boolean(true)?;
        Ok(t.into_unknown())
    }

    fn parse_false(&mut self) -> Result<JsUnknown> {
        self.expect_str("false")?;
        let t = self.env.get_boolean(false)?;
        Ok(t.into_unknown())
    }

    fn parse_number(&mut self) -> Result<JsUnknown> {
        let mut num_str = String::with_capacity(64);
        let mut has_decimal = false;
        let mut has_exponent = false;

        if let Some('-') = self.chars.peek() {
            num_str.push(self.chars.next().unwrap());
        }

        while let Some(c) = self.chars.peek() {
            match c {
                '0'..='9' => {
                    num_str.push(self.chars.next().unwrap());
                }
                '.' if !has_decimal && !has_exponent => {
                    num_str.push(self.chars.next().unwrap());
                    has_decimal = true;
                }
                'e' | 'E' if !has_exponent => {
                    num_str.push(self.chars.next().unwrap());
                    has_exponent = true;

                    if let Some('+' | '-') = self.chars.peek() {
                        num_str.push(self.chars.next().unwrap());
                    }
                }
                _ => break,
            }
        }

        if has_decimal || has_exponent {
            if self.opts.parse_float_as_big.is_some_and(|e| e) {
                let big_dec = BigDecimal::from_str(&num_str)
                    .map_err(|e| Error::new(Status::InvalidArg, e))?;
                let n = BigNumber(big_dec);
                let napi_value = unsafe { BigNumber::to_napi_value(self.env.raw(), n)? };
                unsafe { JsUnknown::from_raw(self.env.raw(), napi_value) }
            } else if let Ok(v) = num_str.parse::<f64>()
                && v.is_finite()
            {
                let napi_value = self.env.create_double(v)?;
                Ok(napi_value.into_unknown())
            } else {
                Ok(self.env.get_null()?.into_unknown())
            }
        } else {
            if !self.opts.always_parse_as_big.is_some_and(|e| e)
                && let Ok(v) = num_str.as_str().parse::<i64>()
            {
                const MAX: i64 = 9_007_199_254_740_991;
                if (-MAX..=MAX).contains(&v) {
                    return Ok(self.env.create_int64(v)?.into_unknown());
                }
            }

            let bigint = bigdecimal::num_bigint::BigInt::from_str_radix(&num_str, 10)
                .map_err(|e| Error::new(Status::InvalidArg, e))?;
            let (sign, int) = bigint.into_parts();
            let sign_bit = sign == Sign::Minus;
            let words = int.to_u64_digits();
            self.env
                .create_bigint_from_words(sign_bit, words)?
                .into_unknown()
        }
    }

    fn parse_string(&mut self) -> Result<String> {
        self.expect_char('"')?;
        let mut s = String::new();

        loop {
            match self.chars.next() {
                Some('"') => {
                    break;
                }
                Some('\\') => match self.chars.next() {
                    Some('"') => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some('/') => s.push('/'),
                    Some('b') => s.push('\u{0008}'),
                    Some('f') => s.push('\u{000C}'),
                    Some('n') => s.push('\n'),
                    Some('r') => s.push('\r'),
                    Some('t') => s.push('\t'),
                    Some('u') => {
                        let hex_str = self.read_n_chars(4)?;
                        let code = u32::from_str_radix(&hex_str, 16)
                            .map_err(|_| ParseError::InvalidEscapeSequence('u'))?;
                        s.push(
                            std::char::from_u32(code)
                                .ok_or(ParseError::InvalidEscapeSequence('u'))?,
                        );
                    }
                    Some(c) => return Err(ParseError::InvalidEscapeSequence(c).into()),
                    None => return Err(ParseError::UnexpectedEndOfInput.into()),
                },
                Some(c) => {
                    s.push(c);
                }
                None => return Err(ParseError::UnexpectedEndOfInput.into()),
            }
        }
        Ok(s)
    }

    fn parse_array(&mut self) -> Result<JsUnknown> {
        self.expect_char('[')?;
        self.skip_whitespace();

        let mut array = self.env.create_array(0)?;

        if let Some(']') = self.chars.peek() {
            self.chars.next();
            let obj = array.coerce_to_object()?;
            return Ok(obj.into_unknown());
        }

        let mut index = 0;
        loop {
            let value = self.parse_value()?;
            array.set(index, value)?;
            index += 1;
            self.skip_whitespace();

            match self.chars.peek() {
                Some(',') => {
                    self.chars.next();
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.chars.next();
                    break;
                }
                Some(c) => return Err(ParseError::UnexpectedCharacter(*c).into()),
                None => return Err(ParseError::UnexpectedEndOfInput.into()),
            }
        }

        let obj = array.coerce_to_object()?;
        Ok(obj.into_unknown())
    }

    fn parse_object(&mut self) -> Result<JsUnknown> {
        self.expect_char('{')?;
        self.skip_whitespace();

        let mut obj = self.env.create_object()?;

        if let Some('}') = self.chars.peek() {
            self.chars.next();
            return Ok(obj.into_unknown());
        }

        loop {
            let key = self.parse_string()?;

            self.skip_whitespace();
            self.expect_char(':')?;
            self.skip_whitespace();

            let value = self.parse_value()?;

            obj.set_named_property(&key, value)?;

            self.skip_whitespace();

            match self.chars.peek() {
                Some(',') => {
                    self.chars.next();
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.chars.next();
                    break;
                }
                Some(c) => return Err(ParseError::UnexpectedCharacter(*c).into()),
                None => return Err(ParseError::UnexpectedEndOfInput.into()),
            }
        }

        Ok(obj.into_unknown())
    }

    #[inline]
    fn to_js_string(&self, s: String) -> Result<JsString> {
        self.env.create_string_from_std(s)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn expect_char(&mut self, expected: char) -> std::result::Result<(), ParseError> {
        match self.chars.next() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(ParseError::UnexpectedCharacter(c)),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
    }

    fn expect_str(&mut self, expected: &str) -> std::result::Result<(), ParseError> {
        for c in expected.chars() {
            self.expect_char(c)?;
        }
        Ok(())
    }

    fn read_n_chars(&mut self, n: usize) -> std::result::Result<String, ParseError> {
        let mut s = String::with_capacity(n);
        for _ in 0..n {
            match self.chars.next() {
                Some(c) => {
                    s.push(c);
                }
                None => return Err(ParseError::UnexpectedEndOfInput),
            }
        }
        Ok(s)
    }
}
