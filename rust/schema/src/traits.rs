use std::{collections::HashSet, time::Duration};

use automerge::{transaction::Transactable, ObjId, ObjType, Prop, ScalarValue, Value, ROOT};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use common::eyre::{bail, Result};
use indexmap::IndexMap;
use similar::{Algorithm, DiffTag, TextDiffConfig};
use smol_str::SmolStr;

use crate::{
    store::{ReadStore, Store},
    types::{Array, Boolean, Integer, Null, Number, Object, Primitive, Text},
};

macro_rules! bail_type {
    ($message:literal) => {
        bail!($message, type = std::any::type_name::<Self>())
    };
}

macro_rules! bail_load_unexpected {
    ($unexpected:literal) => {
        bail!(
            "unexpected Automerge `{unexpected}` while attempting to load `{type}` from store",
            unexpected = $unexpected,
            type = std::any::type_name::<Self>()
        )
    };
}

const SIMILARITY_MAX: usize = 1000;

pub trait Node: Sized {
    fn load<S: ReadStore>(store: &S) -> Result<Self> {
        Self::load_root(store)
    }

    fn load_root<S: ReadStore>(_store: &S) -> Result<Self> {
        bail_type!("attempting to load node of type `{type}` from store; only maps allowed at root")
    }

    fn load_prop<S: ReadStore>(store: &S, obj: &ObjId, prop: Prop) -> Result<Self> {
        match store.get(obj, prop)? {
            Some((Value::Scalar(scalar), ..)) => match scalar.as_ref() {
                ScalarValue::Null => Self::load_null(),
                ScalarValue::Boolean(value) => Self::load_boolean(value),
                ScalarValue::Int(value) => Self::load_int(value),
                ScalarValue::Uint(value) => Self::load_uint(value),
                ScalarValue::F64(value) => Self::load_f64(value),
                ScalarValue::Str(value) => Self::load_str(value),
                ScalarValue::Counter(..) => Self::load_counter(),
                ScalarValue::Timestamp(value) => Self::load_timestamp(value),
                ScalarValue::Bytes(value) => Self::load_bytes(value),
                ScalarValue::Unknown { type_code, bytes } => Self::load_unknown(*type_code, bytes),
            },
            Some((Value::Object(ObjType::Text), id)) => Self::load_text(store, &id),
            Some((Value::Object(ObjType::List), id)) => Self::load_list(store, &id),
            Some((Value::Object(ObjType::Map), id)) | Some((Value::Object(ObjType::Table), id)) => {
                Self::load_map(store, &id)
            }
            None => Self::load_none(),
        }
    }

    fn load_null() -> Result<Self> {
        bail_load_unexpected!("Null")
    }

    fn load_boolean(_value: &bool) -> Result<Self> {
        bail_load_unexpected!("Boolean")
    }

    fn load_int(_value: &i64) -> Result<Self> {
        bail_load_unexpected!("Int")
    }

    fn load_uint(_value: &u64) -> Result<Self> {
        bail_load_unexpected!("Uint")
    }

    fn load_f64(_value: &f64) -> Result<Self> {
        bail_load_unexpected!("F64")
    }

    fn load_str(_value: &SmolStr) -> Result<Self> {
        bail_load_unexpected!("Str")
    }

    fn load_counter() -> Result<Self> {
        bail_load_unexpected!("Counter")
    }

    fn load_timestamp(_value: &i64) -> Result<Self> {
        bail_load_unexpected!("Timestamp")
    }

    fn load_bytes(_value: &[u8]) -> Result<Self> {
        bail_load_unexpected!("Bytes")
    }

    fn load_unknown(_type_code: u8, _bytes: &[u8]) -> Result<Self> {
        bail_load_unexpected!("Unknown")
    }

    fn load_text<S: ReadStore>(_store: &S, _obj: &ObjId) -> Result<Self> {
        bail_load_unexpected!("Text")
    }

    fn load_list<S: ReadStore>(_store: &S, _obj: &ObjId) -> Result<Self> {
        bail_load_unexpected!("List")
    }

    fn load_map<S: ReadStore>(_store: &S, _obj: &ObjId) -> Result<Self> {
        bail_load_unexpected!("Map")
    }

    fn load_none() -> Result<Self> {
        bail_load_unexpected!("None")
    }

    fn dump(&self, store: &mut Store) -> Result<()> {
        self.dump_root(store)
    }

    fn dump_root(&self, _store: &mut Store) -> Result<()> {
        bail_type!("attempting to dump node of type `{type}` to store; only maps allowed at root")
    }

    fn dump_similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize>;

