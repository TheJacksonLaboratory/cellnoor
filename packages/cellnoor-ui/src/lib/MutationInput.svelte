<script lang="ts">
	import { mutate, type Mutation } from './client.ts';

	let {
		label,
		type = 'text',
		save
	}: { label: string; type?: 'text' | 'date'; save: (value: string) => Mutation } = $props();

	const id = $props.id();

	async function onsubmit(e: SubmitEvent & { currentTarget: HTMLFormElement }) {
		e.preventDefault();
		const form = e.currentTarget;

		if (await mutate(save(new FormData(form).get('value')?.toString() ?? ''))) {
			form.reset();
		}
	}
</script>

<form class="field" {onsubmit}>
	<label class="label" for={id}>{label}</label>
	<div class="cluster">
		<input {id} class="input" {type} name="value" required />
		<button class="button">Save</button>
	</div>
</form>
