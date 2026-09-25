<script lang="ts">
	import type { Snippet } from 'svelte';

	let { filters, children }: { filters: Snippet; children: Snippet } = $props();
</script>

<div>
	<form method="get">
		{@render filters()}
		<button>Apply</button>
	</form>
	{@render children()}
</div>

<style>
	div {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: var(--md-gap);
	}
</style>
