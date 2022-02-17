use buildpack::{libcnb::libcnb_runtime, tokio};
use buildpack_r::RBuildpack;

#[tokio::main]
async fn main() {
    libcnb_runtime(&RBuildpack);
}
