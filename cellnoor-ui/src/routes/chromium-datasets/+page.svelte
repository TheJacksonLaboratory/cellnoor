<script lang="ts">
  import {
    emptyAssayFilter,
    emptySpecimenFilter,
    type Query,
    toQueryString,
  } from "$lib/query-utils.svelte";
  import InputList from "../../components/InputList.svelte";
  import type {
    ChromiumDatasetFilter,
    ChromiumDatasetOrderBy,
    LibraryType,
    SampleMultiplexing,
    Species,
  } from "cellnoor-client";
  import ChromiumDataset from "./ChromiumDataset.svelte";
  import Fieldset from "../../components/Fieldset.svelte";

  const { data } = $props();
  const { assays, chromiumDatasets, projects, error } = $derived(data);

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

  function toLowercaseChoice(s: string) {
    return { label: s.replaceAll("_", " "), value: s };
  }

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
  const speciesChoices = species.map(toLowercaseChoice);

  const plexy: SampleMultiplexing[] = [
    "cellplex",
    "flex_barcode",
    "hashtag",
    "on_chip_multiplexing",
    "singleplex",
  ];
  const plexyChoices = plexy.map(toLowercaseChoice);

  const libraryTypes: LibraryType[] = [
    "antibody_capture",
    "antigen_capture",
    "chromatin_accessibility",
    "crispr_guide_capture",
    "custom",
    "gene_expression",
    "multiplexing_capture",
    "vdj",
  ];
  const libraryTypeChoices = libraryTypes.map(toLowercaseChoice);
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
      <p class="font-bold text-lg px-2">Filter</p>
      <form bind:this={filterForm} action="?/search">
        <Fieldset name="Specimen Information">
          <InputList
            parentForm={filterForm}
            fieldName="Specimen Name"
            bind:values={query.filter.specimen.names}
          />
          <InputList
            parentForm={filterForm}
            fieldName="Species"
            choices={speciesChoices}
            bind:values={query.filter.specimen.species}
          />
        </Fieldset>
        <Fieldset name="Lab Information">
          <InputList
            parentForm={filterForm}
            fieldName="Lab Name"
            choices={(projects || []).map((p) => {
              return { label: p.name, value: p.id };
            })}
            bind:values={query.filter.project_ids}
          />
        </Fieldset>
        <Fieldset name="Assay Information">
          <InputList
            parentForm={filterForm}
            fieldName="Assay Name"
            choices={assays!.map((a) => {
              return { label: a.name, value: a.name };
            })}
            bind:values={query.filter.assay.names}
          />
          <InputList
            parentForm={filterForm}
            fieldName="Multiplexing"
            choices={plexyChoices}
            bind:values={query.filter.assay.sample_multiplexing}
          />
          <InputList
            parentForm={filterForm}
            fieldName="Library Types"
            choices={libraryTypeChoices}
            bind:values={query.filter.assay.library_types_flat}
          />
        </Fieldset>
        <input name="q" hidden bind:value={stringifiedQuery} />
        <button class="btn btn-primary mt-4">Apply</button>
      </form>
    </div>
  </div>
</div>
