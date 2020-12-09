#![feature(proc_macro_hygiene, decl_macro)]
#![allow(unused_imports)]

#[macro_use]
extern crate rocket;

use anyhow::Result;
use err_derive::Error;
use reqwest::blocking;
use rocket::response::Redirect;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use text_io::try_scan;

const IPFS_GATEWAY: &str = "ipfs.io";

#[derive(Debug)]
struct CID<'a>(&'a str);

impl<'a> CID<'a> {
    fn fetch(&'a self) -> Result<String> {
        let u = format!("https://{}/ipfs/{}", IPFS_GATEWAY, self.0);
        println!("downloading from {}", &u);
        let res = reqwest::blocking::get(&u)?.text()?;
        println!("downloaded");
        Ok(res)
    }
}

#[derive(Debug, Error)]
enum PathError {
    #[error(display = "No last component")]
    NoLastComponent,
    #[error(display = "No base component")]
    NoBaseComponent,
    #[error(display = "No operation")]
    NoOp,
    #[error(display = "No CID")]
    NoCid,
    #[error(display = "Cannot convert to UTF-8")]
    BadUTF8,
    #[error(display = "Unknown operation {}", 0)]
    UnknownOperation(String),
}

#[get("/layer/<rest..>")]
fn layer(rest: PathBuf) -> Result<LayerResult> {
    use LayerResult::*;

    let base = rest.parent().ok_or(PathError::NoBaseComponent)?;
    let op = base
        .file_name()
        .ok_or(PathError::NoOp)?
        .to_str()
        .ok_or(PathError::BadUTF8)?;
    let last = rest
        .file_name()
        .ok_or(PathError::NoLastComponent)?
        .to_str()
        .ok_or(PathError::BadUTF8)?;
    let cid = CID(base
        .parent()
        .ok_or(PathError::NoCid)?
        .to_str()
        .ok_or(PathError::BadUTF8)?);

    Ok(match op {
        "manifests" => Manifests(manifests(cid, last)?),
        "blobs" => Blobs(blobs(cid, last)?),
        _ => Err(PathError::UnknownOperation(op.to_string()))?,
    })
}

#[derive(Responder)]
enum LayerResult {
    #[response(content_type = "application/vnd.oci.image.manifest.v1+json")]
    Manifests(Json<ImageManifest>),
    Blobs(BlobsResult),
}

#[derive(Responder)]
enum BlobsResult {
    #[response(content_type = "application/vnd.oci.image.config.v1+json")]
    Config(Json<ImageConfig>),
    Tarball(Redirect),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ImageConfig {
    architecture: String,
    os: String,
    rootfs: RootFS,
}

#[derive(Serialize, Deserialize, Debug)]
struct RootFS {
    #[serde(rename = "type")]
    _type: String,
    diff_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Descriptor {
    media_type: String,
    digest: String,
    size: usize,
    annotations: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LayerManifest {
    media_type: String,
    layer: Descriptor,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ImageManifest {
    schema_version: i32,
    media_type: String,
    config: Descriptor,
    layers: Vec<Descriptor>,
}

impl LayerManifest {
    fn config(&self) -> Result<ImageConfig> {
        let tsha = self
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

impl TryFrom<&CID<'_>> for LayerManifest {
    type Error = anyhow::Error;

    fn try_from(cid: &CID<'_>) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&cid.fetch()?)?)
    }
}

fn manifests<'a>(cid: CID, _tag: &'a str) -> Result<Json<ImageManifest>> {
    let lman = LayerManifest::try_from(&cid)?;
    println!("Manifest = {:?}", &lman);

    let cfg = lman.config()?;
    let iman = ImageManifest {
        schema_version: 2,
        media_type: "application/vnd.oci.image.manifest.v1+json".to_string(),
        config: Descriptor {
            media_type: "application/vnd.oci.image.config.v1+json".to_string(),
            size: serde_json::to_vec(&cfg)?.len(),
            digest: mkdigest(&cfg)?,
            annotations: HashMap::new(),
        },
        layers: vec![lman.layer],
    };

    Ok(Json(iman))
}

#[derive(Debug, Error)]
enum LayerManifestError {
    #[error(display = "Missing io.ocipfs.layer.ipfs.cid annotation")]
    MissingCID,
    #[error(display = "Missing io.ocipfs.layer.fs.digest annotation")]
    MissingTarDigest,
}

#[derive(Debug, Error)]
enum BlobsError {
    #[error(display = "Unknown manifest")]
    UnknownManifest,
}

fn blobs<'a>(cid: CID, digest: &'a str) -> Result<BlobsResult> {
    use BlobsResult::*;

    let lman = LayerManifest::try_from(&cid)?;
    println!("Manifest = {:?}", &lman);
    if digest == lman.layer.digest {
        let lcid = lman
            .layer
            .annotations
            .get("io.ocipfs.layer.ipfs.cid")
            .ok_or(LayerManifestError::MissingCID)?;
        let u = format!("https://{}/ipfs/{}", IPFS_GATEWAY, lcid);
        Ok(Tarball(Redirect::found(u)))
    } else {
        let cfg = lman.config()?;
        if digest == mkdigest(&cfg)? {
            Ok(Config(Json(cfg)))
        } else {
            Err(BlobsError::UnknownManifest)?
        }
    }
}

fn mkdigest<T: Serialize>(v: T) -> Result<String> {
    let mut hasher = Sha256::new();
    serde_json::to_writer(&mut hasher, &v)?;
    let result = hasher.finalize();
    Ok(format!("sha256:{:x}", result))
}

#[get("/")]
fn root() {}

fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/v2", routes![root, layer])
    //rocket::ignite().mount("/v2", routes![root, manifests])
}
fn main() {
    rocket().launch();
}

#[cfg(test)]
#[allow(unused_variables)]
mod test {
    use super::rocket;
    use rocket::http::Status;
    use rocket::local::Client;

    #[test]
    fn root() {
        let client = Client::new(rocket()).expect("valid rocket instance");

        let mut response = client.get("/v2/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), None);
    }

    #[test]
    fn manifest() {
        let client = Client::new(rocket()).expect("valid rocket instance");

        let mut response = client.get("/v2/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), None);
    }
}
