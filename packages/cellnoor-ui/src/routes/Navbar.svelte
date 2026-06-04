<script lang="ts">
  import { resolve } from "$app/paths";
  import "../app.css";
  import { authClient } from "$lib/auth-client";
  import { goto } from "$app/navigation";
  import type { ChromiumDatasetFilter, ChromiumDatasetOrderBy } from "cellnoor-client";
  import {
    emptyAssayFilter,
    emptySpecimenFilter,
    type Query,
    toQueryString,
  } from "$lib/query-utils.svelte";

  const { userName }: { userName: string } = $props();
  const links = [[resolve("/chromium-datasets"), "Chromium Datasets"]];

  const query: Query<ChromiumDatasetFilter, ChromiumDatasetOrderBy> = $state({
    filter: {
      ids: [],
      names: [],
      project_ids: [],
      assay: emptyAssayFilter,
      specimen: {
        ...emptySpecimenFilter,
        names: [""],
      },
    },
    limit: 50,
  });
  let stringifiedQuery = $derived(toQueryString(query));
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
      <form action="/chromium-datasets?/search" class="grow">
        <input
          bind:value={query.filter.specimen.names[0]}
          onkeydown={() => console.log(toQueryString(query))}
          aria-label="search for Chromium datasets by specimen name"
          type="search"
          placeholder="Search Chromium datasets by specimen name"
        />
        <input hidden name="q" bind:value={stringifiedQuery} />
        <button hidden>Search</button>
      </form>
    </label>
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
                  await goto(resolve("/auth/sign-in"), {
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
