FROM docker.io/archlinux:base-devel

COPY ./target/release/king-finance /bin/

ENTRYPOINT ["/bin/king-finance"]
