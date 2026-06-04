FROM alpine:3.20

ARG TARGETARCH
ARG TARGETOS

COPY skillhub-${TARGETOS}-${TARGETARCH} /usr/local/bin/skillhub

RUN chmod +x /usr/local/bin/skillhub && \
    adduser -D -h /home/skillhub skillhub

USER skillhub
WORKDIR /home/skillhub

ENTRYPOINT ["/usr/local/bin/skillhub"]
CMD ["--help"]
