use napi_derive_ohos::napi;

#[napi(object)]
#[derive(Debug, Default)]
pub struct Options {
    pub always_parse_as_big: Option<bool>,
    pub use_native_big_int: Option<bool>,
    pub parse_float_as_big: Option<bool>,
}
