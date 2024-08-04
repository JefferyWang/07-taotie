use clap::{ArgMatches, Parser};

use crate::{Backend, CmdExector, ReplContext, ReplDisplay, ReplMsg};

use super::ReplResult;

#[derive(Debug, Parser)]
pub struct HeadOpts {
    #[clap(help = "The name of the dataset")]
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

    let cmd = HeadOpts::new(name, n);
    let (msg, rx) = ReplMsg::new(cmd);
    Ok(ctx.send(msg, rx))
}

impl HeadOpts {
    pub fn new(name: String, n: usize) -> Self {
        Self { name, n: Some(n) }
    }
}

impl CmdExector for HeadOpts {
    async fn execute<T: Backend>(self, backend: &mut T) -> anyhow::Result<String> {
        let df = backend.head(&self.name, self.n.unwrap_or(5)).await?;
        df.display().await
    }
}
