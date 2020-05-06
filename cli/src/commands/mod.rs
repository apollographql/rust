use structopt::StructOpt;
use std::path::PathBuf;

/// Commands implement the Command trait, which lets us run() them
/// and get Output.
pub trait Command {
    /// Execute the command. TODO: should this return a Result?
    fn run(&self) {}
}

//#region    apollo <command>
#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case", name = "[Experimental] Apollo CLI")]
/// The [Experimental] Apollo CLI, for supporting all your graphql needs :)
pub enum Apollo {
    ///  🖨   parse and pretty print schemas to stdout
    Print(Print),
    ///  🔓  log in to apollo
    Login(Login),
}
//#endregion

//#region    ... print [-h] <files...>
pub mod print;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Print {
    #[structopt(short = "h", long)]
    /// suppress headers when printing multiple files
    pub no_headers: bool,

    #[structopt(short = "s", long)]
    /// print the syntax tree
    pub ast: bool,

    #[structopt(parse(from_os_str))]
    /// schemas to print
    pub files: Vec<PathBuf>,
}
//#endregion

//#region    ... login
pub mod login;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Login {}
//#endregion

impl Command for Apollo {
    fn run(&self) {
        match self {
            Apollo::Print(cmd) => cmd.run(),
            Apollo::Login(cmd) => cmd.run(),
        }
    }
}

impl Apollo {
    pub fn main() {
        Apollo::from_args().run();
    }
}
