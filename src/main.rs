use std::env;
use std::error::Error;
use std::fmt::Debug;
use std::path::PathBuf;

use aws_config::SdkConfig;
use aws_sdk_polly::{Client as PollyClient, Error as PollyError};
use aws_sdk_polly::operation::synthesize_speech::SynthesizeSpeechOutput;
use aws_sdk_polly::types::{OutputFormat, VoiceId};
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::config::Region;
use chrono::{NaiveDateTime, ParseResult};
use clap::{arg, Parser, Subcommand};
use sqlx::Connection;
use sqlx::mysql::MySqlConnection;

use writer::Writer;

use crate::models::test_table_row::TestTableRow;
use crate::writer::{FileWriter, S3Writer};

mod models;
mod writer;

fn parse_date(s: &str) -> ParseResult<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
}

/// Export DB table as MP3s spoken by Polly
#[derive(Debug, Parser)]
#[command(name = "git")]
#[command(about = "Export DB table as MP3s spoken by Polly", long_about = None)]
struct PollyExportCli {
    #[command(subcommand)]
    command: Commands,
    /// The AWS profile used
    #[arg(short, long)]
    profile: String,
    /// The AWS region used
    #[arg(short, long)]
    region: String,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Export to file system
    FileExport {
        /// The output directory
        #[arg(short, long)]
        output_dir: PathBuf,
        /// The optional ISO 8601 formatted timestamp of the last execution
        #[arg(short, long, value_parser = parse_date, default_value = None)]
        last_pollified: Option<NaiveDateTime>,
        /// Table to export
        #[arg(short, long)]
        table_name: String,
        /// The name of the id column
        #[arg(long)]
        id_column: String,
        /// The name of the column with the last update timestamp
        #[arg(long)]
        last_updated_column: String,
        /// The name of the text column
        #[arg(long)]
        text_column: String,
    },
    /// Export to S3
    S3Export {
        /// Name of the output bucket
        #[arg(short, long)]
        bucket_name: String,
        /// Prefix of the output objects within the bucket
        #[arg(short, long)]
        prefix: String,
        /// The optional ISO 8601 formatted timestamp of the last execution
        #[arg(short, long, value_parser = parse_date, default_value = None)]
        last_pollified: Option<NaiveDateTime>,
        /// Table to export
        #[arg(short, long)]
        table_name: String,
        /// The name of the id column
        #[arg(long)]
        id_column: String,
        /// The name of the column with the last update timestamp
        #[arg(long)]
        last_updated_column: String,
        /// The name of the text column
        #[arg(long)]
        text_column: String,
    },
}

async fn call_polly(
    content: &str,
    client: &PollyClient,
) -> Result<SynthesizeSpeechOutput, PollyError> {
    let resp: SynthesizeSpeechOutput = client
        .synthesize_speech()
        .output_format(OutputFormat::Mp3)
        .text(content)
        .voice_id(VoiceId::Joanna)
        .send()
        .await?;

    Ok(resp)
}

async fn load_rows(
    table_name: &str,
    id_column: &str,
    last_updated_column: &str,
    text_column: &str,
    last_pollified: Option<NaiveDateTime>,
    connection: &mut MySqlConnection,
) -> Result<Vec<TestTableRow>, Box<dyn Error>> {
    let rows = match last_pollified {
        Some(ndt) => {
            let query = format!("SELECT {id_column} AS id, {last_updated_column} AS last_update, {text_column} AS text FROM {table_name} WHERE last_update > ?");
            sqlx::query_as::<_, TestTableRow>(&query)
                .bind(ndt.and_utc().to_string())
                .fetch_all(connection)
                .await?
                .into_iter()
                .collect::<Vec<TestTableRow>>()
        }
        None => {
            let query = format!("SELECT {id_column} AS id, {last_updated_column} AS last_update, {text_column} AS text FROM {table_name}");
            sqlx::query_as::<_, TestTableRow>(&query)
                .fetch_all(connection)
                .await?
                .into_iter()
                .collect::<Vec<TestTableRow>>()
        }
    };

    Ok(rows)
}

async fn run_export<W: Writer>(
    rows: &Vec<TestTableRow>,
    writer: &W,
    client: &PollyClient,
) -> Result<(), Box<dyn Error>> {
    for row in rows {
        let resp: SynthesizeSpeechOutput = call_polly(&row.text, client)
            .await
            .expect("Expected aggregated bytes");

        writer.write(resp, format!("{}.mp3", &row.id)).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = PollyExportCli::parse();

    let db_user_name = env::var("DB_USER_NAME").expect("DB_USER_NAME must be set");
    let db_password = env::var("DB_PASSWORD").expect("DB_PASSWORD must be set");
    let database_name = env::var("DATABASE_NAME").expect("DATABASE_NAME must be set");

    if db_user_name.is_empty() {
        panic!("Environment variable DB_USER_NAME is empty string.")
    }

    if db_password.is_empty() {
        panic!("Environment variable DB_PASSWORD is empty string.")
    }

    if database_name.is_empty() {
        panic!("Environment variable DATABASE_NAME is empty string.")
    }

    let db_url = format!("mysql://{db_user_name}:{db_password}@127.0.0.1:3306/{database_name}");
    let mut connection: MySqlConnection = MySqlConnection::connect(&db_url).await?;

    let config: SdkConfig = aws_config::from_env()
        .profile_name(args.profile)
        .region(Region::new(args.region))
        .load()
        .await;

    match args.command {
        Commands::FileExport {
            output_dir,
            last_pollified,
            table_name,
            id_column,
            last_updated_column,
            text_column
        } => {
            let client: PollyClient = PollyClient::new(&config);
            let writer = FileWriter { output_dir };
            let rows = load_rows(&table_name, &id_column, &last_updated_column, &text_column, last_pollified, &mut connection).await?;
            run_export::<FileWriter>(&rows, &writer, &client).await?
        }
        Commands::S3Export {
            bucket_name,
            last_pollified,
            prefix,
            table_name,
            id_column,
            last_updated_column,
            text_column
        } => {
            let client: PollyClient = PollyClient::new(&config);
            let writer = S3Writer {
                s3_client: S3Client::new(&config),
                bucket_name,
                prefix: prefix,
            };
            let rows = load_rows(&table_name, &id_column, &last_updated_column, &text_column, last_pollified, &mut connection).await?;
            run_export::<S3Writer>(&rows, &writer, &client).await?
        }
    }

    Ok(())
}
