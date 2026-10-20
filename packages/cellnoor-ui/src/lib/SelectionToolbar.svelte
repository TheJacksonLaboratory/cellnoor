<script lang="ts" generics="T">
	import type { SelectableData } from './selectable-data.svelte.ts';

	let { selection }: { selection: SelectableData<T> } = $props();
</script>

<button class="button" onclick={() => selection.toggleAll()}>
	{selection.allSelected ? 'Deselect all' : 'Select all'}
</button>
