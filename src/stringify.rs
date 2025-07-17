use std::fmt::Write;

use napi_derive_ohos::napi;
use napi_ohos::bindgen_prelude::{FromNapiValue, JavaScriptClassExt};
use napi_ohos::{
    Env, Error, JsBigInt, JsBoolean, JsNumber, JsObject, JsString, JsUnknown, NapiRaw, Result,
    Status, ValueType,
};

use crate::bignumber::BigNumber;

#[napi]
#[allow(dead_code)]
pub fn stringify(env: Env, value: JsUnknown) -> Result<String> {
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

    fn stringify(&mut self, value: JsUnknown) -> Result<String> {
        let mut output = String::with_capacity(1024);
        self.write_value(&mut output, value)?;
        Ok(output)
    }

    fn write_value(&mut self, output: &mut String, value: JsUnknown) -> Result<()> {
        match value.get_type()? {
            ValueType::String => self.write_string(output, unsafe { value.cast() }),
            ValueType::BigInt => self.write_bigint(output, unsafe { value.cast() }),
            ValueType::Object => self.write_object(output, unsafe { value.cast() }),
            ValueType::Number => self.write_number(output, unsafe { value.cast() }),
            ValueType::Boolean => self.write_boolean(output, unsafe { value.cast() }),
            _ => self.write_null(output),
        }
    }

    fn write_null(&self, output: &mut String) -> Result<()> {
        output.push_str("null");
        Ok(())
    }

    fn write_boolean(&self, output: &mut String, value: JsBoolean) -> Result<()> {
        let v = if value.get_value()? { "true" } else { "false" };
        output.push_str(v);
        Ok(())
    }

    fn write_number(&self, output: &mut String, value: JsNumber) -> Result<()> {
        let s = value.coerce_to_string()?;
        let s = s.into_utf16()?;
        output.push_str(&s.as_str()?);
        Ok(())
    }

    fn write_string(&self, output: &mut String, value: JsString) -> Result<()> {
        let s = value.into_utf16()?;
        let s = s.as_str()?;
        output.reserve(2 + s.len() * 2);

        output.push('"');
        for c in s.chars() {
            match c {
                '"' => output.push_str(r#"\""#),
                '\\' => output.push_str(r"\\"),
                '\n' => output.push_str(r"\n"),
                '\r' => output.push_str(r"\r"),
                '\t' => output.push_str(r"\t"),
                '\u{0008}' => output.push_str(r"\b"),
                '\u{000C}' => output.push_str(r"\f"),
                '\u{0000}'..='\u{001F}' => {
                    write!(output, "\\u{:04X}", c as u32)
                        .map_err(|e| Error::new(Status::GenericFailure, e))?;
                }
                _ => output.push(c),
            }
        }
        output.push('"');
        Ok(())
    }

    fn write_object(&mut self, output: &mut String, obj: JsObject) -> Result<()> {
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
            output.push('[');
            for i in 0..length {
                if i > 0 {
                    output.push(',');
                }
                let element: JsUnknown = obj.get_element_unchecked(i)?;
                self.write_value(output, element)?;
            }
            output.push(']');
            return Ok(());
        }

        // Handle object
        output.push('{');
        let names = obj.get_property_names()?;
        let len = names.get_array_length_unchecked()?;
        for i in 0..len {
            if i > 0 {
                output.push(',');
            }

            let key: JsString = names.get_element_unchecked(i)?;
            self.write_string(output, key)?;

            output.push(':');

            let value: JsUnknown = obj.get_property_unchecked(key)?;
            self.write_value(output, value)?;
        }
        output.push('}');
        Ok(())
    }

    fn write_bigint(&self, output: &mut String, mut bigint: JsBigInt) -> Result<()> {
        use bigdecimal::num_bigint::{BigInt, Sign};
        let (sign, words) = bigint.get_words()?;
        let sign = if sign { Sign::Minus } else { Sign::Plus };
        let words: Vec<_> = words
            .into_iter()
            .flat_map(|w| [(w & 0xFFFFFFFF) as u32, (w >> 32) as u32])
            .collect();
        let num = BigInt::from_slice(sign, &words);
        write!(output, "{num}").map_err(|e| Error::new(Status::GenericFailure, e))?;
        Ok(())
    }
}
