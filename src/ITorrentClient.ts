export interface ITorrentClient {
    updateConnection(): Promise<{ hostId: string; version: string; }>;
    getPort(): Promise<number>;
    updatePort(port: number): Promise<void>;
}