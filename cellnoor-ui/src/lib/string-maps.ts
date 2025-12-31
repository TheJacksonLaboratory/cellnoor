import type { LibraryType } from "cellnoor-types/LibraryType";
import type { SampleMultiplexing } from "cellnoor-types/SampleMultiplexing";

export const libraryTypeMap: Map<LibraryType, string> = new Map([
  ["antibody_capture", "Antibody Capture"],
  ["antigen_capture", "Antigen Capture"],
  ["chromatin_accessibility", "Chromatin Accessibility"],
  ["crispr_guide_capture", "CRISPR Guide Capture"],
  ["custom", "Custom"],
  ["gene_expression", "Gene Expression"],
  ["vdj", "VDJ"],
  ["vdj_b", "VDJ-B"],
  ["vdj_t", "VDJ-T"],
  ["vdj_t_gd", "VDJ-T-GD"],
]);

export const multiplexingTypeMap: Map<SampleMultiplexing | undefined, string> =
  new Map(
    [
      ["cellplex", "CellPlex"],
      ["flex_barcode", "Flex Barcode"],
      ["hashtag", "Hashtagging"],
      ["on_chip_multiplexing", "On Chip Multiplexing"],
      ["singleplex", ""],
      [undefined, ""],
    ],
  );
