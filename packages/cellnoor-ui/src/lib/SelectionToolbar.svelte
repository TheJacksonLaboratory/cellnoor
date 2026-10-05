<script lang="ts" generics="T">
	import type { SelectableData } from './selectable-data.svelte';

	let { selection }: { selection: SelectableData<T> } = $props();
</script>

<div class="cluster">
	<button class="button" onclick={() => selection.toggleAll()}>
		{selection.allSelected ? 'Deselect all' : 'Select all'}
	</button>
	{#if selection.nSelected}
		<span>{selection.nSelected} selected</span>
	{/if}
</div>
