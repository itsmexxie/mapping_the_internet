<script lang="ts">
	import { page } from "$app/state";
</script>

<div class="text-center my-5">
	<h1>:(</h1>
	{#if page.status == 404}
	<p class="text-body-secondary">Stránka nebyla nalezena...</p>
	{/if}
</div>
