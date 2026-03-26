import { Codec } from '../types.js';

/**
 * A JSON codec for encoding and decoding requests and responses.
 * Handles Uint8Array objects by converting them to base64 strings.
 */
export class JsonCodec<TRequest, TResponse, TProtocol extends string> implements Codec<TRequest, TResponse, TProtocol> {
  constructor(private protocol: TProtocol) {}

  /**
   * Recursively convert Uint8Array to base64 strings in an object.
   */
  private serialize(obj: any): any {
    if (obj instanceof Uint8Array) {
      return {
        type: 'Uint8Array',
        data: Buffer.from(obj).toString('base64')
      };
    } else if (Array.isArray(obj)) {
      return obj.map(item => this.serialize(item));
    } else if (obj && typeof obj === 'object') {
      const result: any = {};
      for (const key in obj) {
        if (Object.prototype.hasOwnProperty.call(obj, key)) {
          result[key] = this.serialize(obj[key]);
        }
      }
      return result;
    } else {
      return obj;
    }
  }

  /**
   * Recursively convert base64 strings back to Uint8Array in an object.
   */
  private deserialize(obj: any): any {
    if (obj && typeof obj === 'object' && obj.type === 'Uint8Array' && obj.data) {
      return new Uint8Array(Buffer.from(obj.data, 'base64'));
    } else if (Array.isArray(obj)) {
      return obj.map(item => this.deserialize(item));
    } else if (obj && typeof obj === 'object') {
      const result: any = {};
      for (const key in obj) {
        if (Object.prototype.hasOwnProperty.call(obj, key)) {
          result[key] = this.deserialize(obj[key]);
        }
      }
      return result;
    } else {
      return obj;
    }
  }

  /**
   * Encode a request to a Uint8Array.
   */
  encodeRequest(request: TRequest): Uint8Array {
    const serialized = this.serialize(request);
    const json = JSON.stringify(serialized);
    return new TextEncoder().encode(json);
  }

  /**
   * Decode a request from a Uint8Array.
   */
  decodeRequest(data: Uint8Array): TRequest {
    const json = new TextDecoder().decode(data);
    const parsed = JSON.parse(json);
    return this.deserialize(parsed) as TRequest;
  }

  /**
   * Encode a response to a Uint8Array.
   */
  encodeResponse(response: TResponse): Uint8Array {
    const serialized = this.serialize(response);
    const json = JSON.stringify(serialized);
    return new TextEncoder().encode(json);
  }

  /**
   * Decode a response from a Uint8Array.
   */
  decodeResponse(data: Uint8Array): TResponse {
    const json = new TextDecoder().decode(data);
    const parsed = JSON.parse(json);
    return this.deserialize(parsed) as TResponse;
  }

  /**
   * Get the protocol identifier.
   */
  getProtocol(): TProtocol {
    return this.protocol;
  }
}
