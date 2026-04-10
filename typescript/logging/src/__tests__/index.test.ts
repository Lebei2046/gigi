vi.mock("pino", () => {
  const mockLogger = {
    info: vi.fn(),
    debug: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
  };

  const mockPino = vi.fn(() => mockLogger);
  mockPino.stdSerializers = {
    err: vi.fn((error: unknown) => error),
  };

  return {
    default: mockPino,
    stdSerializers: mockPino.stdSerializers,
  };
});

import { createLogger } from "../index";
import pino from "pino";

describe("createLogger", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Clear LOG_LEVEL to avoid interference from environment
    delete process.env.LOG_LEVEL;
  });

  it("should create a logger with default options", () => {
    createLogger();
    expect(pino).toHaveBeenCalledWith({
      level: "debug",
      name: "gigi",
      serializers: expect.objectContaining({
        peerId: expect.any(Function),
        multiaddr: expect.any(Function),
        error: pino.stdSerializers.err,
      }),
    });
  });

  it("should create a logger with custom options", () => {
    createLogger({
      level: "info",
      name: "test-service",
    });
    expect(pino).toHaveBeenCalledWith({
      level: "info",
      name: "test-service",
      serializers: expect.objectContaining({
        peerId: expect.any(Function),
        multiaddr: expect.any(Function),
        error: pino.stdSerializers.err,
      }),
    });
  });

  it("should use production level when NODE_ENV is production", () => {
    process.env.NODE_ENV = "production";
    createLogger();
    expect(pino).toHaveBeenCalledWith({
      level: "info",
      name: "gigi",
      serializers: expect.any(Object),
    });
    delete process.env.NODE_ENV;
  });

  it("should test peerId serializer", () => {
    createLogger();
    const mockPeerId = {
      toString: () => "12D3KooW...",
    };

    // Access the serializer from the pino call
    const pinoCallArgs = (pino as jest.Mock).mock.calls[0][0];
    const peerIdSerializer = pinoCallArgs.serializers.peerId;

    expect(peerIdSerializer(mockPeerId)).toBe("12D3KooW...");
    expect(peerIdSerializer(null)).toBe(null);
    expect(peerIdSerializer(undefined)).toBe(null);
  });

  it("should test multiaddr serializer", () => {
    createLogger();
    const mockMultiaddr = {
      toString: () => "/ip4/127.0.0.1/tcp/8080",
    };

    // Access the serializer from the pino call
    const pinoCallArgs = (pino as jest.Mock).mock.calls[0][0];
    const multiaddrSerializer = pinoCallArgs.serializers.multiaddr;

    expect(multiaddrSerializer(mockMultiaddr)).toBe("/ip4/127.0.0.1/tcp/8080");
    expect(multiaddrSerializer(null)).toBe(null);
    expect(multiaddrSerializer(undefined)).toBe(null);
  });
});
