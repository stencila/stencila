use buildpack::{libcnb::libcnb_runtime, tokio};
use buildpack_python::PythonBuildpack;

#[tokio::main]
async fn main() {
    libcnb_runtime(&PythonBuildpack);
}
