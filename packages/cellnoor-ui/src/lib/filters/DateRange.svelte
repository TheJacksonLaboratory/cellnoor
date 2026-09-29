<script lang="ts">
	import { getFormSubmissionFn } from './context';

	let { label, value = $bindable() }: { label: string; value: string } = $props();

	const submitForm = getFormSubmissionFn();
</script>

<fieldset>
	<legend>{label}</legend>
	<label>
		From
		<input type="date" bind:value onchange={submitForm} />
	</label>
	<label>
		To
		<input type="date" bind:value onchange={submitForm} />
	</label>
</fieldset>

<style>
	fieldset {
		display: grid;
		gap: var(--sm-gap);
	}

	label {
		display: grid;
	}
</style>
