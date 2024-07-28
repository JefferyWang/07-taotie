mod connect;
mod describe;
mod head;
mod list;
mod sql;

use anyhow::Result;
use clap::Parser;
use connect::ConnectOpts;
use describe::DescribeOpts;
use head::HeadOpts;
use sql::SqlOpts;

pub use self::connect::connect;
pub use self::describe::describe;
pub use self::head::head;
pub use self::list::list;
pub use self::sql::sql;

type ReplResult = Result<Option<String>, reedline_repl_rs::Error>;

#[derive(Debug, Parser)]
pub enum ReplCommand {
    #[command(
        name = "connect",
        about = "Connect to a dataset and register it to Taotie"
    )]
    Connect(ConnectOpts),
    #[command(name = "list", about = "List all registered datasets")]
    List,
    #[command(name = "describe", about = "Describe a dataset")]
    Describe(DescribeOpts),
    #[command(name = "head", about = "Show the first few rows of a dataset")]
    Head(HeadOpts),
    #[command(name = "sql", about = "Show the last few rows of a dataset")]
    Sql(SqlOpts),
}
