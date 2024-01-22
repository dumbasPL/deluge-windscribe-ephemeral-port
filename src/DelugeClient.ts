import { Deluge } from '@ctrl/deluge';
import { ITorrentClient } from './ITorrentClient.js';

export class DelugeClient implements ITorrentClient {

  private deluge: Deluge;
  private currentHost?: string;

  constructor(
    url: string,
    password: string,
    private defaultHostId?: string,
  ) {
    this.deluge = new Deluge({
      baseUrl: url,
      password: password,
    });
  }

  async updateConnection(): Promise<{ hostId: string; version: string; }> {
    // session check
    if (!await this.deluge.checkSession()) {
      // login if not logged in already
      if (!await this.deluge.login()) {
        throw new Error('Failed to connect to deluge');
      }
    }

    // connection check
    if (!this.currentHost || !await this.deluge.connected()) {
      const { result: hosts, error } = await this.deluge.getHosts();

      if (error) {
        throw new Error(`Deluge getHosts error: ${error}`);
      }

      if (hosts.length == 0) {
        throw new Error('No deluge hosts available');
      }

      let hostId = this.currentHost || this.defaultHostId;
      if (hostId) {
        // make sure the host actually exists
        if (!hosts.some(host => host[0] == hostId)) {
          throw new Error(`Deluge host with id ${hostId} does not exist`);
        }
      } else {
        if (hosts.length == 1) {
          // if we have a single host, just use it
          hostId = hosts[0][0];
          console.log(`Selecting the only available deluge host: ${hostId}`);
        } else {
          console.log(
            `Found ${hosts.length} deluge hosts(id: host:port - status): \n` +
            hosts
              .map(host => `\t${host[0]}: ${host[1]}:${host[2]} - ${host[3]}`)
              .join('\n')
          );
          throw new Error(`Found more than one deluge host, select one via DELUGE_HOST_ID env variable`);
        }
      }

      // try to connect if not connected already
      await this.deluge.connect(hostId);
      this.currentHost = hostId;
    }

    // check the status of the current host
    const { result: {
      [0]: hostId,
      [1]: status,
      [2]: version,
    }, error } = await this.deluge.getHostStatus(this.currentHost);

    if (error) {
      throw new Error(`Deluge getHostStatus error: ${error}`);
    }

    // this should never fail in theory
    if (status != 'Connected' && status != 'Online') {
      throw new Error('Not connected to deluge');
    }

    // report status
    return {
      hostId,
      version,
    };
  }

  async getPort(): Promise<number> {
    // make sure we are connected
    await this.updateConnection();

    const { error, result: config } = await this.deluge.getConfig();

    if (error) {
      throw new Error(`Deluge getConfig error: ${error}`);
    }

    return config.random_port ? 0 : config.listen_ports[0];
  }

  async updatePort(port: number): Promise<void> {
    // make sure we are connected
    await this.updateConnection();

    // update port
    const { error } = await this.deluge.setConfig({
      listen_ports: [port, port],
      random_port: false, // turn of random port as well
    });

    if (error) {
      throw new Error(`Deluge setConfig error: ${error}`);
    }

    console.log('Deluge port successfully updated');
  }

}
