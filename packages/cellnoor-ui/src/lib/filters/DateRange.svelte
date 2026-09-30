<script lang="ts">
	import { page } from '$app/state';
	import { getFormSubmissionFn } from './context';
	import { dateRangeParams } from './spec.ts';

	let { name, label }: { name: string; label: string } = $props();

	const params = $derived(dateRangeParams(name));

	const submitForm = getFormSubmissionFn();
</script>

<fieldset>
	<legend class="label">{label}</legend>
	<input
		type="date"
		class="input"
		name={params.from}
		aria-label="From"
		value={page.url.searchParams.get(params.from)}
		onchange={submitForm}
	/>
	to
	<input
		type="date"
		class="input"
		name={params.to}
		aria-label="To"
		value={page.url.searchParams.get(params.to)}
		onchange={submitForm}
	/>
</fieldset>

<style>
	fieldset {
		display: grid;
		grid-template-columns: 1fr auto 1fr;
		align-items: center;
		gap: var(--space-sm);
		margin: 0;
		padding: 0;
		border: none;
	}

	legend {
		padding: 0;
		margin-block-end: var(--space-xs);
	}
</style>