    fn dump_new(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()>;

    fn dump_prop(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()>;

    fn strip_ids(&mut self) -> &mut Self {
        self
    }
}

fn dump_new_scalar<S: Into<ScalarValue>>(
    store: &mut Store,
    obj: &ObjId,
    prop: Prop,
    scalar: S,
) -> Result<()> {
    match prop {
        Prop::Map(key) => store.put(obj, key, scalar)?,
        Prop::Seq(index) => store.insert(obj, index, scalar)?,
    };

    Ok(())
}

fn dump_prop_scalar<S: Into<ScalarValue>>(
    store: &mut Store,
    obj: &ObjId,
    prop: Prop,
    scalar: S,
) -> Result<()> {
    store.put(obj, prop, scalar)?;

    Ok(())
}

fn dump_new_object(store: &mut Store, obj: &ObjId, prop: Prop, obj_type: ObjType) -> Result<ObjId> {
    let id = match prop {
        Prop::Map(key) => store.put_object(obj, key, obj_type)?,
        Prop::Seq(index) => store.insert_object(obj, index, obj_type)?,
    };

    Ok(id)
}

impl Node for Null {
    fn load_null() -> Result<Self> {
        Ok(Self {})
    }

    fn dump_similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Scalar(scalar), ..)) = store.get(obj, prop)? {
            if let ScalarValue::Null = *scalar {
                return Ok(SIMILARITY_MAX);
            }
        }
        Ok(0)
    }

    fn dump_new(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        dump_new_scalar(store, obj, prop, ())
    }

    fn dump_prop(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        dump_prop_scalar(store, obj, prop, ())
    }
}

impl Node for Boolean {
    fn load_boolean(value: &bool) -> Result<Self> {
        Ok(*value)
    }

    fn dump_similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Scalar(scalar), ..)) = store.get(obj, prop)? {
            if let ScalarValue::Boolean(value) = *scalar {
                if value == *self {
                    return Ok(SIMILARITY_MAX);
                }
            }
        }
        Ok(0)
    }

    fn dump_new(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        dump_new_scalar(store, obj, prop, *self)
    }

    fn dump_prop(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        dump_prop_scalar(store, obj, prop, *self)
    }
}

impl Node for Integer {
    fn load_int(value: &i64) -> Result<Self> {
        Ok(*value)
    }

    fn dump_similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Scalar(scalar), ..)) = store.get(obj, prop)? {
            if let ScalarValue::Int(value) = *scalar {
                if value == *self {
                    return Ok(SIMILARITY_MAX);
                }
            }
        }
        Ok(0)
    }

    fn dump_new(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        dump_new_scalar(store, obj, prop, *self)
    }

    fn dump_prop(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        dump_prop_scalar(store, obj, prop, *self)
    }
}

impl Node for Number {
    fn load_f64(value: &f64) -> Result<Self> {
        Ok(*value)
    }

    fn dump_similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Scalar(scalar), ..)) = store.get(obj, prop)? {
            if let ScalarValue::F64(value) = *scalar {
                if value == *self {
                    return Ok(SIMILARITY_MAX);
                }
            }
        }
        Ok(0)
    }

    fn dump_new(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        dump_new_scalar(store, obj, prop, *self)
    }

    fn dump_prop(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        dump_prop_scalar(store, obj, prop, *self)
    }
}

impl Node for String {
    fn load_str(value: &SmolStr) -> Result<Self> {
        Ok(value.to_string())
    }

    fn dump_similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Scalar(scalar), ..)) = store.get(obj, prop)? {
            if let ScalarValue::Str(value) = scalar.as_ref() {
                if value == self {
                    return Ok(SIMILARITY_MAX);
                }
            }
        }
        Ok(0)
    }

    fn dump_new(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        dump_new_scalar(store, obj, prop, self)
    }

    fn dump_prop(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        dump_prop_scalar(store, obj, prop, self)
    }
}

impl Node for Text {
    fn load_text<S: ReadStore>(store: &S, obj: &ObjId) -> Result<Self> {
        let id = Some(obj.to_base64());
        let value = store.text(obj)?;

        Ok(Self {
            id,
            value,
            ..Default::default()
        })
    }

