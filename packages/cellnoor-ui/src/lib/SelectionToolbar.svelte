<script lang="ts">
	import type { SvelteSet } from 'svelte/reactivity';

	let { selected, ids }: { selected: SvelteSet<string>; ids: string[] } = $props();

	const allSelected = $derived(ids.length > 0 && ids.every((id) => selected.has(id)));

	function toggleAll() {
		if (allSelected) {
			selected.clear();
		} else {
			for (const id of ids) {
				selected.add(id);
			}
		}
	}
</script>

<button class="button" onclick={toggleAll}>
	{allSelected ? 'Deselect all' : 'Select all'}
</button>
