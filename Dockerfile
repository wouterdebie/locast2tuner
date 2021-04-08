#Download base image ubuntu 20.04                                               
FROM ubuntu:20.04                                                               
                                                                                
# Disable Prompt During Packages Installation and useless warning               
ARG DEBIAN_FRONTEND=noninteractive                                              
ARG APT_KEY_DONT_WARN_ON_DANGEROUS_USAGE=1                                      
                                                                                
ARG BUILD_PACKAGES="gnupg2 wget ca-certificates apt-transport-https"            
                                                                                
# Install build and PPA packages
RUN apt-get update && apt-get install -y --no-install-recommends \
$BUILD_PACKAGES

# Add the PPA key and repo
RUN wget -q -O - "https://wouterdebie.github.io/ppa/KEY.gpg" | apt-key add -
RUN wget -q -O /etc/apt/sources.list.d/locast2tuner.list "https://wouterdebie.github.io/ppa/sources.list"

# Update Ubuntu repository and install locast2tuner package
RUN apt-get update && apt-get install -y --no-install-recommends \
locast2tuner \
&& rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* /var/cache/apt/*

ENTRYPOINT ["/usr/bin/locast2tuner", "--config", "/app/config/config.ini", "-a", "0.0.0.0"]
