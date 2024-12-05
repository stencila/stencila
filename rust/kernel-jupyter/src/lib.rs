use std::{net::Ipv4Addr, path::Path, process::Stdio};

use runtimelib::{
    create_client_iopub_connection, create_client_shell_connection, list_kernelspecs,
    ClientIoPubConnection, ClientShellConnection, Connection, ConnectionInfo, ExecuteRequest,
    ExecutionState, Header, JupyterMessage, JupyterMessageContent, KernelspecDir, Media, MediaType,
};

use kernel::{
    common::{
        async_trait::async_trait,
        eyre::{bail, eyre, Result},
        itertools::Itertools,
        serde_json,
        tokio::{
            self,
            fs::{create_dir_all, write},
            process::Child,
            sync::broadcast::{self, Receiver},
        },
        tracing,
        uuid::Uuid,
    },
    format::Format,
    generate_id,
    schema::{
        CodeLocation, ExecutionMessage, MessageLevel, Node, Null, SoftwareApplication,
        SoftwareApplicationOptions,
    },
    Kernel, KernelForks, KernelInstance, KernelInterrupt, KernelKill, KernelProvider,
    KernelTerminate, KernelType,
};

const NAME: &str = "jupyter";

/// A Jupyter kernel
#[derive(Default)]
pub struct JupyterKernel;

impl JupyterKernel {
    /// Get the kernelspecs available on the current machine
    async fn kernelspecs(&self) -> Vec<KernelspecDir> {
        runtimelib::list_kernelspecs().await
    }
}

impl Kernel for JupyterKernel {
    fn name(&self) -> String {
        NAME.to_string()
    }

    fn r#type(&self) -> KernelType {
        KernelType::Programming
    }

    fn provider(&self) -> KernelProvider {
        KernelProvider::Environment
    }

    fn supports_languages(&self) -> Vec<Format> {
        // TODO: Consider what should be returned here
        vec![]
    }

    fn supports_forks(&self) -> KernelForks {
        KernelForks::No
    }

    fn supports_interrupt(&self) -> KernelInterrupt {
        KernelInterrupt::Yes
    }

    fn supports_terminate(&self) -> KernelTerminate {
        KernelTerminate::Yes
    }

    fn supports_kill(&self) -> KernelKill {
        KernelKill::Yes
    }

    fn create_instance(&self) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(JupyterKernelInstance::new()))
    }
}

pub struct JupyterKernelInstance {
    /// The unique id of the kernel instance
    id: String,

    /// The Jupyter kernel child process
    process: Option<Child>,

    /// Connection to the kernel's shell socket
    shell_socket: Option<ClientShellConnection>,

    /// Broadcast receiver to forward on the kernel's iopub messages
    iopub_receiver: Option<Receiver<JupyterMessage>>,
}

impl JupyterKernelInstance {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            id: generate_id(NAME),
            process: None,
            shell_socket: None,
            iopub_receiver: None,
        }
    }
}

#[async_trait]
impl KernelInstance for JupyterKernelInstance {
    fn id(&self) -> &str {
        &self.id
    }

    async fn start(&mut self, directory: &Path) -> Result<()> {
        let ip = [127, 0, 0, 1].into();

        let ports = runtimelib::peek_ports(ip, 5)
            .await
            .map_err(|error| eyre!(error))?;
        if ports.len() != 5 {
            bail!("Unable to get five ports")
        }

        let connection_info = ConnectionInfo {
            transport: "tcp".to_string(),
            ip: ip.to_string(),
            stdin_port: ports[0],
            control_port: ports[1],
            hb_port: ports[2],
            shell_port: ports[3],
            iopub_port: ports[4],
            signature_scheme: "hmac-sha256".to_string(),
            key: Uuid::new_v4().to_string(),
            kernel_name: Some(self.id.to_string()),
        };

        let runtime_dir = runtimelib::dirs::runtime_dir();
        create_dir_all(&runtime_dir).await?;

        let connection_path = runtime_dir.join([&self.id, ".json"].concat());
        write(&connection_path, serde_json::to_string(&connection_info)?).await?;

        let kernelspecs = list_kernelspecs().await;
        let kernelspec = kernelspecs
            .iter()
            .find(|k| k.kernel_name.eq("python"))
            .ok_or(eyre!("Python kernel not found"))?;

        let process = kernelspec
            .clone()
            .command(&connection_path, None, None)
            .map_err(|error| eyre!(error))?
            .current_dir(directory)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;
        self.process = Some(process);

        let session_id = Uuid::new_v4().to_string();

        let shell_socket = create_client_shell_connection(&connection_info, &session_id)
            .await
            .map_err(|error| eyre!(error))?;
        self.shell_socket = Some(shell_socket);

        let (sender, receiver) = broadcast::channel(256);
        let mut iopub_socket = create_client_iopub_connection(&connection_info, "", &session_id)
            .await
            .map_err(|error| eyre!(error))?;
        tokio::spawn(async move {
            loop {
                match iopub_socket.read().await {
                    Ok(message) => {
                        if let Err(error) = sender.send(message) {
                            tracing::error!("While sending iopub message: {}", error);
                        }
                    }
                    Err(error) => {
                        tracing::error!("While receiving iopub message: {}", error);
                        break;
                    }
                }
            }
        });
        self.iopub_receiver = Some(receiver);

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(process) = &mut self.process {
            process.start_kill()?;
        }

        Ok(())
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        let (Some(shell_socket), Some(iopub_receiver)) =
            (&mut self.shell_socket, &mut self.iopub_receiver)
        else {
            bail!("Kernel is not yet started")
        };

        tracing::trace!("Executing in Jupyter kernel");

        let execute_request: JupyterMessage = ExecuteRequest::new(code.to_string()).into();
        let execute_request_id = execute_request.header.msg_id.clone();

        let mut iopub_receiver = iopub_receiver.resubscribe();
        let task = tokio::spawn(async move {
            loop {
                match iopub_receiver.recv().await {
                    Ok(JupyterMessage {
                        content: JupyterMessageContent::ExecuteResult(result),
                        parent_header: Some(Header { msg_id, .. }),
                        ..
                    }) => {
                        if msg_id == execute_request_id {
                            return (vec![node_from_media(result.data)], vec![]);
                        }
                    }
                    Err(error) => {
                        return (
                            vec![],
                            vec![ExecutionMessage::new(
                                MessageLevel::Error,
                                format!("While receiving Jupyter message: {error}"),
                            )],
                        );
                    }
                    _ => {}
                }
            }
        });

        shell_socket
            .send(execute_request)
            .await
            .map_err(|error| eyre!(error))?;

        let (nodes, messages) = task.await?;

        Ok((nodes, messages))
    }

    async fn evaluate(&mut self, code: &str) -> Result<(Node, Vec<ExecutionMessage>)> {
        tracing::trace!("Evaluating in Jupyter kernel");

        Ok((Node::Null(Null), vec![]))
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        tracing::trace!("Getting Jupyter runtime info");

        Ok(SoftwareApplication {
            // TODO: Get information from the kernel spec
            name: "Jupyter Kernel".to_string(),
            options: Box::new(SoftwareApplicationOptions {
                operating_system: Some(std::env::consts::OS.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}

fn node_from_media(media: Media) -> Node {
    for media_type in media.content {
        match media_type {
            MediaType::Plain(text) => return Node::String(text),
            _ => {}
        }
    }

    Node::Null(Null)
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use kernel::{common::tokio, schema::Node};

    use super::*;
}
