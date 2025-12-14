<script lang="ts">
  import { resolve } from "$app/paths";
  import { DATE_FORMATTER } from "$lib/date.js";
  import { type ChromiumDatasetQuery } from "scamplers-types/ChromiumDatasetQuery";
  import { type LibraryType } from "scamplers-types/LibraryType";

  const { data } = $props();

  const assayNames = new Set(data.assays.map((a) => {
    return a.name;
  }));
  const libraryTypes: LibraryType[] = [
    "antibody_capture",
    "antigen_capture",
    "chromatin_accessibility",
    "crispr_guide_capture",
    "custom",
    "gene_expression",
    "vdj",
    "vdj_b",
    "vdj_t",
    "vdj_t_gd",
  ];
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
    <div class="text-center">
      <h1 class="text-4xl font-bold my-4">Chromium Datasets</h1>
    </div>
    <label for="filter-drawer" class="btn mx-2 drawer-button lg:hidden">
      Filter and sort
    </label>
    <table class="table">
      <thead>
        <tr>
          <td>
            Name
          </td>
          <td>
            Delivered on
          </td>
          <td>Assay</td>
          <td></td>
        </tr>
      </thead>
      <tbody>
        {#each data.chromiumDatasets as ds}
          <tr>
            <td>
              {ds.name}
            </td>
            <td>
              {DATE_FORMATTER.format(new Date(ds.delivered_at))}
            </td>
            <td>my assay</td>
            <td>
              <button class="btn btn-outline btn-primary">
                <a href={resolve(`/chromium-datasets/${ds.id}`)}>Details</a>
              </button>
            </td>
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
    <form class="menu bg-base-300 min-h-full">
      <fieldset class="fieldset">
        <legend class="text-lg fieldset-legend">Assay Name</legend>
        {#each assayNames as assay}
          <input
            onclick={() => {query.filter?.assay?.names?.push(assay)}}
            class="btn"
            type="checkbox"
            name="assay-name"
            aria-label={assay}
          />
        {/each}
      </fieldset>
    </form>
  </div>
</div>
