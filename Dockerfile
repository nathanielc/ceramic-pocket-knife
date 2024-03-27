FROM public.ecr.aws/r5b3e0r5/3box/rust-builder:latest as builder

RUN mkdir -p /home/builder/cpk
WORKDIR /home/builder/cpk

# Use the same ids as the parent docker image by default
ARG UID=1001
ARG GID=1001

# Copy in source code
COPY . .

# Build application using a docker cache
# To clear the cache use:
#   docker builder prune --filter type=exec.cachemount
RUN --mount=type=cache,target=/home/builder/.cargo,uid=$UID,gid=$GID \
	--mount=type=cache,target=/home/builder/cpk/target,uid=$UID,gid=$GID \
    make build-all && \
    cp ./target/debug/cpk ./

FROM debian:bookworm-slim

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /home/builder/cpk/cpk /usr/bin

# Adding this step after copying the ckp binary so that we always take the newest libs from the builder if the main
# binary has changed. Updated dependencies will result in an updated binary, which in turn will result in the latest
# versions of the dependencies being pulled from the builder.
COPY --from=builder /usr/lib/*-linux-gnu*/libsqlite3.so* /usr/lib/
COPY --from=builder /usr/lib/*-linux-gnu*/libssl.so* /usr/lib/
COPY --from=builder /usr/lib/*-linux-gnu*/libcrypto.so* /usr/lib/

ENTRYPOINT ["/usr/bin/cpk"]