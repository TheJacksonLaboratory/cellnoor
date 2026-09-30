<script lang="ts">
	import type { Selection } from './selection.svelte.ts';

	let { selection, ids }: { selection: Selection; ids: string[] } = $props();
</script>

<div class="cluster">
	<button class="button" onclick={() => selection.toggleAll(ids)}>
		{selection.allChecked(ids) ? 'Deselect all' : 'Select all'}
	</button>
	{#if selection.count}
		<span>{selection.count} selected</span>
	{/if}
</div>
