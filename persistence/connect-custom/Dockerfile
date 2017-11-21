FROM debian:jessie

# install dependencies
RUN apt-get update \
    && apt-get install -y --no-install-recommends wget \
    software-properties-common python-software-properties maven git

# install Java 8
RUN echo deb http://http.debian.net/debian jessie-backports main >> /etc/apt/sources.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends -t jessie-backports openjdk-8-jdk \
    && update-alternatives --set java /usr/lib/jvm/java-8-openjdk-amd64/jre/bin/java \
    && java -version

# install Confluent
RUN wget -qO - http://packages.confluent.io/deb/3.3/archive.key | apt-key add - \
    && add-apt-repository "deb [arch=amd64] http://packages.confluent.io/deb/3.3 stable main" \
    && apt-get update \
    && apt-get install -y confluent-platform-oss-2.11

# install Couchbase connect client
WORKDIR /usr/src/
RUN wget -q http://packages.couchbase.com/clients/kafka/3.1.3/kafka-connect-couchbase-3.1.3.zip \
    && unzip kafka-connect-couchbase-3.1.3.zip \
    && ls -la kafka-connect-couchbase-3.1.3 \
    && mkdir -p /usr/local/share/kafka/plugins \
    && cp -r kafka-connect-couchbase-3.1.3/share/java/kafka-connect-couchbase /usr/local/share/kafka/plugins \
    && ls -la /usr/local/share/kafka/plugins

ENV CLASSPATH /usr/local/share/kafka/plugins/kafka-connect-couchbase/*

# copy customised properties files into docker container
COPY *.properties ./

# set the command to run connect
CMD connect-standalone worker.properties couchbase-sink.properties
