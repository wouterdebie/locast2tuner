#!/bin/bash
if [[ -f "/app/config/config" ]]; then
	config="--config /app/config/config"
elif [[ ! -z "$l2t_config" ]]; then
	config="--config ${l2t_config}"
fi

/app/locast2tuner -b 0.0.0.0 -d 8 --tuner_count 4 $config
exit $?
