<script lang="ts">
	import { onMount } from "svelte";
	const endpoint = "http://localhost:7030/address/1.1.1.1";

	onMount(async function () {
		const response = await fetch(endpoint);
		const data = await response.json();
		console.log(data);
	});
</script>

<main></main>
