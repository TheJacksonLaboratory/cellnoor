<script lang="ts">
	const links = ['Specimens', 'Libraries', 'Chromium Datasets'];

	let { data, children } = $props();
</script>

<nav>
	<a class="brand" href="/">
		<span>cellnoor</span>
	</a>
	{#each links as link (link)}
		<a href={`/${link.replace(' ', '-').toLowerCase()}`}>{link}</a>
	{/each}

	<a href="/profile">{data.user.name}</a>
</nav>

{@render children()}

<style>
	.brand {
		font-family: 'Comfortaa Variable', system-ui, sans-serif;
		font-size: 2rem;
		margin-inline-end: auto;
	}

	nav {
		display: flex;
		align-items: center;
		gap: var(--md-gap);
		padding: var(--md-gap);
		border-bottom: 1px solid var(--jax-cyan);

		a {
			display: flex;
			align-items: center;
			text-decoration: none;
			color: black;

			&:hover {
				color: var(--jax-cyan);
			}
		}
	}
</style>
