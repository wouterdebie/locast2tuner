# MacOS
{{#include ../locast_account.md}}

A MacOS package is available though [Homebrew](https://brew.sh/):

```sh
# Add the homebrew tap
brew tap wouterdebie/repo

# Install locast2tuner
brew install locast2tuner

# Get the sample config
curl -o locast2tuner.config \
  https://raw.githubusercontent.com/wouterdebie/locast2tuner/main/assets/config.example

# .. edit locast2tuner.config ..

# Run locast2tuner
locast2tuner --config locast2tuner.config
```
