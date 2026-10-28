<script lang="ts">
	import type { Snippet } from 'svelte';

	let { label, children }: { label: string; children: Snippet } = $props();
</script>

<section class="card">
	<h3 class="subheading card-header">{label}</h3>
	<div class="stack card-body">
		{@render children()}
	</div>
</section>

<style>
	.stack {
		gap: var(--space-lg);
	}
</style>
