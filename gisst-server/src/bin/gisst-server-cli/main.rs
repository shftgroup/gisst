mod args;

use clap::Parser;
use args::GISSTCli;

fn main() {
    let cli = GISSTCli::parse();

}