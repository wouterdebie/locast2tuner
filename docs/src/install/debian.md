# Ubuntu/Debian
{{#include ../locast_account.md}}

Ubuntu/Debian packages are available for both amd64 and arm7 (Raspbian):
```sh
# Add the PPA key
curl -s "https://wouterdebie.github.io/ppa/KEY.gpg" | sudo apt-key add -

# Add the locast2tuner repository
sudo curl -o /etc/apt/sources.list.d/locast2tuner.list \
   "https://wouterdebie.github.io/ppa/sources.list"
sudo apt update

# Install locast2tuner
sudo apt install locast2tuner
```

Create a config file. Don't forget to edit the config file!
```sh
sudo cp /etc/locast2tuner/config.example /etc/locast2tuner/config
# .. edit the config /etc/locast2tuner/config (e.g nano /etc/locast2tuner/config) ..
```
Finally, enable and start the service:

```
sudo systemctl enable locast2tuner
sudo systemctl start locast2tuner
```
