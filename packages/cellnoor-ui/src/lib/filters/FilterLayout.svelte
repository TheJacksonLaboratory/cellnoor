<script lang="ts">
	import { type Snippet } from 'svelte';
	import { setFormContext } from './context';

	let { filters, children }: { filters: Snippet; children: Snippet } = $props();

	let form: HTMLFormElement | null = $state(null);
	$effect(() => {
		if (form) {
			setFormContext(form);
		}
	});
</script>

<div class="layout">
	<form method="get" bind:this={form} data-sveltekit-keepfocus data-sveltekit-noscroll>
		{@render filters()}
	</form>
	<div class="data">
		{@render children()}
	</div>
</div>

<style>
	.layout {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 2fr);
	}

	.data {
		padding: var(--md-gap);
	}

	form {
		box-sizing: border-box;
		position: sticky;
		top: 0;
		max-height: 100dvh;
		overflow-y: auto;
		border-inline-end: 1px solid var(--color-secondary);
		display: flex;
		flex-direction: column;
		gap: var(--md-gap);
	}

	form > :global(fieldset) {
		display: flex;
		flex-direction: column;
		gap: var(--md-gap);
	}

	form > :global(fieldset) > :global(legend) {
		font-size: larger;
	}
</style>
