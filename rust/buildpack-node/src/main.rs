use buildpack::{libcnb::libcnb_runtime, tokio};
use buildpack_node::NodeBuildpack;

#[tokio::main]
async fn main() {
    libcnb_runtime(&NodeBuildpack);
}
