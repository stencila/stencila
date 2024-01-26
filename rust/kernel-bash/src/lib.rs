use kernel_micro::{
    common::eyre::Result, format::Format, Kernel, KernelAvailability, KernelForking,
    KernelInstance, Microkernel,
};

/// A kernel for executing Bash code locally
pub struct BashKernel;

impl Kernel for BashKernel {
    fn id(&self) -> String {
        "bash-micro".to_string()
    }

    fn availability(&self) -> KernelAvailability {
        Microkernel::microkernel_availability(self)
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::Bash, Format::Shell]
    }

    fn supports_forking(&self) -> KernelForking {
        KernelForking::No
    }

    fn create_instance(&self) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(Microkernel::microkernel_instance(self)?))
    }
}

impl Microkernel for BashKernel {
    fn executable_name(&self) -> String {
        "bash".to_string()
    }

    fn microkernel_script(&self) -> String {
        include_str!("kernel.bash").to_string()
    }
}

#[cfg(test)]
mod tests {
    use kernel_micro::{
        common::{eyre::bail, tokio},
        schema::{Array, Node, Object, Paragraph, Primitive},
    };

    use super::*;

    /// Create and start a new kernel instance if Bash is available
    async fn bash_kernel() -> Result<Option<Box<dyn KernelInstance>>> {
        let kernel = BashKernel {};
        match kernel.availability() {
            KernelAvailability::Available => {
                let mut instance = kernel.create_instance()?;
                instance.start_here().await?;
                Ok(Some(instance))
            }
            _ => Ok(None),
        }
    }

    /// Test execute tasks that just generate outputs of different types
    #[tokio::test]
    async fn outputs() -> Result<()> {
        let Some(mut kernel) = bash_kernel().await? else {
            return Ok(())
        };

        // Print a string
        let (outputs, messages) = kernel.execute("echo 'Hello'").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::String("Hello\n".to_string())]);

        // Print a string in double quotes
        let (outputs, messages) = kernel.execute("printf \"Hello\"").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::String("Hello".to_string())]);

        // Print a number
        let (outputs, messages) = kernel.execute("echo '1.23'").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::Number(1.23)]);

        // Print an array
        let (outputs, messages) = kernel.execute("echo '[1,2,3]'").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![Node::Array(Array::from([
                Primitive::Integer(1),
                Primitive::Integer(2),
                Primitive::Integer(3)
            ]))]
        );

        // Print an object
        let (outputs, messages) = kernel.execute(r#"echo '{"a":1, "b":2.3}'"#).await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![Node::Object(Object::from([
                ("a", Primitive::Integer(1)),
                ("b", Primitive::Number(2.3))
            ]))]
        );

        // Print a content node type
        let (outputs, messages) = kernel
            .execute(r#"echo '{"type":"Paragraph", "content":[]}'"#)
            .await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::Paragraph(Paragraph::new(vec![]))]);

        Ok(())
    }

    /// Test execute tasks that set and use state within the kernel
    #[tokio::test]
    async fn state() -> Result<()> {
        let Some(mut kernel) = bash_kernel().await? else {
            return Ok(())
        };

        // Set some variables
        let (outputs, messages) = kernel.execute("a=1\nb=2").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![]);

        // Use the variables
        let (outputs, messages) = kernel.execute("echo $((a + b))").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::Integer(3)]);

        Ok(())
    }

    /// Test evaluate tasks
    #[tokio::test]
    async fn evaluate() -> Result<()> {
        let Some(mut kernel) = bash_kernel().await? else {
                return Ok(())
            };

        let (outputs, messages) = kernel.evaluate("1 + 3").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::Integer(4)]);

        Ok(())
    }

    /// Test execute tasks that intentionally generate error messages
    #[tokio::test]
    async fn messages() -> Result<()> {
        let Some(mut kernel) = bash_kernel().await? else {
            return Ok(())
        };

        // Syntax error
        let (outputs, messages) = kernel.execute("if").await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(outputs, vec![]);

        // Runtime error
        let (outputs, messages) = kernel.execute("foo").await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(outputs, vec![]);

        Ok(())
    }

    /// Test execution tasks that involve additional escaping
    #[tokio::test]
    async fn escaping() -> Result<()> {
        let Some(mut kernel) = bash_kernel().await? else {
            return Ok(())
        };

        // Test escaping of percent signs in commands
        let (outputs, messages) = kernel.execute("date +%s").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs.len(), 1);

        match outputs.first() {
            Some(Node::Integer(timestamp)) => assert!(*timestamp > 1600000000),
            _ => bail!("Expected an integer output"),
        }

        Ok(())
    }
}
