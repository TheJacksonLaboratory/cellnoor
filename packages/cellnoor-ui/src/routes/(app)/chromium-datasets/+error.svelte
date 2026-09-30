<script lang="ts">
	import { page } from '$app/state';
</script>

<!-- TODO: improve this error page with some kind of expand details button -->
<div>
	{page.error?.message}
	{#if page.error?.error}
		<pre>{JSON.stringify(page.error.error, null, 2)}</pre>
	{/if}
</div>
