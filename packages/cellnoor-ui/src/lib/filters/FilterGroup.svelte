<script lang="ts">
	import type { Snippet } from 'svelte';

	let { label, children }: { label: string; children: Snippet } = $props();
</script>

<section class="card">
	<h3 class="subheading">{label}</h3>
	<div class="stack">
		{@render children()}
	</div>
</section>

<style>
	section {
		overflow: clip;
	}

	h3 {
		padding: var(--space-sm) var(--space-md);
		border-block-end: var(--border);
		background: var(--color-surface);
		text-transform: uppercase;
	}

	.stack {
		gap: var(--space-lg);
		padding: var(--space-md);
	}
</style>
