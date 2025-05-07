// Copyright 2021 Datafuse Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::client::PresignMode;
use crate::error::{Error, Result};
use crate::request::StageAttachmentConfig;
use crate::{APIClient, Page, Pages, QueryStats, StageLocation};
use log::{info, warn};
use reqwest::multipart::{Form, Part};
use reqwest::{Body, Client as HttpClient, StatusCode};
use std::sync::Arc;
use std::{collections::BTreeMap, path::Path};
use tokio::io::AsyncRead;
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

const HEADER_STAGE_NAME: &str = "X-DATABEND-STAGE-NAME";

pub type Reader = Box<dyn AsyncRead + Send + Sync + Unpin + 'static>;

pub struct PresignedResponse {
    pub method: String,
    pub headers: BTreeMap<String, String>,
    pub url: String,
}

pub async fn presign_upload_to_stage(
    presigned: PresignedResponse,
    data: Reader,
    size: u64,
) -> Result<()> {
    info!("upload to stage with presigned url, size: {}", size);
    let client = HttpClient::new();
    let mut builder = client.put(presigned.url);
    for (k, v) in presigned.headers {
        if k.to_lowercase() == "content-length" {
            continue;
        }
        builder = builder.header(k, v);
    }
    builder = builder.header("Content-Length", size.to_string());
    let stream = Body::wrap_stream(ReaderStream::new(data));
    let resp = builder.body(stream).send().await?;
    let status = resp.status();
    let body = resp.bytes().await?;
    match status {
        StatusCode::OK => Ok(()),
        _ => Err(Error::IO(format!(
            "Upload with presigned url failed: {}",
            String::from_utf8_lossy(&body)
        ))),
    }
}

pub async fn presign_download_from_stage(
    presigned: PresignedResponse,
    local_path: &Path,
) -> Result<u64> {
    if let Some(p) = local_path.parent() {
        tokio::fs::create_dir_all(p).await?;
    }
    let client = HttpClient::new();
    let mut builder = client.get(presigned.url);
    for (k, v) in presigned.headers {
        builder = builder.header(k, v);
    }

    let resp = builder.send().await?;
    let status = resp.status();
    match status {
        StatusCode::OK => {
            let mut file = tokio::fs::File::create(local_path).await?;
            let mut body = resp.bytes_stream();
            while let Some(chunk) = body.next().await {
                file.write_all(&chunk?).await?;
            }
            file.flush().await?;
            let metadata = file.metadata().await?;
            Ok(metadata.len())
        }
        _ => Err(Error::IO(format!(
            "Download with presigned url failed: {}",
            status
        ))),
    }
}

impl APIClient {
    /// Upload data to stage with stream api, should not be used directly, use `upload_to_stage` instead.
    async fn upload_to_stage_with_stream(
        &self,
        stage: &str,
        data: Reader,
        size: u64,
    ) -> Result<()> {
        info!("upload to stage with stream: {}, size: {}", stage, size);
        if let Some(info) = self.need_pre_refresh_session().await {
            self.refresh_session_token(info).await?;
        }
        let endpoint = self.endpoint.join("v1/upload_to_stage")?;
        let location = StageLocation::try_from(stage)?;
        let query_id = self.gen_query_id();
        let mut headers = self.make_headers(Some(&query_id))?;
        headers.insert(HEADER_STAGE_NAME, location.name.parse()?);
        let stream = Body::wrap_stream(ReaderStream::new(data));
        let part = Part::stream_with_length(stream, size).file_name(location.path);
        let form = Form::new().part("upload", part);
        let mut builder = self.cli.put(endpoint.clone());
        builder = self.wrap_auth_or_session_token(builder)?;
        let resp = builder.headers(headers).multipart(form).send().await?;
        let status = resp.status();
        if status != 200 {
            return Err(
                Error::response_error(status, &resp.bytes().await?).with_context("upload_to_stage")
            );
        }
        Ok(())
    }

    pub async fn insert_with_stage(
        self: &Arc<Self>,
        sql: &str,
        stage: &str,
        file_format_options: BTreeMap<&str, &str>,
        copy_options: BTreeMap<&str, &str>,
    ) -> Result<QueryStats> {
        info!(
            "insert with stage: {}, format: {:?}, copy: {:?}",
            sql, file_format_options, copy_options
        );
        let stage_attachment = Some(StageAttachmentConfig {
            location: stage,
            file_format_options: Some(file_format_options),
            copy_options: Some(copy_options),
        });
        let resp = self.start_query_inner(sql, stage_attachment).await?;
        let mut pages = Pages::new(self.clone(), resp, false);
        let mut all = Page::default();
        while let Some(page) = pages.next().await {
            all.update(page?);
        }
        Ok(all.stats)
    }

    async fn get_presigned_upload_url(self: &Arc<Self>, stage: &str) -> Result<PresignedResponse> {
        info!("get presigned upload url: {}", stage);
        let sql = format!("PRESIGN UPLOAD {}", stage);
        let resp = self.query_all(&sql).await?;
        if resp.data.len() != 1 {
            return Err(Error::Decode(
                "Empty response from server for presigned request".to_string(),
            ));
        }
        if resp.data[0].len() != 3 {
            return Err(Error::Decode(
                "Invalid response from server for presigned request".to_string(),
            ));
        }
        // resp.data[0]: [ "PUT", "{\"host\":\"s3.us-east-2.amazonaws.com\"}", "https://s3.us-east-2.amazonaws.com/query-storage-xxxxx/tnxxxxx/stage/user/xxxx/xxx?" ]
        let method = resp.data[0][0].clone().unwrap_or_default();
        if method != "PUT" {
            return Err(Error::Decode(format!(
                "Invalid method for presigned upload request: {}",
                method
            )));
        }
        let headers: BTreeMap<String, String> =
            serde_json::from_str(resp.data[0][1].clone().unwrap_or("{}".to_string()).as_str())?;
        let url = resp.data[0][2].clone().unwrap_or_default();
        Ok(PresignedResponse {
            method,
            headers,
            url,
        })
    }

    pub(crate) async fn check_presign(self: &Arc<Self>) -> Result<()> {
        let mode = match self.get_presign_mode() {
            PresignMode::Auto => {
                if self.host().ends_with(".databend.com") || self.host().ends_with(".databend.cn") {
                    PresignMode::On
                } else {
                    PresignMode::Off
                }
            }
            PresignMode::Detect => match self.get_presigned_upload_url("@~/.bendsql/check").await {
                Ok(_) => PresignMode::On,
                Err(e) => {
                    warn!("presign mode off with error detected: {}", e);
                    PresignMode::Off
                }
            },
            mode => mode,
        };
        self.set_presign_mode(mode);
        Ok(())
    }

    pub async fn upload_to_stage(
        self: &Arc<Self>,
        stage: &str,
        data: Reader,
        size: u64,
    ) -> Result<()> {
        match self.get_presign_mode() {
            PresignMode::Off => self.upload_to_stage_with_stream(stage, data, size).await,
            PresignMode::On => {
                let presigned = self.get_presigned_upload_url(stage).await?;
                presign_upload_to_stage(presigned, data, size).await
            }
            PresignMode::Auto => {
                unreachable!("PresignMode::Auto should be handled during client initialization")
            }
            PresignMode::Detect => {
                unreachable!("PresignMode::Detect should be handled during client initialization")
            }
        }
    }
}
