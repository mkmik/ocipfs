#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use anyhow::Result;
use rocket::http::ContentType;
use rocket::request::Request;
use rocket::response::{self, Redirect, Responder, Response};
use std::io::Cursor;
use text_io::try_scan;

const IPFS_GATEWAY: &str = "ipfs.io";
const EMPTY_OBJECT_DIGEST: &str =
    "sha256:bcd90c310ea94b930d370ca1a3996471a92beb68fdb39246b7172ac5f0679f88";
const EMPTY_OBJECT_CID: &str = "QmSmwSKtijUZ7GSzso695rP8PRcRdjdJe7WVC2BdtX7Jt9";

fn image_manifest_content_type() -> ContentType {
    ContentType::new("application", "vnd.oci.image.manifest.v1+json")
}

#[get("/")]
fn root() {}

struct ManifestResponse {
    sha: String,
    size: i64,
}

impl<'r> Responder<'r> for ManifestResponse {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
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
            self.sha, self.size
        );
        Response::build()
            .sized_body(Cursor::new(body))
            .header(image_manifest_content_type())
            .ok()
    }
}

#[get("/layer/<repo>/manifests/<tag>")]
fn manifests(repo: String, tag: String) -> Result<ManifestResponse> {
    let size: i64;
    let sha: String;
    try_scan!(tag.bytes() => "{}-{}", size, sha);

    println!(
        "manifest repo={}, tag={}, size={}, sha={}!",
        repo, tag, size, sha
    );
    Ok(ManifestResponse { sha, size })
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
