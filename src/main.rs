#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use anyhow::Result;
use rocket::response::{Redirect};
use text_io::try_scan;

const IPFS_GATEWAY: &str = "ipfs.io";
const EMPTY_OBJECT_DIGEST: &str =
    "sha256:bcd90c310ea94b930d370ca1a3996471a92beb68fdb39246b7172ac5f0679f88";
const EMPTY_OBJECT_CID: &str = "QmSmwSKtijUZ7GSzso695rP8PRcRdjdJe7WVC2BdtX7Jt9";

#[get("/")]
fn root() {}

#[derive(Responder)]
#[response(content_type = "application/vnd.oci.image.manifest.v1+json")]
struct ManifestResponder(String);

#[get("/layer/<_repo>/manifests/<tag>")]
fn manifests(_repo: String, tag: String) -> Result<ManifestResponder> {
    let size: i64;
    let sha: String;
    try_scan!(tag.bytes() => "{}-{}", size, sha);

    let body = format!(
        r#"{{
  "schemaVersion": 2,
  "mediaType": "application/vnd.oci.image.manifest.v1+json",
  "config": {{
    "mediaType": "application/vnd.oci.image.config.v1+json",
    "digest": "sha256:bcd90c310ea94b930d370ca1a3996471a92beb68fdb39246b7172ac5f0679f88",
    "size": 123
  }},
  "layers": [
    {{
      "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
      "digest": "sha256:{}",
      "size": {}
    }}
  ]
}}"#,
        sha, size
    );

    Ok(ManifestResponder(body))
}

#[get("/layer/<repo>/blobs/<digest>")]
fn blobs(repo: String, digest: String) -> Redirect {
    println!("blob: repo={}, digest={}!", repo, digest);

    let to = if digest == EMPTY_OBJECT_DIGEST {
        EMPTY_OBJECT_CID.to_string()
    } else {
        repo
    };

    Redirect::found(format!("https://{}/ipfs/{}", IPFS_GATEWAY, to))
}

fn main() {
    rocket::ignite()
        .mount("/v2", routes![root, manifests, blobs])
        .launch();
}
