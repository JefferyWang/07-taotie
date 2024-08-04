mod connect;
mod describe;
mod head;
mod list;
mod schema;
mod sql;

use anyhow::Result;
use clap::Parser;
use enum_dispatch::enum_dispatch;

pub use self::connect::connect;
pub use self::describe::describe;
pub use self::head::head;
pub use self::list::list;
pub use self::schema::schema;
pub use self::sql::sql;
pub use {
    connect::{ConnectOpts, DatasetConn},
    describe::DescribeOpts,
    head::HeadOpts,
    list::ListOpts,
    schema::SchemaOpts,
    sql::SqlOpts,
};

type ReplResult = Result<Option<String>, reedline_repl_rs::Error>;

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExector)]
pub enum ReplCommand {
    #[command(
        name = "connect",
        about = "Connect to a dataset and register it to Taotie"
    )]
    Connect(ConnectOpts),
    #[command(name = "list", about = "List all registered datasets")]
    List(ListOpts),
    #[command(name = "schema", about = "Show the schema of a dataset")]
    Schema(SchemaOpts),
    #[command(name = "describe", about = "Describe a dataset")]
    Describe(DescribeOpts),
    #[command(name = "head", about = "Show the first few rows of a dataset")]
    Head(HeadOpts),
    #[command(name = "sql", about = "Show the last few rows of a dataset")]
    Sql(SqlOpts),
}
