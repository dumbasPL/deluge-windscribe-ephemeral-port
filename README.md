# [deluge-windscribe-ephemeral-port](https://github.com/dumbasPL/deluge-windscribe-ephemeral-port)

Automatically create ephemeral ports in windscribe and update deluge config to use the new port

## Important information

This project was designed to work along side containers like [kabe0/deluge-windscribe](https://github.com/Kabe0/deluge-windscribe) in mind.  
It will not help you configure windscribe to use a vpn!  
It will only update the port that deluge listens on to the same port that's configured on windscribe website.

**I strongly advise against using a "restart on error" policy since windscribe will temporary block your ip address after a few failed login attempts.**

# Running

## Using docker (and docker compose in this example)
```yml
version: '3'
services:
  deluge-windscribe-ephemeral-port:
    image: dumbaspl/deluge-windscribe-ephemeral-port
    restart: unless-stopped
    environment:
      - WINDSCRIBE_USERNAME=<your windscribe username>
      - WINDSCRIBE_PASSWORD=<your windscribe password>
      - DELUGE_URL=<url of your Deluge Web UI>
      - DELUGE_PASSWORD=<password for the Deluge Web UI>
```

## Using nodejs
Tested on node 16 but should work on node 14 as well.  
This project uses [yarn](https://classic.yarnpkg.com/) to manage dependencies, make sure you have it installed first.

1. Install dependencies by running `yarn`
2. Create a `.env` file with the necessary configuration
```
WINDSCRIBE_USERNAME=<your windscribe username>
WINDSCRIBE_PASSWORD=<your windscribe password>
DELUGE_URL=<url of your Deluge Web UI>
DELUGE_PASSWORD=<password for the Deluge Web UI>
```
3. Start using `yarn start`

Tip: you can use tools like [pm2](https://www.npmjs.com/package/pm2) to manage nodejs applications
