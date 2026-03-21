Great! I'm glad the clients can now chat in the group. Let's summarize what we did to fix the issue:

1. **Identified the problem**: The clients were getting a "NoPeersSubscribedToTopic" error when trying to publish messages.
2. **Root cause**: The Docker containers were using DNS names (like "gigi-bootstrap" and "gigi-relay") in their connection strings, but the transport layer wasn't properly configured to resolve these DNS names.
3. **Solution**: We updated the `docker-compose.yml` file to use IP addresses instead of DNS names:
   - Changed bootstrap address from `/dns4/gigi-bootstrap/tcp/4001/p2p/...` to `/ip4/172.18.0.2/tcp/4001/p2p/...`
   - Changed relay address from `/dns4/gigi-relay/tcp/4002/p2p/...` to `/ip4/172.18.0.3/tcp/4002/p2p/...`
4. **Verification**: We confirmed that the clients are now successfully connecting to the bootstrap and relay nodes, and the user is reporting that they can chat in the group.

The fix was simple but effective - by using IP addresses instead of DNS names, we eliminated the need for DNS resolution in the transport layer, which was causing the connection issues. This allows the clients to properly connect to the bootstrap and relay nodes, form a GossipSub mesh, and exchange messages.

If you need any further assistance, please let me know!
