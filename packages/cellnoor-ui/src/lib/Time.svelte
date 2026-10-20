<script lang="ts">
	import { DATE_FORMATTER } from './format.ts';

	let { datetime }: { datetime: string } = $props();
</script>

<time {datetime}>{DATE_FORMATTER.format(new Date(datetime))}</time>
