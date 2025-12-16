# Task 1 - Create a chat group when signing up

- in `SignupInfoInput` component,  if the user  click `Create the first chat group` check box, enable the input for the first chat group name
- from the mnemonics to derive the peer id of the group as the group id
  - the private key of hdKey.derive("m/44'/60'/1'/0/0")
  - call `try_get_peer_id` to get the peer_id
- flag `joined` as false, use current date as `created_at`
- save the chat group: `id`, `name`, `joined`, `created_at` in `groups` table in indexedDB of `GigiDatabase`, in `SignupFinish` page

# Task 2 - Save the latest chat message in indexedDB

When coming back from `ChatRoom` to `peer list` page, the latest chat message not displayed anymore.

- save the latest chat message in indexedDB
- define the structure of `chats` table for the latest chat message in indexedDB
- display the latest chat message in `peer list` page if any
  
# Task 3 - Share the chat group with other peers in the rust backend

- implement function `send_direct_share_group_message` in `gigi-p2p` crate
- implement command `messaging_send_direct_share_group_message` in the rust backend
- implement frontend API and events for `send_direct_share_group_message`

# Task 4 - Share the chat group with other peers in the frontend

**For sender**

- rename `peer list` page to `Chats` page
- in `Chats` page, also list the chat groups that the user has created or joined
- in `Chats` page, create a context menu with `Share` button
- when user click the `Share` button, display a drawer from bottom to pick other peers of online to share the chat group with
- send share group message to the selected peers

**For receiver**

- in `Chats` page, display a notification to the user when receiving a share group message
- when clicking the notification, open the chat room of the sender
- with the received share group message, pick `Ignore` to ignore the message
- with the received share group message, pick `Accept` to accept the message, and save the chat group in indexedDB with `joined` as true



We now have implemented the direct chat function, but we need to add a group chat function. We have direct chat persistence to local storage, but we need to add group chat persistence to indexedDB. So, we need to design a unified structure of chat persistence to indexedDB and transfer the direct chat persistence from local storage to indexedDB. 

# Task 1 - Design a unified structure of chat persistence of indexedDB

- check `src/models/db.ts` to see the structure of the unified chat persistence whether it is correct
- give a brief description of the unified structure of chat persistence of indexedDB

# Task 2 - Transfer the direct chat persistence from local storage to indexedDB

# Task 3 - Implement the creation of group for chat from mnemonics
