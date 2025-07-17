use std::fmt::Write;

use napi_derive_ohos::napi;
use napi_ohos::bindgen_prelude::{FromNapiValue, JavaScriptClassExt};
use napi_ohos::{
    Env, Error, JsBigInt, JsBoolean, JsNumber, JsObject, JsString, JsUnknown, NapiRaw, Result,
    Status, ValueType,
};
use widestring::{Utf16Str, Utf16String, utf16str};

use crate::bignumber::BigNumber;

#[napi]
#[allow(dead_code)]
pub fn stringify(env: Env, value: JsUnknown) -> Result<JsString> {
    let mut stringifier = JsonStringifier::new(env);
    stringifier.stringify(value)
}

struct JsonStringifier {
    env: Env,
}

impl JsonStringifier {
    fn new(env: Env) -> Self {
        JsonStringifier { env }
    }

    fn stringify(&mut self, value: JsUnknown) -> Result<JsString> {
        let mut output = Utf16String::with_capacity(1024);
        self.write_value(&mut output, value)?;
        self.env.create_string_utf16(output.as_slice())
    }

    fn write_value(&mut self, output: &mut Utf16String, value: JsUnknown) -> Result<()> {
        match value.get_type()? {
            ValueType::String => self.write_string(output, unsafe { value.cast() }),
            ValueType::BigInt => self.write_bigint(output, unsafe { value.cast() }),
            ValueType::Object => self.write_object(output, unsafe { value.cast() }),
            ValueType::Number => self.write_number(output, unsafe { value.cast() }),
            ValueType::Boolean => self.write_boolean(output, unsafe { value.cast() }),
            _ => self.write_null(output),
        }
    }

    fn write_null(&self, output: &mut Utf16String) -> Result<()> {
        output.push_utfstr(utf16str!("null"));
        Ok(())
    }

    fn write_boolean(&self, output: &mut Utf16String, value: JsBoolean) -> Result<()> {
        let v = if value.get_value()? {
            utf16str!("true")
        } else {
            utf16str!("false")
        };
        output.push_utfstr(v);
        Ok(())
    }

    fn write_number(&self, output: &mut Utf16String, value: JsNumber) -> Result<()> {
        let utf16_c = value.coerce_to_string()?.into_utf16()?;
        let (_, s) = utf16_c.as_slice().split_last().unwrap();
        let s = unsafe { Utf16Str::from_slice_unchecked(s) };
        output.push_utfstr(s);
        Ok(())
    }

    fn write_string(&self, output: &mut Utf16String, value: JsString) -> Result<()> {
        let utf16_c = value.into_utf16()?;
        let (_, s) = utf16_c.as_slice().split_last().unwrap();
        let s = unsafe { Utf16Str::from_slice_unchecked(s) };
        output.reserve(2 + s.len());

        output.push_utfstr(utf16str!("\""));
        for c in s.chars() {
            match c {
                '"' => output.push_utfstr(utf16str!(r#"\""#)),
                '\\' => output.push_utfstr(utf16str!(r"\\")),
                '\n' => output.push_utfstr(utf16str!(r"\n")),
                '\r' => output.push_utfstr(utf16str!(r"\r")),
                '\t' => output.push_utfstr(utf16str!(r"\t")),
                '\u{0008}' => output.push_utfstr(utf16str!(r"\b")),
                '\u{000C}' => output.push_utfstr(utf16str!(r"\f")),
                '\u{0000}'..='\u{001F}' => {
                    write!(output, "\\u{:04X}", c as u32)
                        .map_err(|e| Error::new(Status::GenericFailure, e))?;
                }
                _ => output.push(c),
            }
        }
        output.push_utfstr(utf16str!("\""));
        Ok(())
    }

    fn write_object(&mut self, output: &mut Utf16String, obj: JsObject) -> Result<()> {
        // Handle BigNumber
        if BigNumber::instance_of(self.env, &obj)? {
            let num: &BigNumber =
                unsafe { FromNapiValue::from_napi_value(self.env.raw(), obj.raw())? };
            let s = num.to_string();
            output.push_str(&s);
            return Ok(());
        }

        // Handle array
        if obj.is_array()? {
            let length = obj.get_array_length()?;
            output.push_utfstr(utf16str!("["));
            for i in 0..length {
                if i > 0 {
                    output.push(',');
                }
                let element: JsUnknown = obj.get_element_unchecked(i)?;
                self.write_value(output, element)?;
            }
            output.push_utfstr(utf16str!("]"));
            return Ok(());
        }

        // Handle object
        output.push_utfstr(utf16str!("{"));
        let names = obj.get_property_names()?;
        let len = names.get_array_length_unchecked()?;
        for i in 0..len {
            if i > 0 {
                output.push_utfstr(utf16str!(","));
            }

            let key: JsString = names.get_element_unchecked(i)?;
            self.write_string(output, key)?;

            output.push_utfstr(utf16str!(":"));

            let value: JsUnknown = obj.get_property_unchecked(key)?;
            self.write_value(output, value)?;
        }
        output.push_utfstr(utf16str!("}"));
        Ok(())
    }

    fn write_bigint(&self, output: &mut Utf16String, bigint: JsBigInt) -> Result<()> {
        let utf16_c = bigint.coerce_to_string()?.into_utf16()?;
        let (_, utf16) = utf16_c.split_last().unwrap();
        let s = unsafe { Utf16Str::from_slice_unchecked(utf16) };
        output.push_utfstr(s);
        Ok(())
    }
}
