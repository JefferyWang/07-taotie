use clap::{ArgMatches, Parser};

use crate::ReplContext;

use super::{ReplCommand, ReplResult};

#[derive(Debug, Parser)]
pub struct HeadOpts {
    #[clap(short, long, help = "The name of the dataset")]
    pub name: String,
    #[clap(short, long, help = "The number of rows to show")]
    pub n: Option<usize>,
}

pub fn head(args: ArgMatches, ctx: &mut ReplContext) -> ReplResult {
    let name = args
        .get_one::<String>("name")
        .expect("expect name")
        .to_string();
    let n = args.get_one::<usize>("n").copied().unwrap_or(5);

    let cmd = HeadOpts::new(name, n).into();
    ctx.send(cmd);

    Ok(None)
}

impl From<HeadOpts> for ReplCommand {
    fn from(opts: HeadOpts) -> Self {
        ReplCommand::Head(opts)
    }
}

impl HeadOpts {
    pub fn new(name: String, n: usize) -> Self {
        Self { name, n: Some(n) }
    }
}
