<script lang="ts">
  import "../app.css";
  let { children } = $props();
  import { authClient } from "$lib/auth-client";
  const session = authClient.useSession();
  import Navbar from "./Navbar.svelte";
</script>

<svelte:head></svelte:head>

{#if $session.data}
    <Navbar name={$session.data.user.name} />
{/if}


{@render children?.()}
