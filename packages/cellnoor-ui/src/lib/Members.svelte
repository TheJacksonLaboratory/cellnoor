<script lang="ts">
	import { mutate, type Mutation } from './client.ts';
	import MultiSelect from './MultiSelect.svelte';

	let {
		label,
		members,
		options,
		add,
		remove
	}: {
		label: string;
		members: string[];
		options: Map<string, string>;
		add: (ids: string[]) => Mutation;
		remove: (ids: string[]) => Mutation;
	} = $props();

	let value = $derived(members);

	async function onValueChange(next: string[]) {
		const added = next.filter((id) => !members.includes(id));
		const removed = members.filter((id) => !next.includes(id));

		if (!(await mutate(added.length ? add(added) : remove(removed)))) {
			value = members;
		}
	}
</script>

<MultiSelect {label} {options} bind:value {onValueChange} />
