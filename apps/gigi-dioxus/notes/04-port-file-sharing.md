# Port file sharing from gigi-mobile to gigi-dioxus

## Analyze gigi-mobile

- the flow of file sharing
- the component for rendering of file message or image message(sender side)
- the component for  rendering of file message or image message(receiver side)

## Port to gigi-dioxus

### the sender side

- pick a file or an image
- calculate the hash of file, save file info to db
- for an image, make a thumbnail, save it into GIGI_DATA_DIR/uploads
- create and send a file sharing message to p2p network
- render the message to message list as the thumbnail(images)
- render the message to message list as the placeholder with file info(files)

### the receiver side
- when receiving a file sharing message, launch the downloading process
- render the received message in progress when downloading
- when downloading finished, calculate the hash of file, save the file into GIGI_DATA_DIR/downloads
- for an image, make a thumbnail, save it into GIGI_DATA_DIR/downloads
- render the message as the thumbnail(images)
- render the message to message list as the placeholder with file info(files)

**Be careful to write the message rendering  components for sender and receiver in dioxus**

## Build and Debug

- dx build --desktop, build the app
- GIGI_DATA_DIR=~/.gigi-dioxus-alice dx serve --desktop, launch Alice
- GIGI_DATA_DIR=~/.gigi-dioxus-bob dx serve --desktop, launch Bob
