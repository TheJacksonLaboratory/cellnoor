<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title,
		href,
		subtitle,
		subtitleHref,
		checked = $bindable(),
		onCheck,
		disabled,
		children
	}: {
		title: string;
		href: string;
		subtitle?: string;
		subtitleHref?: string;
		checked?: boolean;
		onCheck?: () => void;
		disabled?: boolean;
		children?: Snippet;
	} = $props();
</script>

<li>
	{#if checked !== undefined}
		<input type="checkbox" {disabled} aria-label="Select {title}" bind:checked onchange={onCheck} />
	{/if}
	<div class="stack">
		<div class="divided-inline">
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
