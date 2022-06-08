// Copyright (c) 2022 MASSA LABS <info@massa.net>

//! Provides various tools to manipulate ledger entries and changes happening on them.

use massa_serialization::{Deserializer, SerializeError, Serializer};
use nom::IResult;

/// Trait marking a structure that supports another one (V) being applied to it
pub trait Applicable<V> {
    /// apply changes from other to mutable self
    fn apply(&mut self, _: V);
}

/// Enumeration representing set/update/delete change on a value T
#[derive(Debug, Clone)]
pub enum SetUpdateOrDelete<T: Default + Applicable<V>, V: Applicable<V> + Clone> {
    /// Sets the value T a new absolute value T
    Set(T),

    /// Applies an update V to an existing value T.
    /// If the value T doesn't exist:
    /// a `new_t = T::default()` is created,
    /// the update V is applied to it,
    /// and the enumeration is changed to `SetUpdateOrDelete::Set(new_t)`
    Update(V),

    /// Deletes the value T
    Delete,
}

pub struct SetUpdateOrDeleteDeserializer<
    T: Default + Applicable<V>,
    V: Applicable<V> + Clone,
    DT: Deserializer<T>,
    DV: Deserializer<V>,
> {
    inner_deserializer_set: DT,
    inner_deserializer_update: DV,
    phantom_t: std::marker::PhantomData<T>,
    phantom_v: std::marker::PhantomData<V>,
}

impl<
        T: Default + Applicable<V>,
        V: Applicable<V> + Clone,
        DT: Deserializer<T>,
        DV: Deserializer<V>,
    > SetUpdateOrDeleteDeserializer<T, V, DT, DV>
{
    pub fn new(inner_deserializer_set: DT, inner_deserializer_update: DV) -> Self {
        Self {
            inner_deserializer_set,
            inner_deserializer_update,
            phantom_t: std::marker::PhantomData,
            phantom_v: std::marker::PhantomData,
        }
    }
}

impl<
        T: Default + Applicable<V>,
        V: Applicable<V> + Clone,
        DT: Deserializer<T>,
        DV: Deserializer<V>,
    > Deserializer<SetUpdateOrDelete<T, V>> for SetUpdateOrDeleteDeserializer<T, V, DT, DV>
{
    fn deserialize<'a>(&self, buffer: &'a [u8]) -> IResult<&'a [u8], SetUpdateOrDelete<T, V>> {
        match buffer[0] {
            0 => {
                let (rest, value) = self.inner_deserializer_set.deserialize(&buffer[1..])?;
                Ok((rest, SetUpdateOrDelete::Set(value)))
            }
            1 => {
                let (rest, value) = self.inner_deserializer_update.deserialize(&buffer[1..])?;
                Ok((rest, SetUpdateOrDelete::Update(value)))
            }
            2 => Ok((&buffer[1..], SetUpdateOrDelete::Delete)),
            _ => Err(nom::Err::Error(nom::error::Error::new(
                buffer,
                nom::error::ErrorKind::Digit,
            ))),
        }
    }
}

pub struct SetUpdateOrDeleteSerializer<
    T: Default + Applicable<V>,
    V: Applicable<V> + Clone,
    ST: Serializer<T>,
    SV: Serializer<V>,
> {
    inner_serializer_set: ST,
    inner_serializer_update: SV,
    phantom_t: std::marker::PhantomData<T>,
    phantom_v: std::marker::PhantomData<V>,
}

impl<
        T: Default + Applicable<V>,
        V: Applicable<V> + Clone,
        ST: Serializer<T>,
        SV: Serializer<V>,
    > SetUpdateOrDeleteSerializer<T, V, ST, SV>
{
    pub fn new(inner_serializer_set: ST, inner_serializer_update: SV) -> Self {
        Self {
            inner_serializer_set,
            inner_serializer_update,
            phantom_t: std::marker::PhantomData,
            phantom_v: std::marker::PhantomData,
        }
    }
}

