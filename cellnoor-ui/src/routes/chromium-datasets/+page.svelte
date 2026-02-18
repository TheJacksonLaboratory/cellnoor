<script lang="ts">
  import { DATE_FORMATTER } from "$lib/date.js";
  import LibraryTypeBadges from "../LibraryTypeBadges.svelte";
  import type {
    ChromiumDatasetFilter,
    ChromiumDatasetOrderBy,
  } from "cellnoor-client";

  const { data } = $props();
  const { chromiumDatasets } = $derived(data);

  let query: {
    filter?: ChromiumDatasetFilter;
    limit?: number;
    offset?: number;
    order_by?: ChromiumDatasetOrderBy[];
  } = $state({});
</script>

<div class="drawer lg:drawer-open">
  <input id="filter-drawer" type="checkbox" class="drawer-toggle" />
  <div class="drawer-content flex flex-col items-stretch">
    <label for="filter-drawer" class="btn mx-2 drawer-button lg:hidden">
      Filter and sort
    </label>
    {#if chromiumDatasets}
      {#each chromiumDatasets as { name, delivered_at, assay, links, files }}
        <div
          class="flex flex-row p-4 mx-8 my-2 border border-neutral rounded-box place-content-between"
        >
          <div class="flex flex-col gap-1">
            <a
              class="link link-primary link-hover text-xl font-semibold max-w-fit"
              href={links.self as string}
            >{name}</a>
            <p>
              {assay.name}
              <span class="font-extralight">({assay.chemistry_version})</span>
            </p>
            {#if assay.library_types}
              <LibraryTypeBadges
                libraryTypes={assay.library_types}
              />
            {/if}
            <span class="text-sm mt-2">Delivered on {
                DATE_FORMATTER.format(
                  new Date(delivered_at),
                )
              }</span>
          </div>
          <div class="menu flex-row flex-wrap gap-2">
            {#each files.entries() as [directory, links]}
              <li>
                <details class="dropdown">
                  <summary>
                    <div
                      class="flex flex-row gap-1 items-center"
                    >
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="size-6"
                      >
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          d="M2.25 12.75V12A2.25 2.25 0 0 1 4.5 9.75h15A2.25 2.25 0 0 1 21.75 12v.75m-8.69-6.44-2.12-2.12a1.5 1.5 0 0 0-1.061-.44H4.5A2.25 2.25 0 0 0 2.25 6v12a2.25 2.25 0 0 0 2.25 2.25h15A2.25 2.25 0 0 0 21.75 18V9a2.25 2.25 0 0 0-2.25-2.25h-5.379a1.5 1.5 0 0 1-1.06-.44Z"
                        />
                      </svg>
                      <span>{directory}</span>
                    </div>
                  </summary>
                  <ul>
                    {#each links as link}
                      <li>
                        <a target="_blank" href={link}>
                          <svg
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke-width="1.5"
                            stroke="currentColor"
                            class="size-6"
                          >
                            <path
                              stroke-linecap="round"
                              stroke-linejoin="round"
                              d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z"
                            />
                          </svg>
                          {link.split("/").at(-1)}
                        </a>
                      </li>
                    {/each}
                  </ul>
                </details>
              </li>
            {/each}
          </div>
        </div>
      {/each}
    {:else}
      <p class="text-center text-error">Something went wrong</p>
    {/if}
  </div>
  <div class="drawer-side bg-base">
    <label
      for="filter-drawer"
      aria-label="close sidebar"
      class="drawer-overlay"
    ></label>
  </div>
</div>
