<script lang="ts" generics="T">
	import { page } from '$app/state';
	import { getFormSubmissionFn } from './context';

	let {
		from,
		to,
		label
	}: { fields: T; from: keyof T & string; to: keyof T & string; label: string } = $props();

	const submitForm = getFormSubmissionFn();
</script>

<fieldset>
	<legend class="label">{label}</legend>
	<input
		type="date"
		class="input"
		name={from}
		aria-label="From"
		value={page.url.searchParams.get(from)}
		onchange={submitForm}
	/>
	to
	<input
		type="date"
		class="input"
		name={to}
		aria-label="To"
		value={page.url.searchParams.get(to)}
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
