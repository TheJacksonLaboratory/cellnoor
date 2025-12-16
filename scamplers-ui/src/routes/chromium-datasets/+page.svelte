<script lang="ts">
  import { resolve } from "$app/paths";
  import { DATE_FORMATTER } from "$lib/date.js";
  import { type ChromiumDatasetQuery } from "scamplers-types/ChromiumDatasetQuery";
  import { type LibraryType } from "scamplers-types/LibraryType";
  import type { SampleMultiplexing } from "scamplers-types/SampleMultiplexing.js";
  import Header from "../Header.svelte";
  import LinkButton from "../LinkButton.svelte";
  import { libraryTypeMap, multiplexingTypeMap } from "$lib/string-maps";
  import LibraryTypeBadges from "../LibraryTypeBadges.svelte";

  const { data } = $props();
  const { chromiumDatasets } = $derived(data);

  let query: ChromiumDatasetQuery = $state({
    filter: {
      ids: [],
      specimen: {
        ids: [],
        names: [],
        submitted_by: [],
        labs: [],
        species: [],
        host_species: [],
        types: [],
        embedded_in: [],
        fixatives: [],
        tissues: [],
        returned_by: [],
        additional_data: {},
      },
      assay: {
        ids: [],
        names: [],
        library_types: [],
        sample_multiplexing: [],
        chemistry_versions: [],
        chromium_chips: [],
      },
      lab_ids: [],
    },
    order_by: [],
  });

  let currentSpecimenName = $state("");
</script>

<!-- TODO: factor this layout into a common table component -->
<div class="drawer lg:drawer-open">
  <input id="filter-drawer" type="checkbox" class="drawer-toggle" />
  <div class="drawer-content">
    <Header header="Chromium Datasets" />
    <label for="filter-drawer" class="btn mx-2 drawer-button lg:hidden">
      Filter and sort
    </label>
    <table class="table">
      <thead>
        <tr>
          {#each             [
              "Name",
              "Delivered on",
              "Assay",
              "Library types",
              "Sample multiplexing",
              "Web summaries",
              "Metrics files",
              "",
              "",
              "",
            ] as
            column
          }
            <td>{column}</td>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each chromiumDatasets as { name, delivered_at, assay, links }}
          <tr>
            <td>
              {name}
            </td>
            <td>
              {DATE_FORMATTER.format(new Date(delivered_at))}
            </td>
            <td>
              {assay.name}
            </td>
            <td>
              <LibraryTypeBadges libraryTypes={assay.library_types} />
            </td>
            <td>
              {multiplexingTypeMap.get(assay.sample_multiplexing)}
            </td>
            <td>
              <ul>
                {#each links["web-summaries"] as summaryLink}
                  <li>
                    <a target="_blank" class="link" href={summaryLink}>{
                      summaryLink.split("/").at(-1)
                    }</a>
                  </li>
                {/each}
              </ul>
            </td>
            <td>
              <ul>
                {#each links["metrics-files"] as summaryLink}
                  <li>
                    <a target="_blank" class="link" href={summaryLink}>{
                      summaryLink.split("/").at(-1)
                    }</a>
                  </li>
                {/each}
              </ul>
            </td>
            <td>
              <div class="flex flex-row flex-wrap gap-1 place-content-between">
                {#each                 [[links.specimens, "Specimens"], [
                  links.self_,
                  "Details",
                ]] as
                  [link, buttonText]
                }
                  <LinkButton
                    link={link as string}
                    buttonText={buttonText as string}
                  />
                {/each}
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
  <div class="drawer-side bg-base">
    <label
      for="filter-drawer"
      aria-label="close sidebar"
      class="drawer-overlay"
    ></label>
    <div class="flex flex-col items-center">
      <form
        onsubmit={() => {
          if (currentSpecimenName) {
            query.filter?.specimen?.names?.push(currentSpecimenName);
          }
        }}
      >
        <input class="input" type="text" bind:value={currentSpecimenName} />
      </form>
      <ul>
        {#each query.filter?.specimen?.names as specimenName}
          <li>
            {specimenName}
          </li>
        {/each}
      </ul>
    </div>
  </div>
</div>
