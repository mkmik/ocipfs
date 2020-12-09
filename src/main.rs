#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use anyhow::Result;
use rocket::response::Redirect;
use text_io::try_scan;

const IPFS_GATEWAY: &str = "ipfs.io";
const EMPTY_OBJECT_DIGEST: &str =
    "sha256:15d612984db0dead3f98670882d92203e2d9ce167bc87cf49cafbe465cb6e9f1";

const ZEROS: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

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
    "digest": "{}",
    "size": 196
  }},
  "layers": [
    {{
      "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
      "digest": "sha256:{}",
      "size": {}
    }}
  ]
}}"#,
        EMPTY_OBJECT_DIGEST, sha, size
    );

    Ok(ManifestResponder(body))
}

#[derive(Responder)]
enum BlobsResponder {
    #[allow(dead_code)]
    Config(String),
    Redirect(Redirect),
}

#[get("/layer/<repo>/blobs/<digest>")]
fn blobs(repo: String, digest: String) -> BlobsResponder {
    println!("blob: repo={}, digest={}!", repo, digest);

    if digest == ZEROS || digest == EMPTY_OBJECT_DIGEST {
        BlobsResponder::Config(format!(
            r#"{{
    "architecture": "amd64",
    "os": "linux",
    "rootfs": {{
        "type": "layers",
        "diff_ids": ["sha256:e02ebdee01b51ceb76a06a45debfb962d6484728f4c348b9ca0c8a2309830ec6"]
    }}
}}
"#
        ))
    } else {
        BlobsResponder::Redirect(Redirect::found(format!(
            "https://{}/ipfs/{}",
            IPFS_GATEWAY, repo
        )))
    }
}

fn main() {
    rocket::ignite()
        .mount("/v2", routes![root, manifests, blobs])
        .launch();
}
