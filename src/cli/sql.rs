use super::ReplResult;
use crate::{Backend, CmdExector, ReplContext, ReplDisplay, ReplMsg};
use clap::{ArgMatches, Parser};

#[derive(Debug, Parser)]
pub struct SqlOpts {
    #[clap(help = "The SQL query to execute")]
    pub query: String,
}

pub fn sql(args: ArgMatches, ctx: &mut ReplContext) -> ReplResult {
    let query = args
        .get_one::<String>("query")
        .expect("expect query")
        .to_string();

    let cmd = SqlOpts::new(query);
    let (msg, rx) = ReplMsg::new(cmd);
    Ok(ctx.send(msg, rx))
}

impl SqlOpts {
    pub fn new(query: String) -> Self {
        Self { query }
    }
}

impl CmdExector for SqlOpts {
    async fn execute<T: Backend>(self, backend: &mut T) -> anyhow::Result<String> {
        let df = backend.sql(&self.query).await?;
        df.display().await
    }
}
