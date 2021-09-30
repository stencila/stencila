use crate::{
    graphs::{Relation, Resource},
    utils::uuids,
};
use chrono::{DateTime, Utc};
use derive_more::{Deref, DerefMut};
use enum_dispatch::enum_dispatch;
use eyre::{bail, eyre, Result};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::{hash_map::Entry, BTreeMap, HashMap};
use stencila_schema::Node;
use validator::Contains;

type KernelId = String;

#[enum_dispatch]
pub trait KernelTrait {
    /// Get the name of the kernel's programming language, and/or
    /// check that it is able to execute a given language.
    ///
    /// If a `language` identifier is supplied, e.g. `Some("py")`, and the kernel
    /// can execute that language, should return the canonical name of the language
    /// e.g. `Ok("python3")`. If the language can not execute the language should
    /// return a `IncompatibleLanguage` error.
    fn language(&self, language: Option<String>) -> Result<String>;

    /// Get a symbol from the kernel
    fn get(&self, name: &str) -> Result<Node>;

    /// Set a symbol in the kernel
    fn set(&mut self, name: &str, value: Node) -> Result<()>;

    /// Execute some code in the kernel
    fn exec(&mut self, code: &str) -> Result<Vec<Node>>;
}

mod default;
use default::*;

mod calc;
use calc::*;

#[enum_dispatch(KernelTrait)]
#[derive(Debug, Clone, JsonSchema, Serialize)]
#[serde(tag = "type")]
pub enum Kernel {
    Default(DefaultKernel),
    Calc(CalcKernel),
}

#[derive(Debug, Clone, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct SymbolInfo {
    /// The type of the object that the symbol refers to (e.g `Number`, `Function`)
    ///
    /// Should be used as a hint only, to the underlying, native type of the symbol.
    kind: String,

    /// The home kernel of the symbol
    ///
    /// The home kernel of a symbol is the kernel that it was last assigned in.
    /// As such, a symbol's home kernel can change, although this is discouraged.
    home: KernelId,

    /// The time that the symbol was last assigned in the home kernel
    ///
    /// A symbol is considered assigned when  a `CodeChunk` with an `Assign` relation
    /// to the symbol is executed or the `kernel.set` method is called.
    assigned: DateTime<Utc>,

    /// The time that the symbol was last mirrored to other kernels
    ///
    /// A timestamp is recorded for each time that a symbol is mirrored to another
    /// kernel. This allows unnecessary mirroring to be avoided if the symbol has
    /// not been assigned since it was last mirrored to that kernel.
    mirrored: HashMap<KernelId, DateTime<Utc>>,
}

impl SymbolInfo {
    pub fn new(kind: &str, kernel_id: &str) -> Self {
        SymbolInfo {
            kind: kind.into(),
            home: kernel_id.into(),
            assigned: Utc::now(),
            mirrored: HashMap::new(),
        }
    }
}

/// A map of [`KernelId`] to [`Kernel`]
///
/// A `newtype` that exists solely to provide a `Result` (rather than `<Option>`
/// when getting a kernel by id.
#[derive(Debug, Clone, Default, Deref, DerefMut, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
struct KernelMap(BTreeMap<KernelId, Kernel>);

impl KernelMap {
    /// Get a reference to a kernel
    fn get(&self, kernel_id: &str) -> Result<&Kernel> {
        (**self)
            .get(kernel_id)
            .ok_or_else(|| eyre!("Unknown kernel `{}`", kernel_id))
    }

    /// Get a mutable reference to a kernel
    fn get_mut(&mut self, kernel_id: &str) -> Result<&mut Kernel> {
        (**self)
            .get_mut(kernel_id)
            .ok_or_else(|| eyre!("Unknown kernel `{}`", kernel_id))
    }
}

#[derive(Debug, Clone, Default, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct KernelSpace {
    /// The kernels in the document kernel space
    kernels: KernelMap,

    /// The symbols in the document kernel space
    symbols: HashMap<String, SymbolInfo>,
}

impl KernelSpace {
    /// Get a list of symbols in the kernel space
    ///
    /// Mainly for inspection, in the future may return a list with
    /// more information e.g. the type of symbol.
    pub fn symbols(&self) -> HashMap<String, SymbolInfo> {
        self.symbols.clone()
    }

    /// Get a symbol from the kernel space
    pub fn get(&self, name: &str) -> Result<Node> {
        let symbol_info = self
            .symbols
            .get(name)
            .ok_or_else(|| eyre!("Unknown symbol `{}`", name))?;

        let kernel = self.kernels.get(&symbol_info.home)?;
        kernel.get(name)
    }

