#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use anyhow::Result;
use count_write::CountWrite;
use err_derive::Error;
use ocipfs::oci::*;
use rocket::response::Redirect;
use rocket_contrib::json::Json;
use serde::Serialize;
use sha2::{Digest as Sha2Digest, Sha256};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::path::PathBuf;

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

impl TryFrom<&CID<'_>> for LayerManifest {
    type Error = anyhow::Error;

    fn try_from(cid: &CID<'_>) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&cid.fetch()?)?)
    }
}

fn manifests<'a>(cid: CID, _tag: &'a str) -> Result<Json<ImageManifest>> {
    let lman = LayerManifest::try_from(&cid)?;
    println!("Manifest = {:?}", &lman);

    let digest = Digest::new(&ImageConfig::try_from(&lman)?)?;
    let iman = ImageManifest {
        schema_version: 2,
        media_type: "application/vnd.oci.image.manifest.v1+json".to_string(),
        config: Descriptor {
            media_type: "application/vnd.oci.image.config.v1+json".to_string(),
            size: digest.size,
            digest: digest.to_string(),
            annotations: HashMap::new(),
        },
        layers: vec![lman.layer],
    };

    Ok(Json(iman))
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
        let cfg = ImageConfig::try_from(&lman)?;
        let d = Digest::new(&cfg)?;
        if digest == d.to_string() {
            Ok(Config(Json(cfg)))
        } else {
            Err(BlobsError::UnknownManifest)?
        }
    }
}

struct Digest {
    hash: generic_array::GenericArray<u8, <Sha256 as Sha2Digest>::OutputSize>,
    size: u64,
}

impl Digest {
    fn new<T: Serialize>(v: T) -> Result<Self> {
        let mut hasher = Sha256::new();
        let size = {
            let mut cw = CountWrite::from(&mut hasher);
            serde_json::to_writer(&mut cw, &v)?;
            cw.count()
        };
        let hash = hasher.finalize();
        Ok(Digest { hash, size })
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sha256:{:x}", self.hash)
    }
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
