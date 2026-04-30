# Implement group sharing in gigi-dioxus

## On the sender side

1. click the button in the red circle on the picture, pop up a window
2. list the nodes online in the window
3. pick up the nodes to share
4. send group sharing messages to the nodes picked

## On the receiver side

When receiveing a group sharing message

1. save the message in db
2. add a record in the conversion list
3. when the user clicks the record to enter into the chat room, list the message
4. when the user clicks the message, pop up a window to provide options to join the group or ignore the message
5. if the user picks joining the group, add the group to db with `joined` of `true` and subscribe to the group
6. if the user picks ignoring the group,  remove the message from db

When the user returns back to the conversasion list, if the user added the group, list the group on the conversation list.
