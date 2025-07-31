<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		subscribeTopic,
		unsubscribeTopic,
		sendMessage,
		onMessageReceived
	} from 'tauri-plugin-libp2p-messaging-api';

	let topic = 'test';
	let name = '';
	let greetMsg = '';
	let handle: any = undefined;

	onMount(async () => {
		console.log(`Subscribing to topic: ${topic}`);
		await subscribeTopic(topic);
		console.log(`Subscribed to topic: ${topic}`);
		handle = onMessageReceived((message) => {
			console.log(message);
			greetMsg = JSON.stringify(message, null, 2);
		});
	});

	onDestroy(() => {
		// unsubscribeTopic(topic);
		if (handle) {
			handle();
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
