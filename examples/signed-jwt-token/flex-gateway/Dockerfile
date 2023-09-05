# Use the original image as a base
FROM mulesoft/flex-gateway

# Copy the configuration file and script into the image
COPY config.properties /config.properties

RUN . /config.properties && flexctl register --client-id=$CLIENT_ID --client-secret=$CLIENT_SECRET --environment=$ENV_ID --organization=$ORG_ID --output-directory=/usr/local/share/mulesoft/flex-gateway/conf.d $FLEX_GW_NAME

# Execute the custom script and then run the original CMD
ENTRYPOINT ["/bin/sh", "-c", "exec /init"]
