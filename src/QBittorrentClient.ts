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
    // Unfortunately QBitTorrent does not have a session check
    //if (!await this.qbittorrent.checkSession()) {
    // login if not logged in already
    if (!await this.qbittorrent.login()) {
      throw new Error('Failed to connect to qbittorrent');
    }
    return true;
  }

  async getPort() {
    // make sure we are connected
    await this.updateConnection();

    const { listen_port } = await this.qbittorrent.getPreferences();
    console.log('got listen_port from preferences: ' + listen_port)
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
