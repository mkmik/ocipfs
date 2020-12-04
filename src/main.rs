#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::response::Redirect;

const IPFS_GATEWAY: &str = "ipfs.io";
const EMPTY_OBJECT_DIGEST: &str = "sha256:44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a";
const EMPTY_OBJECT_CID: &str = "QmbJWAESqCsf4RFCqEY7jecCashj8usXiyDNfKtZCwwzGb";

#[get("/")]
fn root() {}

#[get("/layer/<repo>/manifests/<tag>")]
fn manifests(repo: String, tag: String) -> String {
    println!("manifest repo={}, digest={}!", repo, tag);
    r#"{
  "schemaVersion": 2,
  "config": {
    "mediaType": "application/vnd.dummy.config.v1+json",
    "digest": "sha256:44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
    "size": 622
  },
  "layers": [
    {
      "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
      "digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
      "size": 168
    }
  ]
}"#.to_string()
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
