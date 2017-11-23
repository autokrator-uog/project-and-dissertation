# SED-DEV Project

This is the main repository for the SED-DEV project for Avaloq.

We are making an Event Sourcing system based around Apache Kafka.


## Setting up with docker-compose

1. Ensure you have docker correctly installed. Run the following commands to get your host networking set up:

    `docker network create -d bridge --subnet 172.25.0.0/16 thenet`
    `sudo echo "nameserver 127.0.0.11" >> /etc/resolv.conf`

2. `./run.sh` This may take a while to pull in dependencies and build an image (from scratch, could be 10+ minutes).
    You can specify the `in_docker` flag to this script to run the event bus as a container within docker.

3. Set up Couchbase: http://localhost:8091
  - Cluster name `couchbase.db`, user `connect`, pass `connect`
  - accept license terms
  - go to advanced options and in server name enter `couchbase.db` **THIS IS IMPORTANT**, otherwise connect won't be able to resolve Couchbase.
  - you may have to change some memory reservation sizes etc.
  - once in to the main GUI console, go to buckets and create bucket called `events`

4. Reboot connect `docker-compose restart connect`

5. Tail logs, check for any errors `docker-compose logs -f`

6. Install `kt` (https://github.com/fgeller/kt) and produce an event:
    - `echo '{ "key": "HelloEvent1234123", "value": "MyWeirdEvent", "partition": 0 }' | kt produce -topic events -brokers kafka:9092 -timeout 1s`
    - (you may have to add `127.0.0.1 kafka` to your /etc/hosts file)

7. Check the Couchbase bucket documents to see if the event has been persisted.
