#[macro_export]
macro_rules! strip_code {
    ($self:expr) => {
        $self.code = String::new();
        $self.programming_language = String::new();
        $self.guess_language = None;
    };
}

#[macro_export]
macro_rules! strip_execution {
    ($self:expr) => {
        $self.options.execution_auto = None;
        $self.options.compilation_digest = None;
        $self.options.execution_digest = None;
        $self.options.execution_count = None;
        $self.options.execution_dependants = None;
        $self.options.execution_dependencies = None;
        $self.options.execution_duration = None;
        $self.options.execution_ended = None;
        $self.options.execution_kernel = None;
        $self.options.execution_required = None;
        $self.options.execution_status = None;
        $self.options.execution_tags = None;
    };
}
