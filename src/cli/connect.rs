use clap::{ArgMatches, Parser};
use datafusion::datasource::file_format::file_compression_type::FileCompressionType;

use crate::{CmdExector, ReplContext, ReplMsg};

use super::ReplResult;

#[derive(Debug, Clone)]
pub enum DatasetConn {
    Postgres(String),
    Csv(FileOpts),
    Parquet(String),
    NdJson(FileOpts),
}
#[derive(Debug, Clone)]
pub struct FileOpts {
    pub filename: String,
    pub ext: String,
    pub compression: FileCompressionType,
}

#[derive(Debug, Parser)]
pub struct ConnectOpts {
    #[arg(value_parser = verify_conn_str, help = "The connection string to the dataset, could be postgres of local file (support: csv, parquet, json)")]
    pub conn: DatasetConn,
    #[clap(short, long, help = "If database, the name of the table")]
    pub table: Option<String>,
    #[clap(short, long, help = "The name of the dataset")]
    pub name: String,
}

pub fn connect(args: ArgMatches, ctx: &mut ReplContext) -> ReplResult {
    let conn = args
        .get_one::<DatasetConn>("conn")
        .expect("expect conn_str")
        .to_owned();
    let table = args.get_one::<String>("table").map(|s| s.to_string());
    let name = args
        .get_one::<String>("name")
        .expect("expect name")
        .to_string();

    let cmd = ConnectOpts::new(conn, table, name);
    let (msg, rx) = ReplMsg::new(cmd);
    Ok(ctx.send(msg, rx))
}

impl ConnectOpts {
    pub fn new(conn: DatasetConn, table: Option<String>, name: String) -> Self {
        Self { conn, table, name }
    }
}

fn verify_conn_str(s: &str) -> Result<DatasetConn, String> {
    let conn_str = s.to_lowercase();
    if conn_str.starts_with("postgres://") {
        return Ok(DatasetConn::Postgres(conn_str));
    }
    let exts = conn_str.split('.').rev().collect::<Vec<_>>();
    let len = exts.len();
    let mut exts = exts.into_iter().take(len - 1);
    let ext1 = exts.next();
    let ext2 = exts.next();
    match (ext1, ext2) {
        (Some(ext1), Some(ext2)) => {
            let compression = match ext1 {
                "gz" => FileCompressionType::GZIP,
                "bz2" => FileCompressionType::BZIP2,
                "xz" => FileCompressionType::XZ,
                "zstd" => FileCompressionType::ZSTD,
                v => return Err(format!("Unsupported compression type: {}", v)),
            };
            let opts = FileOpts {
                filename: s.to_string(),
                ext: ext2.to_string(),
                compression,
            };
            match ext1 {
                "csv" => Ok(DatasetConn::Csv(opts)),
                "json" | "jsonl" | "ndjson" => Ok(DatasetConn::NdJson(opts)),
                v => Err(format!("Unsupported file type: {}", v)),
            }
        }
        (Some(ext1), None) => match ext1 {
            "csv" => Ok(DatasetConn::Csv(FileOpts {
                filename: s.to_string(),
                ext: ext1.to_string(),
                compression: FileCompressionType::UNCOMPRESSED,
            })),
            "json" | "jsonl" | "ndjson" => Ok(DatasetConn::NdJson(FileOpts {
                filename: s.to_string(),
                ext: ext1.to_string(),
                compression: FileCompressionType::UNCOMPRESSED,
            })),
            "parquet" => Ok(DatasetConn::Parquet(s.to_string())),
            v => Err(format!("Unsupported file type: {}", v)),
        },
        _ => Err(format!("Invalid connection string: {}", s)),
    }
}

impl CmdExector for ConnectOpts {
    async fn execute<T: crate::Backend>(self, backend: &mut T) -> anyhow::Result<String> {
        backend.connect(&self).await?;

        Ok(format!("Connected to dataset: {}", &self.name))
    }
}
