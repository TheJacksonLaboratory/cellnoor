<script lang="ts">
  import { resolve } from "$app/paths";
  import { DATE_FORMATTER } from "$lib/date.js";
  import { type ChromiumDatasetQuery } from "scamplers-types/ChromiumDatasetQuery";
  import { type LibraryType } from "scamplers-types/LibraryType";
  import type { SampleMultiplexing } from "scamplers-types/SampleMultiplexing.js";
  import Header from "../Header.svelte";
  import LinkButton from "../LinkButton.svelte";
  import { libraryTypeMap, multiplexingTypeMap } from "$lib/string-maps";

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
</script>

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
              {
                assay.library_types?.map((libTy) => {
                  return libraryTypeMap.get(libTy);
                }).join(", ")
              }
            </td>
            <td>
              {multiplexingTypeMap.get(assay.sample_multiplexing)}
            </td>
            <td>
              <ul class="list">
                {#each links["web-summaries"] as summaryLink}
                  <li class="list-row">
                    <a target="_blank" class="link" href={summaryLink}>{
                      summaryLink.split("/").at(-1)
                    }</a>
                  </li>
                {/each}
              </ul>
            </td>
            {#each             [[links.specimens, "Specimens"], [
              links.self_,
              "Details",
            ]] as
              [link, buttonText]
            }
              <td>
                <LinkButton
                  link={link as string}
                  buttonText={buttonText as string}
                />
              </td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
  <div class="drawer-side">
    <label
      for="filter-drawer"
      aria-label="close sidebar"
      class="drawer-overlay"
    ></label>
    <form class="menu bg-base-300 min-h-full"></form>
  </div>
</div>
