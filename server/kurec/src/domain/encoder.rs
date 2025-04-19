use kurec_adapter::{MeilisearchAdapter, MirakcAdapter, NatsAdapter};
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use tracing::debug;
use tracing::warn;

pub struct EncoderDomain {
    mirakc_adapter: MirakcAdapter,
    nats_adapter: NatsAdapter,
    meilisearch_adapter: MeilisearchAdapter,
    config: kurec_interface::KurecConfig,
}

impl EncoderDomain {
    pub fn new(
        mirakc_adapter: MirakcAdapter,
        nats_adapter: NatsAdapter,
        meilisearch_adapter: MeilisearchAdapter,
        config: kurec_interface::KurecConfig,
    ) -> Self {
        Self {
            mirakc_adapter,
            nats_adapter,
            meilisearch_adapter,
            config,
        }
    }

    pub async fn encode_recorded(&self) -> Result<(), anyhow::Error> {
        let f = |ev: kurec_interface::RecordingStatusMessage| async move {
            debug!("received encode request id:{}", ev.record_id);

            // TODO: 場所は設定出来るようにする？
            let tempdir = tempfile::TempDir::new_in("/tmp")?;
            debug!("tempdir: {:?}", tempdir.path());

            let script = format!(
                r#"#!/bin/bash
            set -e
            {}
            "#,
                self.config.encoder.script
            );
            let script_path = tempdir.path().join("encode.sh");

            // asyncioにする？
            std::fs::write(&script_path, script).unwrap();
            std::fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755)).unwrap();

            let metadata = self
                .mirakc_adapter
                .get_record(&ev.tuner_url, &ev.record_id)
                .await
                .unwrap();
            std::fs::write(
                tempdir.path().join("metadata.json"),
                serde_json::to_string(&metadata)?,
            )?;

            {
                let mut input_ts_writer = std::fs::File::create(tempdir.path().join("input.ts"))?;

                self.mirakc_adapter
                    .save_record_stream(&ev.tuner_url, &ev.record_id, &mut input_ts_writer)
                    .await
                    .unwrap();
                input_ts_writer.flush().unwrap();
            }
            let stat = std::fs::metadata(tempdir.path().join("input.ts"))?;
            debug!("input.ts size: {}", stat.len());

            // TODO: 字幕抽出
            // TODO: 文字起こし

            let output = Command::new(&script_path)
                .current_dir(tempdir.path())
                .output()
                .unwrap();

            // TODO: TBD: stdout, stderrをどうしよう？
            if !output.status.success() {
                return Err(anyhow::anyhow!(
                    "encode failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            let mut message = kurec_interface::EncodeResultMessage {
                tuner_url: ev.tuner_url,
                encode_results: vec![],
            };
            for output in self.config.encoder.outputs.iter() {
                let output_path = tempdir.path().join(&output.name);
                let file_size = output_path.metadata()?.len();
                let encode_result = kurec_interface::EncodeResultFile {
                    name: output.name.clone(),
                    description: output.description.clone(),
                    file_size,
                    storage: output.storage.clone(),
                };
                match output.storage {
                    kurec_interface::StorageType::Local => {
                        let output_path = tempdir.path().join(&output.name);
                        let target_path =
                            Path::new(self.config.storage.local_path.as_str()).join(&output.name);
                        if target_path.exists() {
                            // 2回以上エンコードが呼ばれたときとかの場合
                            // ファイルサイズチェックとかした方が良い？
                            warn!("output file already exists: {}", target_path.display());
                            fs::remove_file(&target_path)?;
                        }
                        fs::rename(output_path, target_path)?;
                    }
                    kurec_interface::StorageType::S3 => {
                        return Err(anyhow::anyhow!(
                            "S3 storage is not supported yet: {}",
                            output.name
                        ));
                    }
                }
                message.encode_results.push(encode_result);
            }

            Ok(None as Option<kurec_interface::EncodeResultMessage>)
        };
        // TODO: エンコード失敗をどう処理するか？
        self.nats_adapter
            .filter_map_stream_async(
                kurec_adapter::StreamType::RecordFinished,
                kurec_adapter::StreamType::EncodeResult,
                "encoder",
                f,
            )
            .await?;
        Ok(())
    }
}