    fn dump_similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Object(ObjType::Text), prop_obj)) = store.get(obj, prop)? {
            if let Some(id) = self.id.as_deref() {
                if ObjId::from_base64(id)? == *obj {
                    return Ok(SIMILARITY_MAX);
                }
            }

            let value = store.text(prop_obj)?;

            let diff = TextDiffConfig::default()
                .algorithm(Algorithm::Patience)
                .timeout(Duration::from_secs(15))
                .diff_graphemes(&value, &self.value);

            return Ok((diff.ratio() * SIMILARITY_MAX as f32) as usize);
        }

        Ok(0)
    }

    fn dump_new(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        let prop_obj = dump_new_object(store, obj, prop, ObjType::Text)?;
        store.splice_text(prop_obj, 0, 0, &self.value)?;

        Ok(())
    }

    fn dump_prop(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        let existing = store.get(obj, prop.clone())?;

        if let Some((Value::Object(ObjType::Text), prop_obj)) = existing {
            // Existing property is text, so get its value, diff it with the
            // current value and apply diff operations as `splice_text` operations
            let value = store.text(&prop_obj)?;

            let diff = TextDiffConfig::default()
                .algorithm(Algorithm::Patience)
                .timeout(Duration::from_secs(15))
                .diff_graphemes(&value, &self.value);

            let mut pos = 0usize;
            for op in diff.ops() {
                match op.tag() {
                    DiffTag::Insert => {
                        let insert = &self.value[op.new_range()];
                        store.splice_text(&prop_obj, pos, 0, insert)?;
                    }
                    DiffTag::Delete => {
                        let delete = op.old_range().len();
                        store.splice_text(&prop_obj, pos, delete, "")?;
                    }
                    DiffTag::Replace => {
                        let delete = op.old_range().len();
                        let insert = &self.value[op.new_range()];
                        store.splice_text(&prop_obj, pos, delete, insert)?;
                    }
                    DiffTag::Equal => {}
                }
                pos += op.new_range().len();
            }
        } else {
            // Remove any existing property of different type
            if existing.is_some() {
                store.delete(obj, prop.clone())?;
            }

            // Insert a new `Text` object
            self.dump_new(store, obj, prop)?;
        }

        Ok(())
    }

    fn strip_ids(&mut self) -> &mut Self {
        self.id = None;

        self
    }
}

impl<T> Node for Vec<T>
where
    T: Node + std::fmt::Debug,
{
    fn load_list<S: ReadStore>(store: &S, obj: &ObjId) -> Result<Self> {
        let mut vec = Vec::new();
        for (index, ..) in store.list_range(obj, ..) {
            let node = T::load_prop(store, obj, index.into())?;
            vec.push(node);
        }

        Ok(vec)
    }

    fn dump_similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Object(ObjType::List), _prop_obj)) = store.get(obj, prop)? {}
        Ok(0)
    }

    fn dump_new(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        let prop_obj = dump_new_object(store, obj, prop, ObjType::List)?;
        for (index, node) in self.iter().enumerate() {
            node.dump_new(store, &prop_obj, index.into())?;
        }

        Ok(())
    }

    fn dump_prop(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        let existing = store.get(obj, prop.clone())?;

        if let Some((Value::Object(ObjType::List), prop_obj)) = existing {
            // TODO: correlate nodes with existing ones: create two arrays with unique id
            // (but shared on both sides) then do a patience diff to compare
            for (index, node) in self.iter().enumerate() {
                node.dump_prop(store, &prop_obj, index.into())?;
            }
        } else {
            if existing.is_some() {
                store.delete(obj, prop.clone())?;
            }
            self.dump_new(store, obj, prop)?;
        }

        Ok(())
    }

    fn strip_ids(&mut self) -> &mut Self {
        for node in self.iter_mut() {
            node.strip_ids();
        }

        self
    }
}

impl<T> Node for IndexMap<String, T>
where
    T: Node,
{
    fn load_root<S: ReadStore>(store: &S) -> Result<Self> {
        Self::load_map(store, &ROOT)
    }

    fn load_map<S: ReadStore>(store: &S, obj: &ObjId) -> Result<Self> {
        let mut map = Self::new();
        for (key, ..) in store.map_range(obj, ..) {
            let node = T::load_prop(store, obj, key.into())?;
            map.insert(key.to_string(), node);
        }

        Ok(map)
    }

    fn dump_root(&self, store: &mut Store) -> Result<()> {
        map_dump_existing(store, self, &ROOT)
    }

    fn dump_similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Object(ObjType::Map), _prop_obj)) = store.get(obj, prop)? {}
        Ok(0)
    }

    fn dump_new(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        let prop_obj = dump_new_object(store, obj, prop, ObjType::Map)?;
        for (key, node) in self {
            node.dump_new(store, &prop_obj, key.into())?;
        }

        Ok(())
    }

    fn dump_prop(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        let existing = store.get(obj, prop.clone())?;

        if let Some((Value::Object(ObjType::Map), prop_obj)) = existing {
            // Existing object is a map so dump to it
            map_dump_existing(store, self, &prop_obj)
        } else {
            // Remove any existing property of different type
            if existing.is_some() {
                store.delete(obj, prop.clone())?;
            }

            // Insert a new `Map` object
            self.dump_new(store, obj, prop)?;

            Ok(())
        }
    }

    fn strip_ids(&mut self) -> &mut Self {
        for node in self.values_mut() {
            node.strip_ids();
        }

        self
    }
}

