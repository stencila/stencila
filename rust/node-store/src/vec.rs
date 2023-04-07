use common::eyre::Result;

use crate::prelude::*;

impl<T> Read for Vec<T>
where
    T: Read + std::fmt::Debug,
{
    fn load_list<S: ReadStore>(store: &S, obj: &ObjId) -> Result<Self> {
        let mut vec = Vec::new();
        for (index, ..) in store.list_range(obj, ..) {
            let node = T::load_prop(store, obj, index.into())?;
            vec.push(node);
        }

        Ok(vec)
    }
}

impl<T> Write for Vec<T>
where
    T: Write + std::fmt::Debug,
{
    fn similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Object(ObjType::List), _prop_obj)) = store.get(obj, prop)? {}
        Ok(0)
    }

    fn dump_new(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()> {
        let prop_obj = dump_new_object(store, obj, prop, ObjType::List)?;
        for (index, node) in self.iter().enumerate() {
            node.dump_new(store, &prop_obj, index.into())?;
        }

        Ok(())
    }

    fn dump_prop(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()> {
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
}
