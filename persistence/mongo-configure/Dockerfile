FROM debian:jessie

RUN apt-get update && apt-get install -y --no-install-recommends curl jq

WORKDIR /usr/local/entry
COPY entrypoint.sh .
copy config.json .

CMD /usr/local/entry/entrypoint.sh
