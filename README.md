# [deluge-windscribe-ephemeral-port](https://github.com/dumbasPL/deluge-windscribe-ephemeral-port)

Automatically create ephemeral ports in windscribe and update deluge config to use the new port

## Important information

This project was designed to work along side containers like [kabe0/deluge-windscribe](https://github.com/Kabe0/deluge-windscribe) in mind.  
It will not help you configure windscribe to use a vpn!  
It will only update the port that deluge listens on to the same port that's configured on windscribe website.

# Configuration

Configuration is done using environment variables

| Variable | Description | Required | Default |
| :-: | :-: | :-: | :-: |
| BIT_TORRENT_CLIENT | choose which torrent client to use: qbittorrent or deluge | NO | deluge |
| BIT_TORRENT_USERNAME | username for qbittorrent client | NO | admin |
| WINDSCRIBE_USERNAME | username you use to login at windscribe.com/login | YES |  |
| WINDSCRIBE_PASSWORD | password you use to login at windscribe.com/login | YES |  |
| DELUGE_URL | The base URL for the bittorrent web UI | YES |  |
| DELUGE_PASSWORD | The password for the bittorrent web UI | YES |  |
| CRON_SCHEDULE | An extra cron schedule used to periodically validate and update the port if needed. Disabled if left empty | NO |  |
| DELUGE_HOST_ID | The internal host id to connect to in the deluge web UI. It will be printed in stdout after the first successful connection to deluge | Only if you have more then one connection configured in connection manager | If you have multiple configured in deluge web ui the app will print them out and crash. If you have only one that one will be used and you don't need to specify it explicitly |
| WINDSCRIBE_RETRY_DELAY | how long to wait (in milliseconds) before retrying after a windscribe error. For example a failed login. | NO | 3600000 (1 hour) |
| WINDSCRIBE_EXTRA_DELAY | how long to wait (in milliseconds) after the ephemeral port expires before trying to create a new one. | NO | 60000 (1 minute) |
| DELUGE_RETRY_DELAY | how long to wait (in milliseconds) before retrying after a bittorrent error. For example a failed login. | NO | 300000 (5 minutes) |
| CACHE_DIR | A directory where to store cached data like windscribe session cookies | NO | `/cache` in the docker container and `./cache` everywhere else |

# Running

## Using docker (and docker compose in this example)

```yaml
version: '3.8'
services:
  deluge-windscribe-ephemeral-port:
    image: dumbaspl/deluge-windscribe-ephemeral-port:4
    restart: unless-stopped
    volumes:
      - windscribe-cache:/cache
    environment:
      - WINDSCRIBE_USERNAME=<your windscribe username>
      - WINDSCRIBE_PASSWORD=<your windscribe password>
      - DELUGE_URL=<url of your bittorrent Web UI>
      - DELUGE_PASSWORD=<password for the bittorrent Web UI>

      # optional
      # - DELUGE_HOST_ID=
      # - DELUGE_RETRY_DELAY=300000
      # - WINDSCRIBE_RETRY_DELAY=3600000
      # - WINDSCRIBE_EXTRA_DELAY=60000
      # - CRON_SCHEDULE=
      # - CACHE_DIR=/cache
volumes:
  windscribe-cache:
```

## Using nodejs

**This project requires Node.js version 18 or newer**  
**This project uses [yarn](https://classic.yarnpkg.com/) to manage dependencies, make sure you have it installed first.**

1. clone this repository
2. Install dependencies by running `yarn install`
3. Create a `.env` file in the root of the project with the necessary configuration
```shell
WINDSCRIBE_USERNAME=<your windscribe username>
WINDSCRIBE_PASSWORD=<your windscribe password>
DELUGE_URL=<url of your bittorrent Web UI>
DELUGE_PASSWORD=<password for the bittorrent Web UI>

# optional
# DELUGE_HOST_ID=
# DELUGE_RETRY_DELAY=300000
# WINDSCRIBE_RETRY_DELAY=3600000
# WINDSCRIBE_EXTRA_DELAY=60000
# CRON_SCHEDULE=
# CACHE_DIR=./cache
```
4. Build and start using `yarn install`

Tip: you can use tools like pm2 to manage nodejs applications
