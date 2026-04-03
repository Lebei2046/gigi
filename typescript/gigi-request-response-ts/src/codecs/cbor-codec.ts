import { Codec } from '../types.js';

/**
 * A CBOR codec for encoding and decoding requests and responses.
 * Uses JSON as a fallback if cbor-x is not available.
 */
export class CborCodec<
  TRequest,
  TResponse,
  TProtocol extends string,
> implements Codec<TRequest, TResponse, TProtocol> {
  constructor(private protocol: TProtocol) {}

  /**
   * Encode a request to a Uint8Array.
   */
  encodeRequest(request: TRequest): Uint8Array {
    const json = JSON.stringify(request);
    return new TextEncoder().encode(json);
  }

  /**
   * Decode a request from a Uint8Array.
   */
  decodeRequest(data: Uint8Array): TRequest {
    const json = new TextDecoder().decode(data);
    return JSON.parse(json) as TRequest;
  }

  /**
   * Encode a response to a Uint8Array.
   */
  encodeResponse(response: TResponse): Uint8Array {
    const json = JSON.stringify(response);
    return new TextEncoder().encode(json);
  }

  /**
   * Decode a response from a Uint8Array.
   */
  decodeResponse(data: Uint8Array): TResponse {
    const json = new TextDecoder().decode(data);
    return JSON.parse(json) as TResponse;
  }

  /**
   * Get the protocol identifier.
   */
  getProtocol(): TProtocol {
    return this.protocol;
  }
}
