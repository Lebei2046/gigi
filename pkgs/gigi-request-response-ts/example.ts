import { createLibp2p } from 'libp2p';
import { RequestResponse, JsonCodec } from './src/index.js';

// Define request and response types
interface PingRequest {
  type: 'ping';
  timestamp: number;
}

interface PingResponse {
  type: 'pong';
  timestamp: number;
  responseTime: number;
}

async function main() {
  console.log('Starting libp2p nodes...');

  // Create two libp2p nodes
  const node1 = await createLibp2p({
    addresses: {
      listen: ['/ip4/127.0.0.1/tcp/0']
    },
    transports: [
      // Add transports here
    ],
    streamMuxers: [
      // Add stream muxers here
    ],
    connectionEncryption: [
      // Add connection encryption here
    ]
  });

  const node2 = await createLibp2p({
    addresses: {
      listen: ['/ip4/127.0.0.1/tcp/0']
    },
    transports: [
      // Add transports here
    ],
    streamMuxers: [
      // Add stream muxers here
    ],
    connectionEncryption: [
      // Add connection encryption here
    ]
  });

  console.log('Node 1 ID:', node1.peerId.toString());
  console.log('Node 2 ID:', node2.peerId.toString());

  // Create codecs
  const codec1 = new JsonCodec<PingRequest, PingResponse>('/ping/1.0.0');
  const codec2 = new JsonCodec<PingRequest, PingResponse>('/ping/1.0.0');

  // Create request-response instances
  const rr1 = new RequestResponse(node1, codec1);
  const rr2 = new RequestResponse(node2, codec2);

  // Set up event listener for node 2 to handle incoming requests
  rr2.onEvent((event) => {
    if (event.type === 'Message' && event.message.type === 'Request') {
      const { request, channel } = event.message;
      console.log('Node 2 received request:', request);

      // Create response
      const response: PingResponse = {
        type: 'pong',
        timestamp: Date.now(),
        responseTime: Date.now() - request.timestamp
      };

      // Send response
      channel.send(response);
      console.log('Node 2 sent response:', response);
    }
  });

  // Set up event listener for node 1 to handle incoming responses
  rr1.onEvent((event) => {
    if (event.type === 'Message' && event.message.type === 'Response') {
      const { response } = event.message;
      console.log('Node 1 received response:', response);
    }
  });

  // Connect node 1 to node 2
  console.log('Connecting nodes...');
  const node2Addrs = node2.getMultiaddrs();
  if (node2Addrs.length > 0) {
    await node1.dial(node2Addrs[0]);
    console.log('Nodes connected');
  } else {
    console.error('Node 2 has no addresses');
    return;
  }

  // Send ping request from node 1 to node 2
  console.log('Sending ping request...');
  const request: PingRequest = {
    type: 'ping',
    timestamp: Date.now()
  };

  const requestId = await rr1.sendRequest(node2.peerId, request);
  console.log('Sent ping request with ID:', requestId.toString());

  // Wait for a moment to allow the response to be processed
  await new Promise(resolve => setTimeout(resolve, 2000));

  // Close the request-response instances
  rr1.close();
  rr2.close();

  // Stop the libp2p nodes
  await node1.stop();
  await node2.stop();

  console.log('Done');
}

main().catch(console.error);
