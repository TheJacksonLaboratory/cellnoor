<script lang="ts">
	import { getFormContext } from './context';

	let { label, value = $bindable() }: { label: string; value: string } = $props();

	const form = getFormContext();
	const submit = () => form.requestSubmit();
</script>

<fieldset>
	<legend>{label}</legend>
	<label>
		From
		<input type="date" bind:value onchange={submit} />
	</label>
	<label>
		To
		<input type="date" bind:value onchange={submit} />
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
