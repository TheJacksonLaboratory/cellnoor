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
  import SortAndLimit from "../../components/SortAndLimit.svelte";

  const { data } = $props();
  const { assays, chromiumDatasets, projects, error } = $derived(data);

  let filterForm: HTMLFormElement | undefined = $state();
  const query: Query<ChromiumDatasetFilter, ChromiumDatasetOrderBy> = $state({
    filter: {
      ids: [],
      names: [],
      project_ids: [],
      assay: emptyAssayFilter,
      specimen: emptySpecimenFilter,
    },
    limit: 10_000,
    order_by: [{ field: "delivered_at", descending: true }],
  });

  let advancedQuery = $state("");
  let advancedQueryError = $derived.by(() => {
    if (!advancedQuery) {
      return "";
    }
    try {
      JSON.parse(advancedQuery);
      return "";
    } catch (error) {
      return error;
    }
  });

  let stringifiedSimpleQuery = $derived(toQueryString(query));

  let stringifiedQuery = $derived.by(() => advancedQuery || stringifiedSimpleQuery);

  const exampleAdvancedQuery = JSON.stringify(
    {
      filter: {
        names: ["super scientific science"],
        delivered_before: "3000-12-31T00:00Z",
        delivered_after: "1999-01-01T00:00Z",
        assay: {
          names: ["Universal 3' Gene Expression"],
          chemistry_versions: ["v4 - GEM-X"],
          sample_multiplexing: [
            "singleplex",
            "on_chip_multiplexing",
            "cellplex",
            "flex_barcode",
            "hashtag",
          ],
          chromium_chips: ["GEM-X FX"],
          library_types_flat: ["gene_expression", "chromatin_accessibility"],
          library_types: [["gene_expression", "chromatin_accessibility"], ["gene_expression"]],
        },
        specimen: {
          names: ["some cool sample", "another cool sample"],
          species: ["mus_musculus", "homo_sapiens"],
          host_species: ["mus_musculus"],
          fixatives: ["formaldehyde_derivative", "dithiobis_succinimidylpropionate"],
          embedded_in: [
            "paraffin",
            "optimal_cutting_temperature_compound",
            "carboxymethyl_cellulose",
          ],
          tissues: ["kleenex"],
          fresh: false,
          received_before: "3000-12-31T00:00Z",
          received_after: "1999-01-01T00:00Z",
          returned_before: "3000-12-31T00:00Z",
          returned_after: "1999-01-01T00:00Z",
          types: ["block", "cell_pellet", "suspension", "tissue"],
          thermal_preservation_methods: ["controlled_rate_freezing", "flash_freezing"],
        },
      },
    },
    null,
    2,
  );
  const advancedQueryPlaceholder = `{
    "filter": {
      "names": []
    }
}`;

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

  const orderByValues: ChromiumDatasetOrderBy[] = [{ field: "delivered_at" }, { field: "name" }];
  const orderByChoices = orderByValues.map(({ field }) => field).map(toLowercaseChoice);
</script>

<div class="drawer drawer-open">
  <input id="filter-drawer" type="checkbox" class="drawer-toggle" />
  <div class="drawer-content mt-4 px-4 flex flex-col items-stretch">
    {#if chromiumDatasets && chromiumDatasets.length != 0}
      <div class="mb-2 flex flex-row justify-between align-middle">
        <p class="text-lg">{chromiumDatasets.length} results</p>
        <SortAndLimit
          bind:orderByField={query.order_by![0]!.field}
          bind:orderByDescending={query.order_by![0]!.descending!}
          choices={orderByChoices}
          parentForm={filterForm!}
        />
      </div>
      {#each chromiumDatasets as cd (cd.id)}
        <ChromiumDataset chromiumDataset={cd} />
      {/each}
    {:else if error}
      <p class="text-center text-error">Something went wrong</p>
    {:else}
      <p class="text-center">No matching Chromium datasets found</p>
    {/if}
  </div>
  <div class="drawer-side bg-base px-4 pt-4 border-r">
    <div class="lg:w-120 sm:w-80">
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
        <Fieldset name="Advanced">
          <div class="collapse">
            <input type="checkbox" />
            <div class="collapse-title font-semibold btn btn-success mb-2">Example</div>
            <div class="collapse-content mockup-code bg-base-200 text-base-content border">
              <pre><code>{exampleAdvancedQuery}</code></pre>
            </div>
          </div>
          <textarea
            bind:value={advancedQuery}
            class="textarea w-full"
            rows="6"
            placeholder={advancedQueryPlaceholder}
          ></textarea>
          <p>{advancedQueryError}</p>
        </Fieldset>
        <input name="q" hidden bind:value={stringifiedQuery} />
        <button type="submit" class="btn btn-primary mt-4">Apply</button>
      </form>
    </div>
  </div>
</div>
