<script lang="ts">
  import { resolve } from "$app/paths";
  import "../app.css";
  import { goto } from "$app/navigation";
  import { createAuthClient } from "better-auth/svelte";
  import { PUBLIC_AUTH_URL } from "$app/env/public";

  const authClient = createAuthClient({ baseURL: PUBLIC_AUTH_URL });

  const { userName }: { userName: string } = $props();
  const links = [[resolve("/chromium-datasets"), "Chromium Datasets"]];
</script>

<nav class="navbar bg-base-200 sticky z-64 justify-between border-b">
  <div>
    <a
      class="flex flex-row shrink items-center text-2xl font-comfortaa font-bold"
      href={resolve("/")}
    >
      <img class="h-12 w-25 object-cover" src="/jax-logo.png" alt="The Jackson Laboratory Logo" />
      cellnoor
    </a>
  </div>
  <div class="flex flex-row items-center">
    <ul class="menu menu-horizontal">
      {#each links as [link, buttonText] (link)}
        <li>
          <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
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
            await authClient.signOut({
              fetchOptions: {
                onSuccess: async () =>
                  await goto(PUBLIC_AUTH_URL || "", {
                    invalidateAll: true,
                  }),
              },
            });
          }}
        >
          Sign Out
        </button>
      </li>
    </ul>
  </div>
</nav>
