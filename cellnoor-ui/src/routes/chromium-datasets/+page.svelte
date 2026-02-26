<script lang="ts">
  import {
    emptyAssayFilter,
    emptySpecimenFilter,
    type Query,
    toQueryString,
  } from "$lib/query-utils.svelte";
  import InputList from "../InputList.svelte";
  import type {
    ChromiumDatasetFilter,
    ChromiumDatasetOrderBy,
    Species,
  } from "cellnoor-client";
  import ChromiumDataset from "./ChromiumDataset.svelte";
  import InputListWithChoices from "../InputListWithChoices.svelte";

  const { data } = $props();
  const { chromiumDatasets, projects, error } = $derived(data);

  let filterForm: HTMLFormElement | undefined = $state();
  const query: Query<ChromiumDatasetFilter, ChromiumDatasetOrderBy> =
    $state({
      filter: {
        ids: [],
        names: [],
        project_ids: [],
        assay: emptyAssayFilter,
        specimen: emptySpecimenFilter,
      },
      limit: 50,
    });
  let stringifiedQuery = $derived(toQueryString(query));

  const species: Species[] = [
    "ambystoma_mexicanum",
    "callithrix_jacchus",
    "canis_familiaris",
    "drosophila_melanogaster",
    "gasterosteus_aculeatus",
    "homo_sapiens",
    "mus_musculus",
    "rattus_norvegicus",
    "sminthopsis_crassicaudata",
  ];
  const speciesOptions = species.map((s) => {
    return { optValue: s, displayText: s.replaceAll("_", " ") };
  });
</script>

<div class="drawer lg:drawer-open">
  <input id="filter-drawer" type="checkbox" class="drawer-toggle" />
  <div class="drawer-content mt-4 px-4 flex flex-col items-stretch">
    <label for="filter-drawer" class="btn mx-2 drawer-button lg:hidden">
      Filter and sort
    </label>
    {#if chromiumDatasets && chromiumDatasets.length != 0}
      <p class="mb-2 text-lg">{chromiumDatasets.length} results</p>
      {#each chromiumDatasets as cd}
        <ChromiumDataset chromiumDataset={cd} />
      {/each}
    {:else if error}
      <p class="text-center text-error">Something went wrong</p>
    {:else}
      <p class="text-center">No matching Chromium datasets found</p>
    {/if}
  </div>
  <div class="drawer-side bg-base px-4 pt-4 border-r">
    <div class="w-80">
      <label
        for="filter-drawer"
        aria-label="close sidebar"
        class="drawer-overlay"
      ></label>
      <form bind:this={filterForm} action="?/search">
        <fieldset class="fieldset bg-base-200 border rounded-box p-2">
          <legend class="fieldset-legend font-bold text-lg">
            Specimen Information
          </legend>
          <InputList
            parentForm={filterForm}
            fieldName="Specimen Name"
            bind:targetArray={query.filter.specimen.names}
          />
          <InputListWithChoices
            parentForm={filterForm}
            fieldName="Species"
            options={speciesOptions}
            bind:targetArray={query.filter.specimen.species}
          />
        </fieldset>
        <fieldset class="fieldset bg-base-200 border rounded-box p-2">
          <legend class="fieldset-legend font-bold text-lg">
            Lab Information
          </legend>
          <InputListWithChoices
            parentForm={filterForm}
            fieldName="Lab Name"
            options={(projects ?? []).map((p) => {
              return { optValue: p.id, displayText: p.name };
            })}
            bind:targetArray={query.filter.project_ids}
          />
        </fieldset>
        <input name="q" hidden bind:value={stringifiedQuery} />
        <button class="btn btn-primary mt-4">Apply</button>
      </form>
    </div>
  </div>
</div>
