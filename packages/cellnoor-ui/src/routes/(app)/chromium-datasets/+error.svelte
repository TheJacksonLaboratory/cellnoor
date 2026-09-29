<script lang="ts">
	import type { DbError } from 'cellnoor-client/cellnoor-types';

	const { error } = $props();
	const structuredError: DbError = $derived(JSON.parse(error.message));
</script>

<!-- TODO: improve this error page with some kind of expand details button -->
<div>
	{error.message}
</div>
