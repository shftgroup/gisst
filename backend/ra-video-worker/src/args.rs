use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct WorkerArgs {
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,
    #[clap(env)]
    pub worker_key: String,
    #[clap(env)]
    pub gisst_url: String,
}
