import { QBittorrent } from '@ctrl/qbittorrent';

export class QBittorrentClient {

  private qbittorrent: QBittorrent;
  //private currentHost?: string;

  constructor(
    url: string,
    password: string,
    private defaultHostId?: string,
  ) {
    this.qbittorrent = new QBittorrent({
      baseUrl: url,
      username: "admin",
      password: password,
    });
  }

  async updateConnection(): Promise<Boolean> {
    // session check
    //if (!await this.qbittorrent.checkSession()) {
    // login if not logged in already
    if (!await this.qbittorrent.login()) {
      throw new Error('Failed to connect to qbittorrent');
    }
    return true;
    //}

    // connection check
    // if (!this.currentHost || !await this.qbittorrent.connected()) {
    //   const { result: hosts, error } = await this.qbittorrent.getHosts();

    //   if (error) {
    //     throw new Error(`Deluge getHosts error: ${error}`);
    //   }

    //   if (hosts.length == 0) {
    //     throw new Error('No qbittorrent hosts available');
    //   }

    //   let hostId = this.currentHost || this.defaultHostId;
    //   if (hostId) {
    //     // make sure the host actually exists
    //     if (!hosts.some(host => host[0] == hostId)) {
    //       throw new Error(`Deluge host with id ${hostId} does not exist`);
    //     }
    //   } else {
    //     if (hosts.length == 1) {
    //       // if we have a single host, just use it
    //       hostId = hosts[0][0];
    //       console.log(`Selecting the only available qbittorrent host: ${hostId}`);
    //     } else {
    //       console.log(
    //         `Found ${hosts.length} qbittorrent hosts(id: host:port - status): \n` +
    //         hosts
    //           .map(host => `\t${host[0]}: ${host[1]}:${host[2]} - ${host[3]}`)
    //           .join('\n')
    //       );
    //       throw new Error(`Found more than one qbittorrent host, select one via DELUGE_HOST_ID env variable`);
    //     }
    //   }

    //   // try to connect if not connected already
    //   await this.qbittorrent.connect(hostId);
    //   this.currentHost = hostId;
    // }

    // // check the status of the current host
    // const { result: {
    //   [0]: hostId,
    //   [1]: status,
    //   [2]: version,
    // }, error } = await this.qbittorrent.getHostStatus(this.currentHost);

    // if (error) {
    //   throw new Error(`Deluge getHostStatus error: ${error}`);
    // }

    // // this should never fail in theory
    // if (status != 'Connected') {
    //   throw new Error('Not connected to qbittorrent');
    // }

    // // report status
    // return {
    //   hostId,
    //   version,
    // };
  }

  async getPort() {
    // make sure we are connected
    await this.updateConnection();

    const { listen_port } = await this.qbittorrent.getPreferences();

    if (!listen_port) {
      throw new Error(`QBittorrent getPreferences->listen_port error`);
    }

    return listen_port;
  }

  async updatePort(port: number): Promise<void> {
    // make sure we are connected
    await this.updateConnection();

    // update port
    await this.qbittorrent.setPreferences({
      listen_port: port,
      random_port: false, // turn of random port as well
    });

    console.log('QBittorrent port successfully updated');
  }

}
