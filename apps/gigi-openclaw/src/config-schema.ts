import { Type, Static } from '@sinclair/typebox';

/**
 * Channel configuration schema for OpenClaw
 */
export const ChannelConfigSchema = Type.Object({
  accounts: Type.Optional(
    Type.Record(
      Type.String(),
      Type.Object({
        peerId: Type.String(),
        multiaddrs: Type.Array(Type.String()),
        displayName: Type.Optional(Type.String()),
      })
    )
  ),
});

export type ChannelConfig = Static<typeof ChannelConfigSchema>;

/**
 * Account configuration schema
 */
export const AccountConfigSchema = Type.Object({
  peerId: Type.String(),
  multiaddrs: Type.Array(Type.String()),
  displayName: Type.Optional(Type.String()),
});

export type AccountConfig = Static<typeof AccountConfigSchema>;

/**
 * Gateway configuration schema
 */
export const GatewayConfigSchema = Type.Object({
  url: Type.String({
    default: 'ws://127.0.0.1:18789',
  }),
  token: Type.Optional(Type.String()),
  autoConnect: Type.Boolean({ default: true }),
  reconnectInterval: Type.Number({ default: 5000 }),
});

export type GatewayOptions = Static<typeof GatewayConfigSchema>;