impl<
        T: Default + Applicable<V>,
        V: Applicable<V> + Clone,
        ST: Serializer<T>,
        SV: Serializer<V>,
    > Serializer<SetUpdateOrDelete<T, V>> for SetUpdateOrDeleteSerializer<T, V, ST, SV>
{
    fn serialize(&self, value: &SetUpdateOrDelete<T, V>) -> Result<Vec<u8>, SerializeError> {
        let mut res = Vec::new();

        match value {
            SetUpdateOrDelete::Set(value) => {
                res.push(0);
                res.extend(self.inner_serializer_set.serialize(value)?);
                Ok(res)
            }
            SetUpdateOrDelete::Update(value) => {
                res.push(1);
                res.extend(self.inner_serializer_update.serialize(value)?);
                Ok(res)
            }
            SetUpdateOrDelete::Delete => {
                res.push(2);
                Ok(res)
            }
        }
    }
}

/// Support applying another `SetUpdateOrDelete` to self
impl<T: Default + Applicable<V>, V: Applicable<V>> Applicable<SetUpdateOrDelete<T, V>>
    for SetUpdateOrDelete<T, V>
where
    V: Clone,
{
    fn apply(&mut self, other: SetUpdateOrDelete<T, V>) {
        match other {
            // the other SetUpdateOrDelete sets a new absolute value => force it on self
            v @ SetUpdateOrDelete::Set(_) => *self = v,

            // the other SetUpdateOrDelete updates the value
            SetUpdateOrDelete::Update(u) => match self {
                // if self currently sets an absolute value, apply other to that value
                SetUpdateOrDelete::Set(cur) => cur.apply(u),

                // if self currently updates a value, apply the updates of the other to that update
                SetUpdateOrDelete::Update(cur) => cur.apply(u),

                // if self currently deletes a value,
                // create a new default value, apply other's updates to it and make self set it as an absolute new value
                SetUpdateOrDelete::Delete => {
                    let mut res = T::default();
                    res.apply(u);
                    *self = SetUpdateOrDelete::Set(res);
                }
            },

            // the other SetUpdateOrDelete deletes a value => force self to delete it as well
            v @ SetUpdateOrDelete::Delete => *self = v,
        }
    }
}

/// `Enum` representing a set/delete change on a value T
#[derive(Debug, Clone)]
pub enum SetOrDelete<T: Clone> {
    /// sets a new absolute value T
    Set(T),

    /// deletes the value
    Delete,
}

pub struct SetOrDeleteDeserializer<T: Clone, DT: Deserializer<T>> {
    inner_deserializer: DT,
    phantom_t: std::marker::PhantomData<T>,
}

impl<T: Clone, DT: Deserializer<T>> SetOrDeleteDeserializer<T, DT> {
    pub fn new(inner_deserializer: DT) -> Self {
        Self {
            inner_deserializer,
            phantom_t: std::marker::PhantomData,
        }
    }
}

impl<T: Clone, DT: Deserializer<T>> Deserializer<SetOrDelete<T>>
    for SetOrDeleteDeserializer<T, DT>
{
    fn deserialize<'a>(&self, buffer: &'a [u8]) -> IResult<&'a [u8], SetOrDelete<T>> {
        match buffer[0] {
            0 => {
                let (rest, value) = self.inner_deserializer.deserialize(&buffer[1..])?;
                Ok((rest, SetOrDelete::Set(value)))
            }
            1 => Ok((&buffer[1..], SetOrDelete::Delete)),
            _ => Err(nom::Err::Error(nom::error::Error::new(
                buffer,
                nom::error::ErrorKind::Digit,
            ))),
        }
    }
}

pub struct SetOrDeleteSerializer<T: Clone, ST: Serializer<T>> {
    inner_serializer: ST,
    phantom_t: std::marker::PhantomData<T>,
}

impl<T: Clone, ST: Serializer<T>> SetOrDeleteSerializer<T, ST> {
    pub fn new(inner_serializer: ST) -> Self {
        Self {
            inner_serializer,
            phantom_t: std::marker::PhantomData,
        }
    }
}

