use common::{
    async_trait::async_trait,
    serde_json,
};

use graph_triples::{
    execution_digest_from_content_semantics,
};


use node_validate::Validator;
use stencila_schema::{
    ExecutionDigest, Node, Parameter,
};

use crate::{
    executable::{Executable},
};

/// Get the value for a parameter
///
/// Uses the parameters `value`, falling back to it's `default`, falling back
/// to a default based on the validator.
fn parameter_value(param: &Parameter) -> Node {
    param
        .value
        .as_deref()
        .or(param.default.as_deref())
        .cloned()
        .or_else(|| {
            param
                .validator
                .as_ref()
                .map(|validator| validator.default_())
        })
        .unwrap_or_else(|| Node::String(String::new()))
}

/// Generate a `ResourceDigest` for a parameter
///
/// The content string is the JSON representation of the parameter value
/// and semantic string adds the parameter name
fn parameter_digest(param: &Parameter, value: &Node) -> ExecutionDigest {
    let content_str = serde_json::to_string(&value).unwrap_or_default();
    let semantic_str = [param.name.as_str(), content_str.as_str()].concat();
    execution_digest_from_content_semantics(&content_str, &semantic_str)
}

#[async_trait]
impl Executable for Parameter {
    /// Compile a `Parameter` node
    ///
    /// Adds an `Assign` relation to the compilation context with the name and kind of value
    /// of the parameter. Uses `Format::Json` to indicate that the parameter will be set in the "store kernel".
    /// Previously we used the context language to set the parameter directly in the kernel
    /// corresponding to the language of preceding code chunks (if any). But always storing in the
    /// core is more reliable e.g. some kernels may not support more complicated types like
    /// `Timestamp`s and `DateTime`s.
    ///
    /// By definition, a `Parameter` is always "impure" (has a side effect).
    #[cfg(ignore)]
    async fn compile(&self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        let id = ensure_id!(self, "pa", context);

        // Add a resource for the parameter itself
        let resource = resources::code(&context.path, id, "Parameter", Format::Json);
        let kind = match self.validator.as_deref() {
            Some(ValidatorTypes::BooleanValidator(..)) => "Boolean",
            Some(ValidatorTypes::EnumValidator(..)) => "Enum",
            Some(ValidatorTypes::IntegerValidator(..)) => "Integer",
            Some(ValidatorTypes::NumberValidator(..)) => "Number",
            Some(ValidatorTypes::StringValidator(..)) => "String",
            Some(ValidatorTypes::DateValidator(..)) => "Date",
            Some(ValidatorTypes::TimeValidator(..)) => "Time",
            Some(ValidatorTypes::DateTimeValidator(..)) => "DateTime",
            Some(ValidatorTypes::TimestampValidator(..)) => "Timestamp",
            Some(ValidatorTypes::DurationValidator(..)) => "Duration",
            Some(ValidatorTypes::TupleValidator(..)) => "Tuple",
            Some(ValidatorTypes::ArrayValidator(..)) => "Array",
            _ => "",
        };
        let symbol = resources::symbol(&context.path, &self.name, kind);
        let mut relations = vec![(relations::assigns(NULL_RANGE), symbol)];

        // If the parameter is derived then add relation with the symbol it is derived from
        if let Some(from) = &self.derived_from {
            let from = resources::symbol(&context.path, from, "");
            relations.push((relations::derives(), from));
        }

        let value = parameter_value(self);
        let compile_digest = parameter_digest(self, &value);

        let resource_info = ResourceInfo::new(
            resource,
            Some(relations),
            None,
            Some(false), // Always impure because affects the kernel space,
            None,
            Some(compile_digest),
            self.execute_digest.clone(),
            None,
        );
        context.resource_infos.push(resource_info);

        Ok(())
    }

    #[cfg(ignore)]
    async fn execute_begin(
        &mut self,
        resource_info: &ResourceInfo,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        _is_fork: bool,
    ) -> Result<Option<TaskInfo>> {
        let id = assert_id!(self)?;
        tracing::trace!("Executing `Parameter` `{id}`");

        let mut errors = Vec::new();

        // If parameter is derived then start by ensuring derived properties
        // are up to date
        if let Some(from) = &self.derived_from {
            match kernel_space.derive("parameter", from).await {
                Ok((_, nodes)) => {
                    if let Some(Node::Parameter(parameter)) = nodes.first() {
                        self.validator = parameter.validator.clone();
                        self.default = parameter.default.clone();
                    } else {
                        // This is not a user error, but a kernel implementation error so bail
                        bail!("Expected to get a parameter from derive call")
                    }
                }
                Err(error) => errors.push(CodeError {
                    error_type: Some(Box::new("DeriveError".to_string())),
                    error_message: error.to_string(),
                    ..Default::default()
                }),
            }
        }

        // Set the parameter value in the kernel and on the parameter itself
        let value = parameter_value(self);
        let kernel_id = kernel_space
            .set(&self.name, value.clone(), kernel_selector)
            .await?;
        self.value = Some(Box::new(value));

        // Update both `compile_digest` and `execute_digest` to the compile digest
        let digest = resource_info.compile_digest.clone();
        self.compile_digest = digest.clone();
        self.execute_digest = digest;

        self.execution_required = Some(ExecutionRequired::No);
        self.execution_kernel = Some(Box::new(kernel_id));
        self.execution_count = Some(self.execution_count.unwrap_or_default() + 1);
        self.errors = if errors.is_empty() {
            None
        } else {
            Some(errors)
        };

        Ok(None)
    }
}
