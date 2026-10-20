<script lang="ts">
	import type { Snippet } from 'svelte';

	let { sidebar, children }: { sidebar: Snippet; children: Snippet } = $props();
</script>

<div class="sidebar-layout">
	<aside class="stack">
		{@render sidebar()}
	</aside>

	<section class="stack">
		{@render children()}
	</section>
</div>

<style>
	.sidebar-layout {
		display: grid;
		grid-template-columns: minmax(min-content, 32rem) minmax(0, 1fr);
		align-items: start;

		> * {
			padding: var(--space-md);
		}
	}

	aside {
		position: sticky;
		inset-block-start: 0;
		max-block-size: 100dvh;
		overflow-y: auto;
		overscroll-behavior: contain;
	}
</style>
