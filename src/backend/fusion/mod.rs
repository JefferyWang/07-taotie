mod describe;
mod df_describe;

use std::ops::Deref;

use arrow::array::RecordBatch;
use arrow::util::pretty::pretty_format_batches;
use datafusion::prelude::{CsvReadOptions, NdJsonReadOptions, SessionContext};
use datafusion::{dataframe::DataFrame, prelude::SessionConfig};
use describe::DataFrameDescribe;

use crate::{
    cli::{ConnectOpts, DatasetConn},
    Backend, ReplDisplay,
};

pub struct DataFusionBackend(SessionContext);

impl DataFusionBackend {
    pub fn new() -> Self {
        let mut config = SessionConfig::new();
        config.options_mut().catalog.information_schema = true;
        Self(SessionContext::new_with_config(config))
    }
}

impl Backend for DataFusionBackend {
    async fn connect(&mut self, opts: &ConnectOpts) -> anyhow::Result<()> {
        match &opts.conn {
            DatasetConn::Postgres(conn_str) => {
                println!("Connecting to Postgres: {}", conn_str);
            }
            DatasetConn::Csv(fileopts) => {
                let csv_opts = CsvReadOptions {
                    file_extension: &fileopts.ext,
                    file_compression_type: fileopts.compression,
                    ..Default::default()
                };
                self.register_csv(&opts.name, &fileopts.filename, csv_opts)
                    .await?;
            }
            DatasetConn::Parquet(filename) => {
                self.register_parquet(&opts.name, filename, Default::default())
                    .await?;
            }
            DatasetConn::NdJson(fileopts) => {
                let json_opts = NdJsonReadOptions {
                    file_extension: &fileopts.ext,
                    file_compression_type: fileopts.compression,
                    ..Default::default()
                };
                self.register_json(&opts.name, &fileopts.filename, json_opts)
                    .await?;
            }
        }
        Ok(())
    }

    async fn list(&self) -> anyhow::Result<impl ReplDisplay> {
        let sql = "select table_name, table_type from information_schema.tables WHERE table_schema = 'public'";
        let df = self.0.sql(sql).await?;
        Ok(df)
    }

    async fn schema(&self, name: &str) -> anyhow::Result<impl ReplDisplay> {
        let df = self.0.sql(&format!("DESCRIBE {}", name)).await?;
        Ok(df)
    }

    async fn describe(&self, name: &str) -> anyhow::Result<impl ReplDisplay> {
        let df = self.0.sql(&format!("select * from {}", name)).await?;
        let ddf = DataFrameDescribe::try_new(df)?;
        ddf.describe().await
    }

    async fn head(&self, name: &str, size: usize) -> anyhow::Result<impl ReplDisplay> {
        let df = self
            .0
            .sql(&format!("SELECT * FROM {} LIMIT {}", name, size))
            .await?;
        Ok(df)
    }

    async fn sql(&self, sql: &str) -> anyhow::Result<impl ReplDisplay> {
        let df = self.0.sql(sql).await?;
        Ok(df)
    }
}

impl Deref for DataFusionBackend {
    type Target = SessionContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ReplDisplay for DataFrame {
    async fn display(self) -> anyhow::Result<String> {
        let batches = self.collect().await?;
        let data = pretty_format_batches(&batches)?;
        Ok(data.to_string())
    }
}

impl ReplDisplay for RecordBatch {
    async fn display(self) -> anyhow::Result<String> {
        let data = pretty_format_batches(&[self])?;
        Ok(data.to_string())
    }
}
