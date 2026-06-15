<script lang="ts">
  import "../app.css";
  import Navbar from "./Navbar.svelte";

  let { children, data } = $props();
</script>

<svelte:head></svelte:head>

{#if data.user}
  <Navbar userName={data.user.name} />
{/if}

{@render children?.()}
