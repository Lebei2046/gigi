import pino, { Logger } from "pino";

export interface LoggerOptions {
  level?: string;
  name?: string;
  serializers?: Record<string, pino.SerializerFn>;
}

export const createLogger = (options: LoggerOptions = {}): Logger => {
  const {
    level = process.env.LOG_LEVEL ||
      (process.env.NODE_ENV === "production" ? "info" : "debug"),
    name = "gigi",
    serializers: customSerializers,
  } = options;

  return pino({
    level,
    name,
    ...options,
    serializers: {
      peerId: (peerId: { toString: () => string } | null | undefined) =>
        peerId?.toString() || null,
      multiaddr: (addr: { toString: () => string } | null | undefined) =>
        addr?.toString() || null,
      error: pino.stdSerializers.err,
      ...customSerializers,
    },
  });
};

export default createLogger;
