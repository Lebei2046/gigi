// Gigi DNS - Auto-discovery protocol for local networks with nicknames
//
// This module exports all the public types and classes for the Gigi DNS protocol.

// Types
export * from './types';

// Protocol
export * from './protocol';

// Behaviour
export * from './behaviour';

// Service name constants
export const SERVICE_NAME = Buffer.from('_gigi-dns._udp.local');
export const SERVICE_NAME_FQDN = '_gigi-dns._udp.local.';
