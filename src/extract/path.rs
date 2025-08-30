use serde::Deserializer;
use serde::de::{DeserializeOwned, MapAccess, Visitor, value};
use std::collections::HashMap;
use std::ops::Deref;
use crate::body::Empty;
use crate::Request;
use crate::request::Extract;

pub struct Path<T>(pub T);

impl<S, T> Extract<S> for Path<T>
where
    T: DeserializeOwned
{
    type Error = Empty;

    fn extract(request: &Request, _: &S) -> impl Future<Output=Result<Self, Self::Error>> {
        async {
            let params = request
                .extensions()
                .get::<HashMap<String, String>>()
                .ok_or_else(|| Empty)?;

            let path =
                T::deserialize(PathDeserializer(params)).map_err(|_| Empty)?;

            Ok(Path(path))
        }
    }
}

impl<T> Deref for Path<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct PathDeserializer<'a>(&'a HashMap<String, String>);

impl<'de, 'a> Deserializer<'de> for PathDeserializer<'a> {
    type Error = value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(PathMapAccess {
            iter: self.0.iter(),
            value_opt: None,
        })
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        seq tuple tuple_struct map struct enum identifier ignored_any option unit
        bytes byte_buf unit_struct newtype_struct
    }
}

struct PathMapAccess<'a> {
    iter: std::collections::hash_map::Iter<'a, String, String>,
    value_opt: Option<&'a String>,
}

impl<'de, 'a> MapAccess<'de> for PathMapAccess<'a> {
    type Error = value::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if let Some((k, v)) = self.iter.next() {
            self.value_opt = Some(v);
            let key_de = value::StrDeserializer::new(k);
            let key = seed.deserialize(key_de)?;
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let v = self.value_opt.take().expect("value missing for key");
        let val_de = value::StrDeserializer::new(v);
        seed.deserialize(val_de)
    }
}
