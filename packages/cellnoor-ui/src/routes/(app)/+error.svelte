<script lang="ts">
	import { page } from '$app/state';
</script>

<div>
	{page.error?.message}
	{#if page.error?.error}
		<pre><code>{JSON.stringify(page.error.error, null, 2)}</code></pre>
	{/if}
</div>
