use std::io::Cursor;

use anyhow::Result;
use arrow::util::pretty::pretty_format_batches;
use futures::{pin_mut, TryStreamExt as _};
use libp2p::bytes::Bytes;
use parquet::arrow::ParquetRecordBatchStreamBuilder;
use parquet::file::metadata::ParquetMetaDataReader;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::cli::{Command, ParquetDump, ParquetInspect, PrintFormat};

pub enum Operation {
    Dump(ParquetDump),
    Inspect(ParquetInspect),
}

impl TryFrom<Command> for Operation {
    type Error = Command;

    fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
        match value {
            Command::ParquetDump(args) => Ok(Operation::Dump(args)),
            Command::ParquetInspect(args) => Ok(Operation::Inspect(args)),
            _ => Err(value),
        }
    }
}

pub async fn run(op: Operation, _stdin: impl AsyncRead, stdout: impl AsyncWrite) -> Result<()> {
    pin_mut!(stdout);
    match op {
        Operation::Dump(args) => {
            let mut batches = Vec::new();
            for file in args.files {
                let file = File::open(file).await?;
                let builder = ParquetRecordBatchStreamBuilder::new(file).await?;
                let mut stream = builder.build()?;
                while let Some(batch) = stream.try_next().await? {
                    batches.push(batch);
                }
            }

            match args.format {
                PrintFormat::Csv => {
                    let mut buffer = Vec::new();
                    let mut header = true;
                    for batch in &batches {
                        {
                            let mut writer = arrow::csv::WriterBuilder::new()
                                .with_header(header)
                                .build(Cursor::new(&mut buffer));
                            writer.write(batch)?;
                            // Write the header only once
                            header = false;
                        }
                        stdout.write_all(&buffer).await?;
                        buffer.clear();
                    }
                }

                PrintFormat::Json => {
                    let mut buffer = Vec::new();
                    for batch in &batches {
                        {
                            let mut writer =
                                arrow::json::Writer::<_, arrow::json::writer::LineDelimited>::new(
                                    Cursor::new(&mut buffer),
                                );
                            writer.write(batch)?;
                        }
                        stdout.write_all(&buffer).await?;
                        buffer.clear();
                    }
                }
                PrintFormat::Pretty => {
                    stdout
                        .write_all(pretty_format_batches(&batches)?.to_string().as_bytes())
                        .await?;
                }
            };
        }
        Operation::Inspect(args) => {
            let mut file_content = Vec::new();
            File::open(args.file)
                .await?
                .read_to_end(&mut file_content)
                .await?;
            let mut reader = ParquetMetaDataReader::new();
            reader.try_parse(&Bytes::from(file_content)).unwrap();
            let metadata = reader.finish().unwrap();
            stdout
                .write_all(format!("{:#?}", metadata).as_bytes())
                .await?;
        }
    }
    Ok(())
}
