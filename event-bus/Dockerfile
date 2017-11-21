FROM rust:1.21

WORKDIR /usr/src/app
COPY . .

RUN cargo install

ENV LOG_LEVEL debug

CMD event-bus --broker $BROKER -g $GROUP -l $LOG_LEVEL server -t $TOPIC
