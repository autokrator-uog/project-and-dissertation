
HOST=http://connect:8083

# WAIT FOR CONNECT TO BECOME AVAILABLE
until $(curl --output /dev/null --silent --head --fail $HOST); do
    printf '.'
    sleep 1
done

echo
echo "===> Show status"
curl -s $HOST | jq

echo
echo "===> Show connector plugins"
curl -s $HOST/connector-plugins | jq

echo
echo "===> Setup mongodb sink"
curl \
  -X POST \
  -H "Content-Type: application/json" \
  --data @config.json \
  $HOST/connectors

echo
echo "...let's wait a while for it to create..."
sleep 3

echo
echo "===> Show connectors"
curl -s $HOST/connectors | jq
NAME="$(curl -s $HOST/connectors | jq -r '.[0]')"

echo
echo "===> Show MongoDB connector"
curl -s $HOST/connectors/$NAME | jq

echo
echo "===> [CLEANUP] Delete the connector"
curl -X DELETE -H "Content-Type: application/json" localhost:8083/connectors/$NAME
echo "===> [CLEANUP] SUCCESS!"
