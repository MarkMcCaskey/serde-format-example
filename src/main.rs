use serde::de::{self, DeserializeSeed, SeqAccess, Visitor};
use serde::Deserialize;
use std::fmt::{self, Display};

#[derive(Deserialize, Debug, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Deserialize, Debug, PartialEq)]
struct Point64 {
    x: i64,
    y: i64,
}

#[derive(Deserialize, Debug, PartialEq)]
struct Compound {
    points: (Point, Point),
    more_points: Vec<Point64>,
}

#[derive(Debug, Clone)]
pub enum Value {
    I32(i32),
    I64(i64),
}

#[test]
fn it_works() {
    let vector: Vec<Value> = vec![Value::I32(1), Value::I32(2)];
    let result: Point = from_values(&vector).unwrap();

    assert_eq!(result, Point { x: 1, y: 2 });

    let vector64: Vec<Value> = vec![Value::I64(1), Value::I64(2)];
    let result: Point64 = from_values(&vector64).unwrap();
    assert_eq!(result, Point64 { x: 1, y: 2 });

    let result: Result<Point64, _> = from_values(&vector);
    assert!(result.is_err());
}

#[test]
fn harder_test() {
    let vector: Vec<Value> = vec![
        Value::I32(1),
        Value::I32(2),
        Value::I32(3),
        Value::I32(4),
        Value::I64(5),
        Value::I64(6),
    ];
    let result: Compound = from_values(&vector).unwrap();

    assert_eq!(
        result,
        Compound {
            points: (Point { x: 1, y: 2 }, Point { x: 3, y: 4 }),
            more_points: vec![Point64 { x: 5, y: 6 }],
        }
    );
}

pub struct Deserializer<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    input: &'de [Value],
    idx: usize,
}

impl<'de> Deserializer<'de> {
    // By convention, `Deserializer` constructors are named like `from_xyz`.
    // That way basic use cases are satisfied by something like
    // `serde_json::from_str(...)` while advanced use cases that require a
    // deserializer can make one with `serde_json::Deserializer::from_str(...)`.
    pub fn from_values(input: &'de [Value]) -> Self {
        Deserializer { input, idx: 0 }
    }
}

impl<'de> Deserializer<'de> {
    fn peek_value(&mut self) -> Result<&Value, Error> {
        if self.idx >= self.input.len() {
            return Err(Error::InputEmpty);
        }
        Ok(&self.input[self.idx])
    }

    fn next_value(&mut self) -> Result<&Value, Error> {
        if self.idx >= self.input.len() {
            return Err(Error::InputEmpty);
        }
        let old_idx = self.idx;
        self.idx += 1;
        Ok(&self.input[old_idx])
    }

    fn next_i32(&mut self) -> Result<i32, Error> {
        match *self.peek_value()? {
            Value::I32(v) => {
                self.idx += 1;
                Ok(v)
            }
            Value::I64(_) => Err(Error::TypeMismatch {
                expected: "i32",
                found: "i64",
            }),
        }
    }

    fn next_i64(&mut self) -> Result<i64, Error> {
        match *self.peek_value()? {
            Value::I64(v) => {
                self.idx += 1;
                Ok(v)
            }
            Value::I32(_) => Err(Error::TypeMismatch {
                expected: "i64",
                found: "i32",
            }),
        }
    }
}

pub fn from_values<'a, T>(s: &'a [Value]) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_values(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.idx >= deserializer.input.len() {
        Ok(t)
    } else {
        Err(Error::InputNotEmpty)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    InputNotEmpty,
    InputEmpty,
    TypeMismatch {
        expected: &'static str,
        found: &'static str,
    },
    Message(String),
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(std::error::Error::description(self))
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InputNotEmpty => "unexpected input remaining",
            Error::Message(ref msg) => msg,
            Error::InputEmpty => "unexpected end of input",
            Error::TypeMismatch { .. } => {
                "type mismatch detected"
                //&format!("type error: expected `{}` but found `{}`", expected, found)
            }
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.peek_value()? {
            Value::I32(_) => self.deserialize_i32(visitor),
            Value::I64(_) => self.deserialize_i64(visitor),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.next_i32()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.next_i64()?)
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(Values::new(&mut self))
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }
}

struct Values<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Values<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Values { de }
    }
}

impl<'de, 'a> SeqAccess<'de> for Values<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.de.peek_value().is_err() {
            return Ok(None);
        }
        seed.deserialize(&mut *self.de).map(Some)
    }
}

fn main() {
    println!("Hello, world!");
}