    /// Set a symbol in the kernel space
    pub fn set(&mut self, name: &str, value: Node, language: &str) -> Result<()> {
        let kernel_id = self.ensure_kernel(language)?;
        tracing::debug!("Setting symbol `{}` in kernel `{}`", name, kernel_id);

        let kernel = self.kernels.get_mut(&kernel_id)?;
        kernel.set(name, value)?;

        match self.symbols.entry(name.to_string()) {
            Entry::Occupied(mut occupied) => {
                let info = occupied.get_mut();
                info.home = kernel_id;
                info.assigned = Utc::now();
            }
            Entry::Vacant(vacant) => {
                vacant.insert(SymbolInfo::new("", &kernel_id));
            }
        }

        Ok(())
    }

    /// Execute some code in the kernel space
    ///
    /// Symbols that the code uses, but have a different home kernel, are mirrored to the kernel.
    pub fn exec(
        &mut self,
        code: &str,
        language: &str,
        relations: Option<Vec<(Relation, Resource)>>,
    ) -> Result<Vec<Node>> {
        // Determine the kernel to execute in
        let kernel_id = self.ensure_kernel(language)?;
        tracing::debug!("Executing code in kernel `{}`", kernel_id);

        // Mirror used symbols into the kernel
        if let Some(relations) = &relations {
            for relation in relations {
                let name = if let (Relation::Use(..), Resource::Symbol(symbol)) = relation {
                    if self.symbols.has_element(&symbol.name) {
                        &symbol.name
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                let SymbolInfo {
                    home,
                    assigned,
                    mirrored,
                    ..
                } = self
                    .symbols
                    .get_mut(name)
                    .ok_or_else(|| eyre!("Unknown symbol `{}`", name))?;

                // Skip if home is the target kernel
                if *home == kernel_id {
                    continue;
                }

                // Skip if already mirrored since last assigned
                if let Some(mirrored) = mirrored.get(&kernel_id) {
                    if mirrored >= assigned {
                        continue;
                    }
                }

                tracing::debug!(
                    "Mirroring symbol `{}` from kernel `{}` to kernel `{}`",
                    name,
                    home,
                    kernel_id
                );

                let home_kernel = self.kernels.get(home)?;
                let value = home_kernel.get(name)?;

                let mirror_kernel = self.kernels.get_mut(&kernel_id)?;
                mirror_kernel.set(name, value)?;

                match mirrored.entry(kernel_id.clone()) {
                    Entry::Occupied(mut occupied) => {
                        let datetime = occupied.get_mut();
                        *datetime = Utc::now();
                    }
                    Entry::Vacant(vacant) => {
                        vacant.insert(Utc::now());
                    }
                }
            }
        }

        // Execute the code
        let kernel = self.kernels.get_mut(&kernel_id)?;
        let nodes = kernel.exec(code)?;

        // Record symbols assigned in kernel
        if let Some(relations) = relations {
            for relation in relations {
                let (name, kind) =
                    if let (Relation::Assign(..), Resource::Symbol(symbol)) = relation {
                        (symbol.name, symbol.kind)
                    } else {
                        continue;
                    };

                match self.symbols.entry(name) {
                    Entry::Occupied(mut occupied) => {
                        let info = occupied.get_mut();
                        info.home = kernel_id.clone();
                        info.assigned = Utc::now();
                    }
                    Entry::Vacant(vacant) => {
                        vacant.insert(SymbolInfo::new(&kind, &kernel_id));
                    }
                }
            }
        }

        Ok(nodes)
    }

    /// Ensure that a kernel exists for a language
    ///
    /// Returns a tuple of the kernel's canonical language name and id.
    fn ensure_kernel(&mut self, language: &str) -> Result<KernelId> {
        // Is there already a kernel capable of executing the language?
        for (kernel_id, kernel) in self.kernels.iter_mut() {
            if kernel.language(Some(language.to_string())).is_ok() {
                return Ok(kernel_id.clone());
            }
        }

        // If unable to set in an existing kernel then start a new kernel
        // for the language.
        let kernel = match language {
            "calc" => Kernel::Calc(CalcKernel::new()),
            "none" | "" => Kernel::Default(DefaultKernel::new()),
            _ => bail!(
                "Unable to create an execution kernel for language `{}`",
                language
            ),
        };
        let kernel_id = uuids::generate(uuids::Family::Kernel);
        self.kernels.insert(kernel_id.clone(), kernel);

        Ok(kernel_id)
    }
}