impl<T: Clone, ST: Serializer<T>> Serializer<SetOrDelete<T>> for SetOrDeleteSerializer<T, ST> {
    fn serialize(&self, value: &SetOrDelete<T>) -> Result<Vec<u8>, SerializeError> {
        let mut res = Vec::new();

        match value {
            SetOrDelete::Set(value) => {
                res.push(0);
                res.extend(self.inner_serializer.serialize(value)?);
                Ok(res)
            }
            SetOrDelete::Delete => {
                res.push(1);
                Ok(res)
            }
        }
    }
}

/// allows applying another `SetOrDelete` to the current one
impl<T: Clone> Applicable<SetOrDelete<T>> for SetOrDelete<T> {
    fn apply(&mut self, other: Self) {
        *self = other;
    }
}

/// represents a set/keep change
#[derive(Debug, Clone)]
pub enum SetOrKeep<T: Clone> {
    /// sets a new absolute value T
    Set(T),

    /// keeps the existing value
    Keep,
}

pub struct SetOrKeepDeserializer<T: Clone, DT: Deserializer<T>> {
    inner_deserializer: DT,
    phantom_t: std::marker::PhantomData<T>,
}

impl<T: Clone, DT: Deserializer<T>> SetOrKeepDeserializer<T, DT> {
    pub fn new(inner_deserializer: DT) -> Self {
        Self {
            inner_deserializer,
            phantom_t: std::marker::PhantomData,
        }
    }
}

impl<T: Clone, DT: Deserializer<T>> Deserializer<SetOrKeep<T>> for SetOrKeepDeserializer<T, DT> {
    fn deserialize<'a>(&self, buffer: &'a [u8]) -> IResult<&'a [u8], SetOrKeep<T>> {
        match buffer[0] {
            0 => {
                let (rest, value) = self.inner_deserializer.deserialize(&buffer[1..])?;
                Ok((rest, SetOrKeep::Set(value)))
            }
            1 => Ok((&buffer[1..], SetOrKeep::Keep)),
            _ => Err(nom::Err::Error(nom::error::Error::new(
                buffer,
                nom::error::ErrorKind::Digit,
            ))),
        }
    }
}

pub struct SetOrKeepSerializer<T: Clone, ST: Serializer<T>> {
    inner_serializer: ST,
    phantom_t: std::marker::PhantomData<T>,
}

impl<T: Clone, ST: Serializer<T>> SetOrKeepSerializer<T, ST> {
    pub fn new(inner_serializer: ST) -> Self {
        Self {
            inner_serializer,
            phantom_t: std::marker::PhantomData,
        }
    }
}

impl<T: Clone, ST: Serializer<T>> Serializer<SetOrKeep<T>> for SetOrKeepSerializer<T, ST> {
    fn serialize(&self, value: &SetOrKeep<T>) -> Result<Vec<u8>, SerializeError> {
        let mut res = Vec::new();

        match value {
            SetOrKeep::Set(value) => {
                res.push(0);
                res.extend(self.inner_serializer.serialize(value)?);
                Ok(res)
            }
            SetOrKeep::Keep => {
                res.push(1);
                Ok(res)
            }
        }
    }
}

/// allows applying another `SetOrKeep` to the current one
impl<T: Clone> Applicable<SetOrKeep<T>> for SetOrKeep<T> {
    fn apply(&mut self, other: SetOrKeep<T>) {
        if let v @ SetOrKeep::Set(..) = other {
            // update the current value only if the other SetOrKeep sets a new one
            *self = v;
        }
    }
}

impl<T: Clone> SetOrKeep<T> {
    /// applies the current `SetOrKeep` to a target mutable value
    pub fn apply_to(self, val: &mut T) {
        if let SetOrKeep::Set(v) = self {
            // only change the value if self is setting a new one
            *val = v;
        }
    }
}

/// By default, `SetOrKeep` keeps the existing value
impl<T: Clone> Default for SetOrKeep<T> {
    fn default() -> Self {
        SetOrKeep::Keep
    }
}
