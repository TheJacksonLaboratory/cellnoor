<script lang="ts" generics="T">
	import { page } from '$app/state';
	import { getFormSubmissionFn } from './context';
	import SelectedValues from './SelectedValues.svelte';

	let { name, label }: { fields: T; name: keyof T & string; label: string } = $props();

	const id = $props.id();

	let values = $derived(page.url.searchParams.getAll(name));
	let query = $state('');

	const submitForm = getFormSubmissionFn();

	function add(e: KeyboardEvent) {
		if (e.key !== 'Enter') return;
		e.preventDefault();

		const added = query.trim();
		query = '';
		if (!added || values.includes(added)) return;

		values = [...values, added];
		submitForm();
	}
</script>

<div class="field">
	<label class="label" for={id}>{label}</label>
	<SelectedValues bind:values />
	{#each values as v (v)}
		<input type="hidden" {name} value={v} />
	{/each}
	<input {id} class="input" bind:value={query} onkeydown={add} />
</div>
