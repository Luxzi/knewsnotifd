FROM busybox
COPY target/release/knewsnotifd /usr/bin
ARG RUST_LOG=info
ARG KNEWSNOTIFD_WEBHOOK_URL=Your_Webhook_Here
RUN /usr/bin/knewsnotifd