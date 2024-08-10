use std::{fmt, sync::Arc};

use arrow::datatypes::{DataType, Field};
use datafusion::{
    functions_aggregate::{count::count, expr_fn::avg, median::median, stddev::stddev, sum::sum},
    logical_expr::{case, cast, col},
    prelude::{array_length, is_null, length, lit, max, min, DataFrame},
};

#[allow(unused)]
#[derive(Debug)]
pub enum DescribeMethod {
    Total,
    NullTotal,
    Mean,
    Stddev,
    Min,
    Max,
    Median,
    Percentile(u8),
}

#[derive(Debug)]
pub struct DataFrameDescribe {
    origin: DataFrame,
    transformed: DataFrame,
    methods: Vec<DescribeMethod>,
}

impl DataFrameDescribe {
    pub fn try_new(df: DataFrame) -> anyhow::Result<Self> {
        let fields = df.schema().fields().iter();
        // change all temporal columns to Float64
        let expressions = fields
            .map(|field| {
                let dt = field.data_type();
                let expr = match dt {
                    dt if dt.is_temporal() => cast(col(field.name()), DataType::Float64),
                    dt if dt.is_numeric() => col(field.name()),
                    DataType::List(_) | DataType::LargeList(_) => array_length(col(field.name())),
                    _ => length(cast(col(field.name()), DataType::Utf8)),
                };
                expr.alias(field.name())
            })
            .collect::<Vec<_>>();

        let transformed = df.clone().select(expressions)?;

        Ok(Self {
            origin: df,
            transformed,
            methods: vec![
                DescribeMethod::Total,
                DescribeMethod::NullTotal,
                DescribeMethod::Mean,
                DescribeMethod::Stddev,
                DescribeMethod::Min,
                DescribeMethod::Max,
                DescribeMethod::Median,
            ],
        })
    }

    pub async fn describe(&self) -> anyhow::Result<DataFrame> {
        let df = self.do_describe().await?;
        self.cast_back(df)
    }

    async fn do_describe(&self) -> anyhow::Result<DataFrame> {
        let df: Option<DataFrame> = self.methods.iter().fold(None, |acc, method| {
            let df = self.transformed.clone();
            let stat_df = match method {
                DescribeMethod::Total => total(df).unwrap(),
                DescribeMethod::NullTotal => null_total(df).unwrap(),
                DescribeMethod::Mean => mean(df).unwrap(),
                DescribeMethod::Stddev => std_dev(df).unwrap(),
                DescribeMethod::Min => minimum(df).unwrap(),
                DescribeMethod::Max => maximum(df).unwrap(),
                DescribeMethod::Median => med(df).unwrap(),
                DescribeMethod::Percentile(_) => todo!(),
            };

            // add a new column to the beginning of the DataFrame
            let mut select_expr = vec![lit(method.to_string()).alias("describe")];
            select_expr.extend(stat_df.schema().fields().iter().map(|f| col(f.name())));

            let stat_df = stat_df.select(select_expr).unwrap();

            match acc {
                Some(acc) => Some(acc.union(stat_df).unwrap()),
                None => Some(stat_df),
            }
        });

        df.ok_or_else(|| anyhow::anyhow!("No statistics describe"))
    }

    fn cast_back(&self, df: DataFrame) -> anyhow::Result<DataFrame> {
        // we need the describe column
        let describe = Arc::new(Field::new("describe", DataType::Utf8, false));
        let mut fields = vec![&describe];
        fields.extend(self.origin.schema().fields().iter());
        let expressions = fields
            .into_iter()
            .map(|field| {
                let dt = field.data_type();
                let expr = match dt {
                    dt if dt.is_temporal() => cast(col(field.name()), dt.clone()),
                    DataType::List(_) | DataType::LargeList(_) => {
                        cast(col(field.name()), DataType::Int32)
                    }
                    _ => col(field.name()),
                };
                expr.alias(field.name())
            })
            .collect::<Vec<_>>();

        Ok(df
            .select(expressions)?
            .sort(vec![col("describe").sort(true, false)])?)
    }
}

impl fmt::Display for DescribeMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DescribeMethod::Total => write!(f, "total"),
            DescribeMethod::NullTotal => write!(f, "null_total"),
            DescribeMethod::Mean => write!(f, "mean"),
            DescribeMethod::Stddev => write!(f, "stddev"),
            DescribeMethod::Min => write!(f, "min"),
            DescribeMethod::Max => write!(f, "max"),
            DescribeMethod::Median => write!(f, "median"),
            DescribeMethod::Percentile(p) => write!(f, "percentile_{}", p),
        }
    }
}

macro_rules! describe_methods {
    ($name:ident, $method:ident) => {
        fn $name(df: DataFrame) -> anyhow::Result<DataFrame> {
            let fields = df.schema().fields().iter();
            let ret = df.clone().aggregate(
                vec![],
                fields
                    .clone()
                    .filter(|f| f.data_type().is_numeric())
                    .map(|f| $method(col(f.name())).alias(f.name()))
                    .collect::<Vec<_>>(),
            )?;
            Ok(ret)
        }
    };
}

describe_methods!(total, count);
describe_methods!(mean, avg);
describe_methods!(std_dev, stddev);
describe_methods!(minimum, min);
describe_methods!(maximum, max);
describe_methods!(med, median);

fn null_total(df: DataFrame) -> anyhow::Result<DataFrame> {
    let fields = df.schema().fields().iter();
    let ret = df.clone().aggregate(
        vec![],
        fields
            .clone()
            .map(|f| {
                sum(case(is_null(col(f.name())))
                    .when(lit(true), lit(1))
                    .otherwise(lit(0))
                    .unwrap())
                .alias(f.name())
            })
            .collect::<Vec<_>>(),
    )?;
    Ok(ret)
}
