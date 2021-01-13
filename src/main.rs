#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() {
    use std::process::exit;

    #[cfg(feature = "cli")]
    exit(stencila::cli::cli(std::env::args().collect()).await);

    #[cfg(all(feature = "serve", not(feature = "cli")))]
    exit(match stencila::serve::serve(None, None, None) {
        Ok(_) => 0,
        Err(error) => {
            eprintln!("Error: {}", error);
            1
        }
    });

    #[cfg(not(any(feature = "serve", feature = "cli")))]
    {
        eprintln!("Warning: neither `cli` nor `serve` features enabled, nothing to do.");
        exit(0)
    }
}
