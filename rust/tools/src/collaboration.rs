use crate::{environments::Devbox, Tool, ToolType};

pub struct Git;

impl Tool for Git {
    fn name(&self) -> &'static str {
        "git"
    }

    fn url(&self) -> &'static str {
        "https://git-scm.com/"
    }

    fn description(&self) -> &'static str {
        "Distributed version control system"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Collaboration
    }

    fn install_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Devbox)]
    }
}
