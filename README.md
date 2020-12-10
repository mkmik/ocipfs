# ocipfs

Stateless synthetic OCI registry that serves single layers from IPFS,
suitable to be declaratively imported using the `COPY --from=...` construct in otherwise imperative `Dockerfiles`.

## Rationale

Ever wanted to compose Docker/OCI images declaratively out of immutable layers but don't want to
install some extra tooling? Why cannot `docker build` just offer a native way to point to a layer by "hash"?

Sadly, while Dockerfiles do have the ability to create layers with `ADD layer.tar`, it only works with local files.

This project offers a simple trick to achieve that workflow.

## Example

```console
$ cat example/Dockerfile
FROM bitnami/minideb:buster-snapshot-20201210T025900Z

COPY --from=ocipfs.io/layer/bafybeiat5rs4tk3kvllj4uu5zr652qnp7kxc4ttewhc5md23gxflqqp7lu:foo-0.1.0.tar.gz / /

ENTRYPOINT [ "/foo" ]

$ docker build -t example .
```

## How does it work

ocipfs implements the OCI registry protocol and servers "images" containing a single layer.

Instead of allowing layers to be served in arbitrary places, ocipfs leverages an existing content-addressed content delivery network: IPFS.

The images and the layer are addressed with IPFS content IDs (CIDs).

The image reference contains the CID of a small JSON file, called the "Layer Manifest":
```console
$ cat example/layer.json
{
  "mediaType": "application/vnd.ocipfs.layer.manifest.v1+json",
  "layer": {
    "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
    "digest": "sha256:cfb9359acc6d98d593d724bb9f0fd4b12dd12688eb44ee9ac391fb3fde6c0415",
    "size": 168,
    "annotations": {
      "io.ocipfs.layer.ipfs.cid": "QmbSCQoNkbvuYXpjUsERkG13em9WXeSiw5m5omLUnrcTT4/foo-0.1.0.tar.gz",
      "io.ocipfs.layer.fs.digest": "sha256:e02ebdee01b51ceb76a06a45debfb962d6484728f4c348b9ca0c8a2309830ec6"
    }
  }
}
```

You need to serve this file with IPFS and save the CIDv1 (base32) value, e.g. `bafybeiat5rs4tk3kvllj4uu5zr652qnp7kxc4ttewhc5md23gxflqqp7lu`.

Then you derive an image by prefixing `ocipfs.io/layer/`:

```
ocipfs.io/layer/bafybeiat5rs4tk3kvllj4uu5zr652qnp7kxc4ttewhc5md23gxflqqp7lu
```

You can also add an optional "tag", for readability:

```
ocipfs.io/layer/bafybeiat5rs4tk3kvllj4uu5zr652qnp7kxc4ttewhc5md23gxflqqp7lu:foo-0.1.0.tar.gz
```

TODO: the filename in the tag position will be actually checked for consistency with the layer manifest.

The ocipfs server is fully stateless and downloads this small metadata JSON file on the fly to know how to
respond to the various OCI protocol requests. In particular the actual blob data is not served by `ocipfs.io`
but instead it returns a 302 Found response which instructs the client to fetch it directly from IPFS gateway.

This means I can host the `opfs.io` service at low cost (e.g. with Google Cloud Run) and you only need
to care about serving your layers via IPFS.