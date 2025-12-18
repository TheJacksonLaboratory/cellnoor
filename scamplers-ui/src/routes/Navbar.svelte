<script lang="ts">
  import { resolve } from "$app/paths";
  import "../app.css";
  import { authClient } from "$lib/auth-client";
  import { afterNavigate } from "$app/navigation";

  let { userName }: { userName: string } = $props();
  const links = [[resolve("/chromium-datasets"), "Chromium Datasets"]];

  let searchBar: HTMLInputElement | undefined = $state();
  afterNavigate(() => {
    if (searchBar) {
      searchBar.value = "";
    }
  });
</script>

<nav class="navbar bg-base-200 sticky top-0 z-50 mb-4 justify-between">
  <div>
    <a
      class="flex flex-row shrink items-center text-2xl font-comfortaa font-bold"
      href={resolve("/")}
    >
      <img
        class="h-12 w-25 object-cover"
        src="/jax-logo.png"
        alt="The Jackson Laboratory Logo"
      />
      cellnoor
    </a>
  </div>
  <div class="flex flex-row items-center">
    <form action={resolve("/chromium-datasets")}>
      <input
        bind:this={searchBar}
        aria-label="search for Chromium datasets"
        type="text"
        name="search"
        class="input input-neutral lg:w-xs h-8"
        placeholder="Search for Chromium datasets"
      />
    </form>
    <ul class="menu menu-horizontal">
      {#each links as [link, buttonText]}
        <li>
          <a class="font-semibold" href={link}>
            {buttonText}
          </a>
        </li>
      {/each}
      <li>
        <button
          class="text-primary font-semibold"
          popovertarget="name-popover"
          style="anchor-name: --name-anchor"
        >
          {userName}
        </button>
      </li>
    </ul>
    <ul
      class="dropdown menu shadow p-2 bg-base-100 rounded-field"
      popover
      id="name-popover"
      style="position-anchor: --name-anchor; min-width: anchor-size(width)"
    >
      <li>
        <a href={resolve("/profile")}>Profile</a>
      </li>
      <li>
        <button
          onclick={async () => {
            await authClient.signOut();
          }}
        >
          Sign Out
        </button>
      </li>
    </ul>
  </div>
</nav>
