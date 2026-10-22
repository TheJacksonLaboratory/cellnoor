<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ResolvedPathname } from '$app/types';

	let {
		title,
		href,
		subtitle,
		subtitleHref,
		checked = $bindable(),
		children
	}: {
		title: string;
		href?: ResolvedPathname;
		subtitle?: string;
		subtitleHref?: ResolvedPathname;
		checked?: boolean;
		children?: Snippet;
	} = $props();
</script>

<li>
	{#if checked !== undefined}
		<input type="checkbox" aria-label="Select {title}" bind:checked />
	{/if}
	<div class="stack">
		<div>
			{#if href}
				<a class="heading" {href}>{title}</a>
			{:else}
				<span class="heading">{title}</span>
			{/if}
			{#if subtitle}
				<a class="subheading link-arrow" href={subtitleHref}>{subtitle}</a>
			{/if}
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

		&:first-child {
			padding-block-start: 0;
		}
	}

	.heading {
		color: var(--color-primary);
	}

	.subheading {
		margin-inline-start: var(--space-sm);
		padding-inline-start: var(--space-sm);
		border-inline-start: var(--border);
	}
</style>
