#!/usr/bin/with-contenv bash

CONFIG_FILE="/config"
cat <<EOF > $CONFIG_FILE
username="${L2TUSER}"
password="${L2TPASS}"
override_zipcodes="${L2TZIP}"
EOF

if [ $multiplex = "true" ]; then
	multiplex="-m"
else
	multiplex=""
fi

locast2tuner --config $CONFIG_FILE -b 0.0.0.0 -d 8 -r --tuner_count 4 $multiplex
exit $?
