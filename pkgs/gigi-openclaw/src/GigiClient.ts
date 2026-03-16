import { createLibp2p } from "libp2p";
import { webSockets } from "@libp2p/websockets";
import { webTransport } from "@libp2p/webtransport";
import { mdns } from "@libp2p/mdns";
import { kadDHT } from "@libp2p/kad-dht";
import { Multiaddr } from "@multiformats/multiaddr";
import type { Libp2p } from "libp2p";
import type { IGigiClient, GigiClientConfig, GigiMessage } from "./types.js";

export class GigiClient implements IGigiClient {
  private libp2p: Libp2p | null = null;
  private messageHandlers: ((msg: GigiMessage) => void)[] = [];
  private config: GigiClientConfig;
  private started = false;

  private readonly GIGI_PROTOCOL = "/gigi/direct/1.0.0";

  constructor(config: GigiClientConfig) {
    this.config = config;
  }

  async start(): Promise<void> {
    if (this.started) {
      throw new Error("GigiClient already started");
    }

    // Create libp2p node
    this.libp2p = await createLibp2p({
      addresses: {
        listen: this.config.multiaddrs,
      },
      transports: [webSockets(), webTransport()],
      peerDiscovery: this.config.enableMdns !== false ? [mdns()] : [],
      dht: this.config.enableDht !== false ? kadDHT() : undefined,
    });

    // Set up protocol handler
    await this.libp2p.handle(this.GIGI_PROTOCOL, async ({ stream, connection }) => {
      const fromPeerId = connection.remotePeer.toString();
      
      try {
        // Read message from stream
        const data = [];
        for await (const chunk of stream) {
          data.push(chunk);
        }
        
        const messageText = new TextDecoder().decode(Uint8Array.from(data));
        const message: GigiMessage = JSON.parse(messageText);
        
        // Emit to handlers
        this.emitMessage(message);
      } catch (error) {
        console.error(`[GigiClient] Error handling message from ${fromPeerId}:`, error);
      }
    });

    // Start libp2p node
    await this.libp2p.start();
    this.started = true;

    console.log(`[GigiClient] Started with peer ID: ${this.libp2p.peerId.toString()}`);
    console.log(`[GigiClient] Listening on: ${this.libp2p.getMultiaddrs().map(m => m.toString()).join(", ")}`);

    // Connect to bootstrap peers if specified
    if (this.config.bootstrapPeers) {
      for (const addr of this.config.bootstrapPeers) {
        try {
          const multiaddr = new Multiaddr(addr);
          await this.libp2p.dial(multiaddr);
          console.log(`[GigiClient] Connected to bootstrap peer: ${addr}`);
        } catch (error) {
          console.error(`[GigiClient] Failed to connect to bootstrap peer ${addr}:`, error);
        }
      }
    }
  }

  async stop(): Promise<void> {
    if (!this.started || !this.libp2p) {
      return;
    }

    await this.libp2p.stop();
    this.started = false;
    this.libp2p = null;
    console.log("[GigiClient] Stopped");
  }

  async sendMessage(targetPeerId: string, content: string): Promise<void> {
    if (!this.started || !this.libp2p) {
      throw new Error("GigiClient not started");
    }

    const message: GigiMessage = {
      from: this.getPeerId(),
      to: targetPeerId,
      content,
      timestamp: Date.now(),
      type: "direct",
    };

    const messageText = JSON.stringify(message);
    const messageBytes = new TextEncoder().encode(messageText);

    try {
      // Open stream to target peer
      const stream = await this.libp2p.dialProtocol(targetPeerId, this.GIGI_PROTOCOL);
      
      // Write message to stream
      const writer = stream.sink([messageBytes]);
      
      // Wait for write to complete
      for await (const _ of writer) {
        // Sink is done
      }
      
      console.log(`[GigiClient] Sent message to ${targetPeerId}`);
    } catch (error) {
      console.error(`[GigiClient] Failed to send message to ${targetPeerId}:`, error);
      throw error;
    }
  }

  onMessage(handler: (msg: GigiMessage) => void): void {
    this.messageHandlers.push(handler);
  }

  private emitMessage(message: GigiMessage): void {
    for (const handler of this.messageHandlers) {
      try {
        handler(message);
      } catch (error) {
        console.error("[GigiClient] Error in message handler:", error);
      }
    }
  }

  getPeerId(): string {
    if (!this.libp2p) {
      throw new Error("GigiClient not started");
    }
    return this.libp2p.peerId.toString();
  }

  getMultiaddrs(): string[] {
    if (!this.libp2p) {
      throw new Error("GigiClient not started");
    }
    return this.libp2p.getMultiaddrs().map(m => m.toString());
  }

  isConnected(): boolean {
    return this.started && this.libp2p?.status === "started";
  }
}
