use err_derive::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageConfig {
    pub architecture: String,
    pub os: String,
    pub rootfs: RootFS,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RootFS {
    #[serde(rename = "type")]
    pub _type: String,
    pub diff_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Descriptor {
    pub media_type: String,
    pub digest: String,
    pub size: u64,
    pub annotations: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LayerManifest {
    pub media_type: String,
    pub layer: Descriptor,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageManifest {
    pub schema_version: i32,
    pub media_type: String,
    pub config: Descriptor,
    pub layers: Vec<Descriptor>,
}

impl TryFrom<&LayerManifest> for ImageConfig {
    type Error = anyhow::Error;

    fn try_from(lm: &LayerManifest) -> Result<Self, Self::Error> {
        let tsha = lm
            .layer
            .annotations
            .get("io.ocipfs.layer.fs.digest")
            .ok_or(LayerManifestError::MissingTarDigest)?;

        Ok(ImageConfig {
            architecture: "amd64".to_string(),
            os: "linux".to_string(),
            rootfs: RootFS {
                _type: "layers".to_string(),
                diff_ids: vec![tsha.clone()],
            },
        })
    }
}

#[derive(Debug, Error)]
pub enum LayerManifestError {
    #[error(display = "Missing io.ocipfs.layer.ipfs.cid annotation")]
    MissingCID,
    #[error(display = "Missing io.ocipfs.layer.fs.digest annotation")]
    MissingTarDigest,
}
