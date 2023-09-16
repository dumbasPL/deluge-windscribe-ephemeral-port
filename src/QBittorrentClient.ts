import { QBittorrent } from '@ctrl/qbittorrent';
import { ITorrentClient } from './ITorrentClient.js';

export class QBittorrentClient implements ITorrentClient {

  private qbittorrent: QBittorrent;

  constructor(
    url: string,
    username: string,
    password: string,
  ) {
    this.qbittorrent = new QBittorrent({
      baseUrl: url,
      username: username,
      password: password,
    });
  }

  async updateConnection(): Promise<{ hostId: string; version: string; }> {
    // Unfortunately QBitTorrent does not have a session check
    //if (!await this.qbittorrent.checkSession()) {
    // attempt to login
    if (!await this.qbittorrent.login()) {
      throw new Error('Failed to connect to qbittorrent');
    }

    // HostId does not exist in @ctrl/qbittorrent
    // Version not implemented because unused by downstream dependencies
    let hostId = "N/A"
    let version = "1"
    return {
      hostId,
      version,
    };
  }

  async getPort(): Promise<number> {
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
