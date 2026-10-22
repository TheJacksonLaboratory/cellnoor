<script lang="ts">
	import { mutate, type Mutation } from './client.ts';

	let {
		label,
		checked,
		save
	}: { label: string; checked: boolean; save: (checked: boolean) => Mutation } = $props();

	async function onchange(e: Event & { currentTarget: HTMLInputElement }) {
		const input = e.currentTarget;

		if (!(await mutate(save(input.checked)))) {
			input.checked = checked;
		}
	}
</script>

<label>
	<input type="checkbox" {checked} {onchange} />
	{label}
</label>
