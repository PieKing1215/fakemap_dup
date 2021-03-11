use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;
use std::borrow::Borrow;
use std::iter::FromIterator;

#[derive(Clone, Debug, Eq, Hash, PartialOrd, PartialEq)]
pub struct FakeMap<K, V> {
    items: Vec<(K, V)>,
}

impl<K, V> Default for FakeMap<K, V> {
    fn default() -> Self {
        FakeMap::new()
    }
}

impl<K, V> FakeMap<K, V> {
    pub fn new() -> Self {
        FakeMap { items: Vec::new() }
    }

    pub fn with_capacity(cap: usize) -> Self {
        FakeMap {
            items: Vec::with_capacity(cap),
        }
    }

    #[inline]
    pub fn insert(&mut self, k: K, v: V) {
        self.items.push((k, v));
    }

    pub fn get_at(&self, index: usize) -> &V {
        &self.items[index].1
    }

    fn get_idx_of_key<Q: ?Sized>(&self, key: &Q) -> Option<usize>
    where
        Q: PartialEq,
        K: Borrow<Q>
    {
        self.items.iter().position(|item| item.0.borrow() == key)
    }

    #[inline]
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        Q: PartialEq,
        K: Borrow<Q>
    {
        self.get_idx_of_key(key).map(|idx| &self.items[idx].1)
    }

    pub fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&mut K, &mut V)> {
        self.items.iter_mut().map(|kv| (&mut kv.0, &mut kv.1))
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.items.iter().map(|kv| &kv.0)
    }

    pub fn keys_mut(&self) -> impl Iterator<Item = &K> {
        self.items.iter().map(|kv| &kv.0)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.items.iter().map(|kv| &kv.1)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.items.iter_mut().map(|kv| &mut kv.1)
    }
}

impl<K, V> FromIterator<(K, V)> for FakeMap<K, V> {
    fn from_iter<T: IntoIterator<Item=(K, V)>>(iter: T) -> Self {
        FakeMap {
            items: iter.into_iter().collect(),
        }
    }
}

// A Visitor is a type that holds methods that a Deserializer can drive
// depending on what is contained in the input data.
//
// In the case of a map we need generic type parameters K and V to be
// able to set the output type correctly, but don't require any state.
// This is an example of a "zero sized type" in Rust. The PhantomData
// keeps the compiler from complaining about unused generic type
// parameters.
struct FakeMapVisitor<K, V> {
    marker: PhantomData<fn() -> FakeMap<K, V>>,
}

impl<K, V> FakeMapVisitor<K, V> {
    fn new() -> Self {
        FakeMapVisitor {
            marker: PhantomData,
        }
    }
}

// This is the trait that Deserializers are going to be driving. There
// is one method for each type of data that our type knows how to
// deserialize from. There are many other methods that are not
// implemented here, for example deserializing from integers or strings.
// By default those methods will return an error, which makes sense
// because we cannot deserialize a MyMap from an integer or string.
impl<'de, K, V> Visitor<'de> for FakeMapVisitor<K, V>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    // The type that our Visitor is going to produce.
    type Value = FakeMap<K, V>;

    // Format a message stating what data this Visitor expects to receive.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map")
    }

    // Deserialize MyMap from an abstract "map" provided by the
    // Deserializer. The MapAccess input is a callback provided by
    // the Deserializer to let us see each entry in the map.
    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = FakeMap::with_capacity(access.size_hint().unwrap_or(0));

        // While there are entries remaining in the input, add them
        // into our map.
        while let Some((key, value)) = access.next_entry()? {
            map.insert(key, value);
        }

        Ok(map)
    }
}

impl<'de, K, V> Deserialize<'de> for FakeMap<K, V>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(FakeMapVisitor::new())
    }
}

impl<K, V> Serialize for FakeMap<K, V>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_map(Some(self.items.len()))?;

        for (key, value) in self.iter() {
            serializer.serialize_entry(key, value)?;
        }

        serializer.end()
    }
}
