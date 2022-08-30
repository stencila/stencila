//! Methods associated with document parameters and calling documents

use std::collections::HashMap;

use common::{
    eyre::{bail, Result},
    indexmap::IndexMap,
};
use node_address::Address;
use node_pointer::{resolve, resolve_mut};
use node_validate::Validator;
use stencila_schema::{InlineContent, Node, Parameter};

use crate::{document::Document, When};

impl Document {
    /// Get the parameters of the document
    ///
    /// Returns all parameters within the root node of the document as a map, indexed by the
    /// parameter name, containing a tuple of the parameter `id`, [`Address`] as well as the [`Parameter`] itself.
    ///
    /// Used in `Document::call` and when compiling a `Call` node so that the `Call` inherits the parameters of
    /// the document as it's own `arguments`.
    pub async fn params(&mut self) -> Result<IndexMap<String, (String, Address, Parameter)>> {
        // Assemble the document to ensure its `addresses` are up to date
        self.assemble(When::Never, When::Never, When::Never).await?;

        // Collect parameters from addresses
        let addresses = self.addresses.read().await;
        let root = &*self.root.read().await;
        let params = addresses
            .iter()
            .filter_map(|(id, address)| {
                if let Ok(pointer) = resolve(root, Some(address.clone()), Some(id.clone())) {
                    if let Some(InlineContent::Parameter(param)) = pointer.as_inline() {
                        return Some((
                            param.name.clone(),
                            (id.clone(), address.clone(), param.clone()),
                        ));
                    }
                }
                None
            })
            .collect();

        Ok(params)
    }

    /// Call the document with arguments
    ///
    /// This function is similar to `Document::execute`, and indeed calls that function, but first
    /// sets the value of parameters to the `Node` with the same name in `args`.
    pub async fn call(&mut self, args: HashMap<String, Node>) -> Result<()> {
        let mut params = self.params().await?;

        {
            let root = &mut *self.root.write().await;
            for (name, value) in args {
                if let Some((id, address, param)) = params.remove(&name) {
                    if let Some(validator) = param.validator.as_deref() {
                        match validator.validate(&value) {
                            Ok(..) => {
                                if let Ok(mut pointer) = resolve_mut(root, Some(address), Some(id))
                                {
                                    if let Some(InlineContent::Parameter(param)) =
                                        pointer.as_inline_mut()
                                    {
                                        param.value = Some(Box::new(value));
                                    }
                                }
                            }
                            Err(error) => bail!(
                                "While attempting to parse document parameter `{}`: {}",
                                name,
                                error
                            ),
                        }
                    }
                } else {
                    bail!("Document does not have a parameter named `{}`", name)
                }
            }
        }

        self.execute(When::Never, None, None, None).await?;

        Ok(())
    }

    /// Call the document with arguments having string values
    ///
    /// This calls, the `Document::call` function but first parses each string value into
    /// a `Node` using the corresponding validator.
    /// 
    /// Used to call a document with strings taken from the command line.
    pub async fn call_strings(&mut self, args: HashMap<String, String>) -> Result<()> {
        let mut params = self.params().await?;
        let mut args_parsed = HashMap::new();
        for (name, value) in args {
            if let Some((_id, _address, param)) = params.remove(&name) {
                if let Some(validator) = param.validator.as_deref() {
                    match validator.parse(&value) {
                        Ok(value) => {
                            args_parsed.insert(name, value);
                        }
                        Err(error) => bail!(
                            "While attempting to parse document parameter `{}`: {}",
                            name,
                            error
                        ),
                    }
                }
            } else {
                bail!("Document does not have a parameter named `{}`", name)
            }
        }

        self.call(args_parsed).await?;

        Ok(())
    }
}
