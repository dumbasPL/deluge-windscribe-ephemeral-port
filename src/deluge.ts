import {Deluge} from '@ctrl/deluge';

// env variables

const url = process.env.DELUGE_URL;
const password = process.env.DELUGE_PASSWORD;

if (!url || url.length == 0) {
  console.log('Missing environment variable DELUGE_URL');
  process.exit(1);
}

if (!password || password.length == 0) {
  console.log('Missing environment variable DELUGE_PASSWORD');
  process.exit(1);
}

// functions

export async function testDelugeConnection() {
  try {
    const client = new Deluge({
      baseUrl: url,
      password: password,
    });

    await client.connect();
  } catch (error) {
    throw new Error(`Failed to connect to deluge: ${error.message}`);
  }
}

export async function updateDelugePort(port: number) {
  try {
    const client = new Deluge({
      baseUrl: url,
      password: password,
    });

    const res = await client.setConfig({
      listen_ports: [port, port],
      random_port: false, // turn of random port as well
    });

    if (res.error) {
      throw new Error(res.error);
    }
    console.log('Deluge port successfully updated');
  } catch (error) {
    throw new Error(`Failed to update deluge port: ${error.message}`);
  }
}
