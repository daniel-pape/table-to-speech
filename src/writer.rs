use std::error::Error;
use std::path::PathBuf;

use async_trait::async_trait;
use aws_sdk_polly::operation::synthesize_speech::SynthesizeSpeechOutput;
use aws_sdk_polly::primitives::{AggregatedBytes, ByteStream};
use aws_sdk_s3::Client as S3Client;
use tokio::io::AsyncWriteExt;

pub struct S3Writer {
    pub s3_client: S3Client,
    pub bucket_name: String,
    pub prefix: String,
}

pub struct FileWriter {
    pub output_dir: PathBuf,
}

#[async_trait]
pub trait Writer {
    async fn write(
        &self,
        synthesize_speech_output: SynthesizeSpeechOutput,
        filename: String,
    ) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
impl Writer for S3Writer {
    async fn write(
        &self,
        synthesize_speech_output: SynthesizeSpeechOutput,
        filename: String,
    ) -> Result<(), Box<dyn Error>> {
        let blob = synthesize_speech_output.audio_stream.collect().await?;

        self.s3_client
            .put_object()
            .bucket(&self.bucket_name)
            .key(format!("{}/{}", self.prefix, filename))
            .body(ByteStream::from(blob.into_bytes()))
            .send()
            .await?;

        Ok(())
    }
}

#[async_trait]
impl Writer for FileWriter {
    async fn write(
        &self,
        synthesize_speech_output: SynthesizeSpeechOutput,
        filename: String,
    ) -> Result<(), Box<dyn Error>> {
        let mut blob: AggregatedBytes = synthesize_speech_output
            .audio_stream
            .collect()
            .await
            .expect("failed to read data");

        let output_dir_exists = tokio::fs::try_exists(&self.output_dir).await?;

        if !output_dir_exists {
            panic!("Output directory {:?} must exist. Please create it.", self.output_dir);
        }

        let mut file = tokio::fs::File::create(self.output_dir.join(filename))
            .await
            .expect("failed to create file");

        file.write_all_buf(&mut blob)
            .await
            .expect("failed to write to file");

        Ok(())
    }
}
