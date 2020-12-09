#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::response::Redirect;
use text_io::try_scan;
use anyhow::Result;

const IPFS_GATEWAY: &str = "ipfs.io";
const EMPTY_OBJECT_DIGEST: &str =
    "sha256:44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a";
const EMPTY_OBJECT_CID: &str = "QmbJWAESqCsf4RFCqEY7jecCashj8usXiyDNfKtZCwwzGb";

#[get("/")]
fn root() {}

#[get("/layer/<repo>/manifests/<tag>")]
fn manifests(repo: String, tag: String) -> Result<String> {
	let size: i64;
	let sha: String;
	try_scan!(tag.bytes() => "{}-{}", size, sha);

    println!("manifest repo={}, tag={}, size={}, sha={}!", repo, tag, size, sha);
    Ok(format!(r#"{{
  "schemaVersion": 2,
  "config": {{
    "mediaType": "application/vnd.dummy.config.v1+json",
    "digest": "sha256:44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
    "size": 2
  }},
  "layers": [
    {{
      "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
      "digest": "sha256:{}",
      "size": {}
    }}
  ]
}}"#, sha, size))
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
