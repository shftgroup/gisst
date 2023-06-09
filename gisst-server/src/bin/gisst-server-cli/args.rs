use clap::{
    Parser,
    Command,
    Args,
    Subcommand
};
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct GISSTCli {
    #[command(subcommand)]
    record_type: RecordType,

    #[arg(short,long)]
    verbose: bool,
}

#[derive(Debug, Subcommand)]
pub enum RecordType {
    Object(ObjectCommand),
    Image(ImageCommand),
    Work(WorkCommand),
    Creator(CreatorCommand),
}

#[derive(Debug, Args)]
struct ObjectCommand {
    #[command(subcommand)]
    pub command: ObjectSubcommand,
}

#[derive(Debug, Args)]
struct ImageCommand {

}

#[derive(Debug, Args)]
struct WorkCommand {

}
#[derive(Debug, Args)]
struct CreatorCommand {

}

#[derive(Debug, Subcommand)]
pub enum ObjectSubcommand {
    /// Create object(s)
    Create(CreateObject),

    /// Update an existing object
    Update(UpdateObject),

    /// Delete an existing object
    Delete(DeleteObject),

    /// Locate objects in CLI interface
    Locate(LocateObject),

    /// Export objects
    Export(ExportObject),
}

#[derive(Debug, Args)]
pub struct CreateObject {
    /// Create objects recursively if input file is directory
    #[arg(short)]
    recursive: bool,

}

#[derive(Debug, Args)]
pub struct UpdateObject {

}

#[derive(Debug, Args)]
pub struct DeleteObject{

}

#[derive(Debug, Args)]
pub struct LocateObject {

}

#[derive(Debug, Args)]
pub struct ExportObject {

}
