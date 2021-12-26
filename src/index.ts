import 'dotenv/config';
import {updateDelugePort} from './deluge';
import {getMyAccountCsrfToken, getPortForwardingInfo, login, removeEphemeralPort, requestMatchingEphemeralPort} from './windscribe';

async function run() {
  try {
    // get a new session each time
    const sessionCookie = await login();

    // get csrf token and time to pass on to future requests
    const csrfToken = await getMyAccountCsrfToken(sessionCookie);

    // check for current status
    let portForwardingInfo = await getPortForwardingInfo(sessionCookie);

    // check for mismatched ports if any present
    if (portForwardingInfo.ports.length == 2 && portForwardingInfo.ports[0] != portForwardingInfo.ports[1]) {
      console.log('detected mismatched ports, removing existing ports');
      await removeEphemeralPort(sessionCookie, csrfToken);

      // update data to match current state
      portForwardingInfo.ports = [];
      portForwardingInfo.epfExpires = 0;
    }

    // request new port of we don't have any
    if (portForwardingInfo.epfExpires == 0) {
      console.log('no port configured, Requesting new matching ephemeral port');
      portForwardingInfo = await requestMatchingEphemeralPort(sessionCookie, csrfToken);
    } else {
      console.log(`Using existing ephemeral port: ${portForwardingInfo.ports[0]}`);
    }

    // update deluge with new port
    console.log('Updating deluge');
    await updateDelugePort(portForwardingInfo.ports[0]);

    // schedule next run in 7 days since the time of creation (+ 1 minute just to be sure)
    // (this code is copied form the windscribe website btw)
    const expiresAt = new Date((portForwardingInfo.epfExpires + 86400 * 7) * 1000 + 60000);
    const diff = expiresAt.getTime() - new Date().getTime(); // time difference in milliseconds
    setTimeout(run, diff);
    console.log(`Port expires in ${Math.floor(diff/1000)} seconds. Next run scheduled at ${expiresAt.toLocaleString()}`);
  } catch (error) {
    console.error(error);
    // just kill the process on error
    process.exit(1);
  }
}

run();
