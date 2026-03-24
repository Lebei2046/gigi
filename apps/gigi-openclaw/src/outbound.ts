import type { IGigiClient, GigiMessage } from "./types.js";

/**
 * Outbound message queue entry
 */
export interface OutboundMessage {
  target: string; // peerId or group:groupName
  content: string;
  timestamp: number;
  retryCount: number;
  maxRetries: number;
  resolve: (value: void) => void;
  reject: (reason: any) => void;
}

/**
 * Outbound message manager
 */
export class OutboundManager {
  private client: IGigiClient;
  private queue: OutboundMessage[] = [];
  private processing = false;
  private maxRetries = 3;
  private retryDelay = 1000;

  constructor(client: IGigiClient, options: { maxRetries?: number; retryDelay?: number } = {}) {
    this.client = client;
    this.maxRetries = options.maxRetries ?? this.maxRetries;
    this.retryDelay = options.retryDelay ?? this.retryDelay;
  }

  /**
   * Send a message to a target (peer or group)
   */
  async sendMessage(target: string, content: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const message: OutboundMessage = {
        target,
        content,
        timestamp: Date.now(),
        retryCount: 0,
        maxRetries: this.maxRetries,
        resolve,
        reject,
      };

      this.queue.push(message);
      this.processQueue();
    });
  }

  /**
   * Process the message queue
   */
  private async processQueue(): Promise<void> {
    if (this.processing || this.queue.length === 0) {
      return;
    }

    this.processing = true;

    try {
      while (this.queue.length > 0) {
        const message = this.queue.shift();
        if (!message) break;

        try {
          if (message.target.startsWith("group:")) {
            const groupName = message.target.replace("group:", "");
            await this.client.sendGroupMessage(groupName, message.content);
          } else {
            await this.client.sendMessage(message.target, message.content);
          }
          message.resolve();
        } catch (error) {
          message.retryCount++;

          if (message.retryCount < message.maxRetries) {
            // Re-queue with delay
            setTimeout(() => {
              this.queue.unshift(message);
              this.processQueue();
            }, this.retryDelay * message.retryCount);
            break; // Wait for retry
          } else {
            message.reject(new Error(`Failed after ${message.maxRetries} retries`));
          }
        }
      }
    } finally {
      this.processing = false;
    }
  }

  /**
   * Clear all pending messages
   */
  clear(): void {
    const failed = this.queue.length;
    this.queue.forEach(msg => {
      msg.reject(new Error("Message cancelled"));
    });
    this.queue = [];
    console.log(`[OutboundManager] Cleared ${failed} pending messages`);
  }

  /**
   * Get queue size
   */
  getQueueSize(): number {
    return this.queue.length;
  }
}

/**
 * Convert OpenClaw message format to Gigi message format
 */
export function toGigiMessage(
  fromAccountId: string,
  toAccountId: string,
  content: string,
  extra?: Record<string, any>
): GigiMessage {
  return {
    from: fromAccountId,
    to: toAccountId,
    content,
    timestamp: Date.now(),
    type: toAccountId.startsWith("group:") ? "broadcast" : "direct",
    ...extra,
  };
}
