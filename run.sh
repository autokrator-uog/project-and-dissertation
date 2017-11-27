#!/bin/bash

# Set up the /etc/hosts file to be able to resolve containers by their name
echo "====> Setting up /etc/hosts"

if ! grep -q kafka /etc/hosts; then
  echo "127.0.0.1 kafka" >> /etc/hosts
fi

if ! grep -q zookeeper /etc/hosts; then
   echo "127.0.0.1 zookeeper" >> /etc/hosts
fi

if ! grep -q connect /etc/hosts; then
   echo "127.0.0.1 connect" >> /etc/hosts
fi

if ! grep -q couchbase.db /etc/hosts; then
   echo "127.0.0.1 couchbase.db" >> /etc/hosts
fi

if ! grep -q event-bus /etc/hosts; then
   echo "127.0.0.1 event-bus" >> /etc/hosts
fi
echo "====> [+] /etc/hosts configured correctly."


# give the option to run the event bus in a docker container or not (default).
echo
echo
echo "====> Starting docker-compose..."
if [ "$1" == 'in_docker' ]; then
    docker-compose -f docker/base.yml -f docker/persistence.yml -f docker/event-bus.yml up --build -d --remove-orphans
else
    docker-compose -f docker/base.yml -f docker/persistence.yml up --build -d --remove-orphans
fi
echo "====> [+] docker-compose started."

echo
echo
echo "====> ....waiting for containers to come online..."
until [ "$(curl --silent --head --output /dev/stdout couchbase.db:8091 2>&1 | cat -)" != "" ]; do
    printf '.'
    sleep 1
done
until [ "$(curl --silent --head --output /dev/stdout connect:8083 2>&1 | cat -)" != "" ]; do
    printf '.'
    sleep 1
done

echo
echo
echo "====> Testing name resolution for containers..."
set -eE   # set the script to fail if any of the commands return non-zero
trap 'echo "====> [-] ERROR IN /etc/hosts SET UP!"' ERR

curl --silent --head --output /dev/null couchbase.db:8091
echo "[+] Couchbase: OK"

curl --silent --head --output /dev/null connect:8083
echo "[+] Connect: OK"

curl --output /dev/stdout --head kafka:9092 2>&1 | grep "Empty reply from server"
echo "[+] Kafka: OK"

curl --output /dev/stdout --head zookeeper:2181 2>&1 | grep "Empty reply from server"
echo "[+] Zookeeper: OK"

set +eE
echo "====> [+] Name resolution successful!"


echo
echo
echo "======================================="
if [ "$1" == 'in_docker' ]; then
    echo "EVENT BUS LOGS: "
    docker-compose -f docker/base.yml -f docker/persistence.yml -f docker/event-bus.yml logs -f event-bus

elif [ "$1" == 'run_bus' ]; then
    echo "RUN EVENT BUS:"
    cd event-bus
    cargo run -- "$@"

else
    echo "====> [+] DEPENDENCIES RUNNING!"
fi
