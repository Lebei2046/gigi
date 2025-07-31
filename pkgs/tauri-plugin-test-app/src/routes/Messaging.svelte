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

	onMount(() => {
		subscribeTopic(topic);
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
</script>

<div>
	<div class="row">
		<input id="greet-input" placeholder="Enter message..." bind:value={name} />
		<button on:click={async () => await sendMessage(topic, name)}> Send </button>
	</div>
	<p>{greetMsg}</p>
</div>
