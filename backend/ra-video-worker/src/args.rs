use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct WorkerArgs {
    /// Whether to allow invalid or self signed certificates. Only turn on during testing!
    #[clap(long = "allow-invalid-certs", action)]
    pub allow_invalid_certs: bool,
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,
    /// Worker key for receiving and submitting tasks
    #[clap(env)]
    pub worker_key: String,
    /// GISST server URL
    #[clap(env)]
    pub gisst_url: String,
    /// A directory for storing cached native cores and their dependencies
    #[clap(env)]
    pub corefile_cache: String,
}
