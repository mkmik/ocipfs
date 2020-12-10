FROM bitnami/minideb:buster-snapshot-20201210T025900Z

RUN install_packages curl

COPY --from=ocipfs.io/layer/bafybeiat5rs4tk3kvllj4uu5zr652qnp7kxc4ttewhc5md23gxflqqp7lu:foo-0.1.0.tar.gz / /

ENTRYPOINT [ "/foo" ]
