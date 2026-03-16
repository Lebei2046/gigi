export enum ErrorCode {
  NOT_STARTED = 'ERR_NOT_STARTED',
  ALREADY_STARTED = 'ERR_ALREADY_STARTED',
  NETWORK_ERROR = 'ERR_NETWORK_ERROR',
  PEER_NOT_FOUND = 'ERR_PEER_NOT_FOUND',
  GROUP_NOT_FOUND = 'ERR_GROUP_NOT_FOUND',
  FILE_NOT_FOUND = 'ERR_FILE_NOT_FOUND',
  FILE_SHARE_ERROR = 'ERR_FILE_SHARE_ERROR',
  DOWNLOAD_ERROR = 'ERR_DOWNLOAD_ERROR',
  INVALID_MESSAGE = 'ERR_INVALID_MESSAGE',
  CONNECTION_FAILED = 'ERR_CONNECTION_FAILED',
  TIMEOUT = 'ERR_TIMEOUT',
  PROTOCOL_ERROR = 'ERR_PROTOCOL_ERROR',
  PERSISTENCE_ERROR = 'ERR_PERSISTENCE_ERROR',
}

export class P2pError extends Error {
  public readonly code: ErrorCode;
  public readonly cause?: Error;

  constructor(code: ErrorCode, message: string, cause?: Error) {
    super(message);
    this.name = 'P2pError';
    this.code = code;
    this.cause = cause;
    Error.captureStackTrace(this, this.constructor);
  }

  static notStarted(message = 'P2P client not started'): P2pError {
    return new P2pError(ErrorCode.NOT_STARTED, message);
  }

  static alreadyStarted(message = 'P2P client already started'): P2pError {
    return new P2pError(ErrorCode.ALREADY_STARTED, message);
  }

  static networkError(message: string, cause?: Error): P2pError {
    return new P2pError(ErrorCode.NETWORK_ERROR, message, cause);
  }

  static peerNotFound(peerId: string): P2pError {
    return new P2pError(ErrorCode.PEER_NOT_FOUND, `Peer not found: ${peerId}`);
  }

  static groupNotFound(groupName: string): P2pError {
    return new P2pError(ErrorCode.GROUP_NOT_FOUND, `Group not found: ${groupName}`);
  }

  static fileNotFound(shareCode: string): P2pError {
    return new P2pError(ErrorCode.FILE_NOT_FOUND, `File not found: ${shareCode}`);
  }

  static fileShareError(message: string, cause?: Error): P2pError {
    return new P2pError(ErrorCode.FILE_SHARE_ERROR, message, cause);
  }

  static downloadError(message: string, cause?: Error): P2pError {
    return new P2pError(ErrorCode.DOWNLOAD_ERROR, message, cause);
  }

  static invalidMessage(message: string): P2pError {
    return new P2pError(ErrorCode.INVALID_MESSAGE, message);
  }

  static connectionFailed(peerId: string, cause?: Error): P2pError {
    return new P2pError(ErrorCode.CONNECTION_FAILED, `Failed to connect to ${peerId}`, cause);
  }

  static timeout(operation: string): P2pError {
    return new P2pError(ErrorCode.TIMEOUT, `Operation timed out: ${operation}`);
  }
}