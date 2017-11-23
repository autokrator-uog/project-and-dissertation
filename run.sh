
if [ "$1" == 'in_docker' ]; then
    docker-compose -f docker/base.yml -f docker/persistence.yml -f docker/event-bus.yml up --build -d
else
    docker-compose -f docker/base.yml -f docker/persistence.yml up --build -d
fi
