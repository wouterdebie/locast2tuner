#!/bin/bash
CONFIG_FILE="/app/config/config"

# Default config file. This is to provide backwards compatibility with
# and old docker-compose.yml where the configuration file always had to
# be mounted.
if [[ -f "${CONFIG_FILE}" ]]; then
	config="--config ${CONFIG_FILE}"
elif [[ ! -z "$l2t_config" ]]; then
	config="--config ${l2t_config}"
fi

/app/locast2tuner -b 0.0.0.0 $config
exit $?
