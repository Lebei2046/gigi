import { Codec } from '../types.js';
import cbor from 'cbor-x';

/**
 * A CBOR codec for encoding and decoding requests and responses.
 * Uses cbor-x for efficient binary serialization.
 */
export class CborCodec<TRequest, TResponse, TProtocol extends string> implements Codec<TRequest, TResponse, TProtocol> {
  constructor(private protocol: TProtocol) {}

  /**
   * Encode a request to a Uint8Array.
   */
  encodeRequest(request: TRequest): Uint8Array {
    return cbor.encode(request);
  }

  /**
   * Decode a request from a Uint8Array.
   */
  decodeRequest(data: Uint8Array): TRequest {
    return cbor.decode(data) as TRequest;
  }

  /**
   * Encode a response to a Uint8Array.
   */
  encodeResponse(response: TResponse): Uint8Array {
    return cbor.encode(response);
  }

  /**
   * Decode a response from a Uint8Array.
   */
  decodeResponse(data: Uint8Array): TResponse {
    return cbor.decode(data) as TResponse;
  }

  /**
   * Get the protocol identifier.
   */
  getProtocol(): TProtocol {
    return this.protocol;
  }
}
