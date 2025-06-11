use poem_openapi::{
    registry::{MetaSchemaRef, Registry},
    types::{ParseError, ParseFromJSON, ToHeader, ToJSON, Type},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt, ops::Deref, str::FromStr};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Zeroizing<T>(pub T)
where
    T: Zeroize;

impl<T> FromStr for Zeroizing<T>
where
    T: FromStr + Zeroize,
{
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        T::from_str(s).map(Self)
    }
}

impl<T> fmt::Display for Zeroizing<T>
where
    T: fmt::Display + Zeroize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Deref for Zeroizing<T>
where
    T: Zeroize,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> ParseFromJSON for Zeroizing<T>
where
    T: ParseFromJSON + Zeroize,
{
    fn parse_from_json(value: Option<Value>) -> poem_openapi::types::ParseResult<Self> {
        T::parse_from_json(value)
            .map(Self)
            .map_err(ParseError::propagate)
    }
}

impl<T> ToJSON for Zeroizing<T>
where
    T: ToJSON + Zeroize,
{
    fn to_json(&self) -> Option<Value> {
        self.0.to_json()
    }
}

impl<T> ToHeader for Zeroizing<T>
where
    T: ToHeader + Zeroize,
{
    fn to_header(&self) -> Option<poem::http::HeaderValue> {
        self.0.to_header()
    }
}

impl<T> Type for Zeroizing<T>
where
    T: Type + Zeroize,
{
    const IS_REQUIRED: bool = T::IS_REQUIRED;

    type RawValueType = T::RawValueType;

    type RawElementValueType = T::RawElementValueType;

    fn name() -> std::borrow::Cow<'static, str> {
        T::name()
    }

    fn schema_ref() -> MetaSchemaRef {
        T::schema_ref()
    }

    fn register(registry: &mut Registry) {
        T::register(registry);
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        self.0.as_raw_value()
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        self.0.raw_element_iter()
    }
}

impl<T> Zeroize for Zeroizing<T>
where
    T: Zeroize,
{
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl<T> ZeroizeOnDrop for Zeroizing<T> where T: Zeroize {}

impl<T> Drop for Zeroizing<T>
where
    T: Zeroize,
{
    fn drop(&mut self) {
        self.zeroize();
    }
}
