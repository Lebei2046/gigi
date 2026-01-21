Let's reimplement signup and login processes using backend store

- create a store for app settings with (key, value) pair
```javascript
// Add settings interface
interface Settings {
  key: string
  value: string
  updatedAt: Date
}
```

- create a store for groups
```javascript
interface Group {
  id: string // group peer-id
  name: string
  joined: boolean // false = group creator/owner, true = invited member who joined
  createdAt: Date
}
```

- when signing up
  - generate peer id from "m/44'/60'/0'/0/0"
  - generate group id from "m/44'/60'/1'/0/0"
  - genreate wallet address for EVM
  - use the password to encrypt the mnemonics, save encrypted mnemonics, nonce, peer_id, address, nickname  as a json string with key `gigi` into the settings store
  ```javascript
  // Save to IndexedDB
  await setStorageItem('gigi', {
    nonce,
    mnemonic: cryptedMnemonic,
    address,
    peerId,
    name: state.name,
  })
  ```
  - save group info into the group store
  ```javascript
  // Derive group peer ID from user's mnemonic
  const groupPeerId = await generateGroupPeerId(state.mnemonic)

  // Save group to IndexedDB
  await db.groups.add({
    id: groupPeerId,
    name: state.groupName.trim(),
    joined: false, // false = group creator/owner, true = invited member
    createdAt: new Date(),
  })
  ```  

- when logining
  - retrieve the key `gigi` from the settings store
  ```javascript
      const gigiData = await getStorageItem<{
      mnemonic?: string
      nonce?: string
      address?: string
      peerId?: string
      name?: string
    }>('gigi')

  ```
  - use the unlock password to decrypt the mnemonics, identify the peer id derived from the mnemonics with the peer id stored in the settings store.
  ```javascript
      const decryptedMnemonics = decryptMnemonics(
        state.mnemonic,
        password,
        state.nonce
      )
      const generatedAddress = getAddress(decryptedMnemonics)

      if (generatedAddress !== state.address) {
        return {
          success: false,
          error: 'Password is incorrect, please re-enter!',
        }
      }

      // Extract private key and initialize P2P
      const privateKey = getPrivateKeyFromMnemonic(decryptedMnemonics)
      // Use the stored name as nickname, fallback to "Anonymous" if not available
      const nickname = state.name || 'Anonymous'
      const peerId = await MessagingClient.initializeWithKey(
        privateKey,
        nickname
      )

      dispatch(login({ password }))
  ```
