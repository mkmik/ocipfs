FROM bitnami/minideb:buster-snapshot-20201210T025900Z@sha256:30b65d14cdf4c3d1c890370c647d11d0841f5a770d645a65bbdeec70dc1d4862

RUN install_packages curl

COPY --from=ocipfs.io/layer/bafybeiat5rs4tk3kvllj4uu5zr652qnp7kxc4ttewhc5md23gxflqqp7lu:foo-0.1.0.tar.gz@sha256:09f6bdadf8c8d846b51cc7a69447262242de9d64391eb708761eaf595e237b29 / /

ENTRYPOINT [ "/foo" ]