// Dump a `IndexMap` into an existing store `Map`
fn map_dump_existing<T: Node>(
    store: &mut Store,
    map: &IndexMap<String, T>,
    obj: &ObjId,
) -> Result<()> {
    // Get all the keys for the root map of the store
    let mut keys: HashSet<String> = store.keys(obj).collect();

    // Insert or dump key that are in both map and store
    for (key, node) in map {
        node.dump_prop(store, obj, key.into())?;
        keys.remove(key);
    }

    // Remove keys that are in the store but not in map
    for key in keys {
        store.delete(obj, key.as_str())?;
    }

    Ok(())
}

impl Node for Primitive {
    fn load_null() -> Result<Self> {
        Ok(Primitive::Null(Null {}))
    }

    fn load_boolean(value: &bool) -> Result<Self> {
        Ok(Primitive::Boolean(*value))
    }

    fn load_int(value: &i64) -> Result<Self> {
        Ok(Primitive::Integer(*value))
    }

    fn load_f64(value: &f64) -> Result<Self> {
        Ok(Primitive::Number(*value))
    }

    fn load_str(value: &SmolStr) -> Result<Self> {
        Ok(Primitive::String(value.to_string()))
    }

    fn load_list<S: ReadStore>(store: &S, obj: &ObjId) -> Result<Self> {
        Ok(Primitive::Array(Array::load_list(store, obj)?))
    }

    fn load_map<S: ReadStore>(store: &S, obj: &ObjId) -> Result<Self> {
        Ok(Primitive::Object(Object::load_map(store, obj)?))
    }

    fn dump_similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        match self {
            Primitive::Null(value) => value.dump_similarity(store, obj, prop),
            Primitive::Boolean(value) => value.dump_similarity(store, obj, prop),
            Primitive::Integer(value) => value.dump_similarity(store, obj, prop),
            Primitive::Number(value) => value.dump_similarity(store, obj, prop),
            Primitive::String(value) => value.dump_similarity(store, obj, prop),
            Primitive::Array(value) => value.dump_similarity(store, obj, prop),
            Primitive::Object(value) => value.dump_similarity(store, obj, prop),
        }
    }

    fn dump_new(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        match self {
            Primitive::Null(value) => value.dump_new(store, obj, prop),
            Primitive::Boolean(value) => value.dump_new(store, obj, prop),
            Primitive::Integer(value) => value.dump_new(store, obj, prop),
            Primitive::Number(value) => value.dump_new(store, obj, prop),
            Primitive::String(value) => value.dump_new(store, obj, prop),
            Primitive::Array(value) => value.dump_new(store, obj, prop),
            Primitive::Object(value) => value.dump_new(store, obj, prop),
        }
    }

    fn dump_prop(&self, store: &mut Store, obj: &ObjId, prop: Prop) -> Result<()> {
        match self {
            Primitive::Null(value) => value.dump_prop(store, obj, prop),
            Primitive::Boolean(value) => value.dump_prop(store, obj, prop),
            Primitive::Integer(value) => value.dump_prop(store, obj, prop),
            Primitive::Number(value) => value.dump_prop(store, obj, prop),
            Primitive::String(value) => value.dump_prop(store, obj, prop),
            Primitive::Array(value) => value.dump_prop(store, obj, prop),
            Primitive::Object(value) => value.dump_prop(store, obj, prop),
        }
    }
}

trait ToBase64 {
    fn to_base64(&self) -> String;
}

impl ToBase64 for ObjId {
    fn to_base64(&self) -> String {
        BASE64_URL_SAFE_NO_PAD.encode(self.to_bytes())
    }
}

trait FromBase64: Sized {
    fn from_base64<S: AsRef<str>>(base64: S) -> Result<Self>;
}

impl FromBase64 for ObjId {
    fn from_base64<S: AsRef<str>>(base64: S) -> Result<Self> {
        let bytes = BASE64_URL_SAFE_NO_PAD.decode(base64.as_ref().as_bytes())?;
        Ok(ObjId::try_from(bytes.as_slice())?)
    }
}
