# [deluge-windscribe-ephemeral-port](https://github.com/dumbasPL/deluge-windscribe-ephemeral-port)

Automatically create ephemeral ports in windscribe and update deluge config to use the new port

## Important information

This project was designed to work along side containers like [kabe0/deluge-windscribe](https://github.com/Kabe0/deluge-windscribe) in mind.  
It will not help you configure windscribe to use a vpn!  
It will only update the port that deluge listens on to the same port that's configured on windscribe website.

**I strongly advise against using a "restart on error" policy since windscribe will temporary block your ip address after a few failed login attempts.**

# Running

TODO