FROM bitnami/minideb:buster-snapshot-20201210T025900Z@sha256:30b65d14cdf4c3d1c890370c647d11d0841f5a770d645a65bbdeec70dc1d4862

RUN install_packages curl

COPY --from=ocipfs.io/layer/bafybeiat5rs4tk3kvllj4uu5zr652qnp7kxc4ttewhc5md23gxflqqp7lu:foo-0.1.0.tar.gz@sha256:b588c6d2f8ba7c4275aa80975b8eaf489ea6456c9ea44297960a34a07f7ab059 / /

ENTRYPOINT [ "/foo" ]
