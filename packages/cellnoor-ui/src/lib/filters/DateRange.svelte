<script lang="ts">
	import { page } from '$app/state';

	let { label, name }: { label: string; name: string } = $props();
</script>

<fieldset>
	<legend>{label}</legend>
	<label>
		From
		<input type="date" name="{name}_from" value={page.url.searchParams.get(`${name}_from`) ?? ''} />
	</label>
	<label>
		To
		<input type="date" name="{name}_to" value={page.url.searchParams.get(`${name}_to`) ?? ''} />
	</label>
</fieldset>
