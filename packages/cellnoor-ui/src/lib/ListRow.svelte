<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title,
		href,
		subtitle,
		subtitleHref,
		checked = $bindable(),
		children
	}: {
		title: string;
		href: string;
		subtitle?: string;
		subtitleHref?: string;
		checked?: boolean;
		children?: Snippet;
	} = $props();
</script>

<li>
	<input type="checkbox" aria-label="Select {title}" bind:checked />
	<div class="stack">
		<div>
			<a class="heading" {href}>{title}</a>
			<a class="subheading link-arrow" href={subtitleHref}>{subtitle}</a>
		</div>
		{@render children?.()}
	</div>
</li>

<style>
	li {
		display: grid;
		grid-template-columns: auto 1fr;
		align-items: baseline;
		gap: var(--space-md);
		padding-block: var(--space-lg);
		overflow-wrap: anywhere;
	}

	.heading {
		color: var(--color-primary);
	}
</style>
