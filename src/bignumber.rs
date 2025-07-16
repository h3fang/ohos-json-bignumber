use std::fmt::Display;

use bigdecimal::{BigDecimal, FromPrimitive, Num};
use napi_derive_ohos::napi;
use napi_ohos::bindgen_prelude::{Either3, Function, JavaScriptClassExt};
use napi_ohos::{Env, Error, JsNumber, JsObject, JsUnknown, Result, Status};

#[napi]
#[derive(Debug, Clone)]
pub struct BigNumber(pub(crate) BigDecimal);

#[napi]
impl BigNumber {
    #[napi(constructor)]
    pub fn new(n: Either3<JsNumber, String, &BigNumber>, base: Option<u32>) -> Result<Self> {
        match n {
            Either3::A(n) => {
                if let Ok(n) = n.get_int64() {
                    Ok(BigNumber(BigDecimal::from_i64(n).unwrap()))
                } else if let Ok(n) = n.get_int32() {
                    Ok(BigNumber(BigDecimal::from_i32(n).unwrap()))
                } else if let Ok(n) = n.get_uint32() {
                    Ok(BigNumber(BigDecimal::from_u32(n).unwrap()))
                } else if let Ok(n) = n.get_double() {
                    Ok(BigNumber(BigDecimal::from_f64(n).unwrap()))
                } else {
                    Err(Error::new(Status::InvalidArg, "invalid number"))
                }
            }
            Either3::B(s) => {
                let n = BigDecimal::from_str_radix(s.as_str(), base.unwrap_or(10))
                    .map_err(|e| Error::new(Status::InvalidArg, e))?;
                Ok(BigNumber(n))
            }
            Either3::C(n) => Ok(n.clone()),
        }
    }

    #[napi]
    pub fn is_big_number(env: Env, value: JsUnknown) -> bool {
        BigNumber::instance_of(env, &value).is_ok_and(|v| v)
    }

    #[napi]
    pub fn absolute_value(&self) -> Self {
        BigNumber(self.0.abs())
    }

    #[napi]
    pub fn abs(&self) -> Self {
        BigNumber(self.0.abs())
    }

    #[napi]
    pub fn compared_to(&self, n: &BigNumber) -> i32 {
        match self.0.cmp(&n.0) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }

    #[napi]
    pub fn decimal_places(&self) -> i64 {
        self.0.fractional_digit_count()
    }

    #[napi]
    pub fn dp(&self) -> i64 {
        self.0.fractional_digit_count()
    }

    #[napi]
    pub fn divided_by(&self, n: &BigNumber) -> Self {
        BigNumber(&self.0 / &n.0)
    }

    #[napi]
    pub fn div(&self, n: &BigNumber) -> Self {
        BigNumber(&self.0 / &n.0)
    }

    #[napi]
    pub fn divided_to_integer_by(&self, n: &BigNumber) -> Self {
        let v = &self.0 / &n.0;
        BigNumber(v.round(0))
    }

    #[napi]
    pub fn idiv(&self, n: &BigNumber) -> Self {
        self.divided_to_integer_by(n)
    }

    #[napi]
    pub fn negated(&self) -> Self {
        BigNumber(-&self.0)
    }

    #[napi]
    pub fn plus(&self, n: &BigNumber) -> Self {
        BigNumber(&self.0 + &n.0)
    }

    #[napi]
    pub fn minus(&self, n: &BigNumber) -> Self {
        BigNumber(&self.0 - &n.0)
    }

    #[napi]
    pub fn times(&self, n: &BigNumber) -> Self {
        BigNumber(&self.0 * &n.0)
    }

    #[napi]
    pub fn exp(&self) -> Self {
        BigNumber(self.0.exp())
    }

    #[napi]
    pub fn integer_value(&self) -> Self {
        BigNumber(self.0.round(0))
    }

    #[napi]
    pub fn modulo(&self, n: &BigNumber) -> Self {
        BigNumber(&self.0 % &n.0)
    }

    #[napi]
    pub fn square(&self) -> BigNumber {
        BigNumber(self.0.square())
    }

    #[napi]
    pub fn cube(&self) -> BigNumber {
        BigNumber(self.0.cube())
    }

    #[napi]
    pub fn sqrt(&self) -> Option<BigNumber> {
        self.0.sqrt().map(BigNumber)
    }

    #[napi]
    pub fn cbrt(&self) -> BigNumber {
        BigNumber(self.0.cbrt())
    }

    #[napi]
    pub fn to_fixed(&self, n: Option<u32>) -> String {
        match n {
            Some(n) => self.0.with_prec(n as u64).to_string(),
            None => self.0.to_plain_string(),
        }
    }

    #[napi]
    pub fn to_exponential(&self, n: u32) -> String {
        self.0.with_prec(n as u64).to_scientific_notation()
    }

    #[napi(js_name = "toString")]
    pub fn to_string_js(&self) -> String {
        self.to_string()
    }

    #[napi(js_name = "toJSON")]
    pub fn to_json(&self, env: Env) -> Result<JsUnknown> {
        let json: JsObject = env.get_global()?.get_named_property_unchecked("JSON")?;
        let raw_json: Function<'_, String, JsUnknown> =
            json.get_named_property_unchecked("rawJSON")?;

        let str = self.0.to_scientific_notation();
        raw_json.call(str)
    }
}

impl Display for BigNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.write_scientific_notation(f)
    }
}
