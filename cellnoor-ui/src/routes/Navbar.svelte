<script lang="ts">
  import { resolve } from "$app/paths";
  import "../app.css";
  import { authClient } from "$lib/auth-client";
  import { afterNavigate, goto } from "$app/navigation";
  import type { ChromiumDatasetQuery } from "cellnoor-types/ChromiumDatasetQuery";
  import qs from "qs";

  let { userName }: { userName: string } = $props();
  const links = [[resolve("/chromium-datasets"), "Chromium Datasets"]];

  let searchBarValue = $state("");
  let query: ChromiumDatasetQuery = $derived({
    filter: { specimen: { names: [`%${searchBarValue}%`] } },
  });
  afterNavigate(() => searchBarValue = searchBarValue.replaceAll("%", ""));
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
    <label class="input lg:w-lg h-8">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 16 16"
        fill="currentColor"
        class="size-4"
      >
        <path
          fill-rule="evenodd"
          d="M9.965 11.026a5 5 0 1 1 1.06-1.06l2.755 2.754a.75.75 0 1 1-1.06 1.06l-2.755-2.754ZM10.5 7a3.5 3.5 0 1 1-7 0 3.5 3.5 0 0 1 7 0Z"
          clip-rule="evenodd"
        />
      </svg>
      <form
        class="grow"
        onsubmit={(event) => {
          event.preventDefault();
          goto(
            `/chromium-datasets?${
              qs.stringify(query, { encodeValuesOnly: true })
            }`,
          );
        }}
      >
        <input
          bind:value={searchBarValue}
          aria-label="search for Chromium datasets by specimen name"
          type="search"
          placeholder="Search Chromium datasets by specimen name"
        />
      </form>
    </label>
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
