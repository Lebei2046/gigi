<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		subscribeTopic,
		unsubscribeTopic,
		sendMessage,
		onMessageReceived,
		onPeerDiscovered
	} from 'tauri-plugin-libp2p-messaging-api';

	let topic = 'test-net';
	let name = '';
	let greetMsg = '';
	let msgHandle: any = undefined;
	let peerHandle: any = undefined;

	onMount(async () => {
		console.log(`Subscribing to topic: ${topic}`);
		await subscribeTopic(topic);
		console.log(`Subscribed to topic: ${topic}`);
		msgHandle = onMessageReceived((message) => {
			console.log(message);
			greetMsg = JSON.stringify(message, null, 2);
		});
		peerHandle = onPeerDiscovered(async (peer) => {
			console.log(peer);
		});
	});

	onDestroy(() => {
		// unsubscribeTopic(topic);
		if (msgHandle) {
			msgHandle();
		}
		if (peerHandle) {
			peerHandle();
		}
	});

	async function send() {
		console.log(`Message sending to ${topic}: ${name}`);
		await sendMessage(topic, name);
		console.log(`Message sent to ${topic}: ${name}`);
	}
</script>

<div>
	<div class="row">
		<input id="greet-input" placeholder="Enter message..." bind:value={name} />
		<button on:click={send}> Send </button>
	</div>
	<p>{greetMsg}</p>
</div>
