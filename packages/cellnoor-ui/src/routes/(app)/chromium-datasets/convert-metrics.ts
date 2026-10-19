import type { SelectableData } from '#lib/selectable-data.svelte.ts';
import type {
	ChromiumDatasetDetailed,
	ChromiumDatasetParsedFileData,
	ChromiumDatasetParsedFileDataTree
} from 'cellnoor-client/cellnoor-types';
import Papa from 'papaparse';

export function convertMetricsToFiles(selectedItems: SelectableData<ChromiumDatasetDetailed>) {
	type MetricsRow = { dataset_name: string } & Record<string, unknown>;

	const metrics: Record<string, MetricsRow[]> = {
		single_row_csv: [],
		multi_row_csv: [],
		json: []
	} satisfies Record<ChromiumDatasetParsedFileData['format'], MetricsRow[]>;

	const singleRowDefaults: Record<string, null> = {};
	const multiRowDefaults: Record<string, null> = {};

	const updateSingleRowDefaults = (obj: Record<string, unknown>) => {
		for (const k of Object.keys(obj)) {
			singleRowDefaults[k] = null;
		}
	};

	const updateMulitRowDefaults = (obj: Record<string, unknown>) => {
		for (const k of Object.keys(obj)) {
			multiRowDefaults[k] = null;
		}
	};

	const pushMetrics = (dataset_name: string, fileData: Iterable<ChromiumDatasetParsedFileData>) => {
		for (const f of fileData) {
			if (f.format === 'single_row_csv') {
				updateSingleRowDefaults(f.data);
				metrics[f.format].push({ dataset_name, ...singleRowDefaults, ...f.data });
			} else if (f.format === 'multi_row_csv') {
				updateMulitRowDefaults(f.data[0]);
				for (const d of f.data) {
					metrics[f.format].push({ dataset_name, ...multiRowDefaults, ...d });
				}
			} else {
				metrics[f.format].push({ dataset_name, ...f.data });
			}
		}
	};

	for (const ds of selectedItems.selectedItems) {
		for (const file of ds.files) {
			pushMetrics(ds.name, filesInDirectory(file));
		}
	}

	return {
		single_row_csv:
			metrics.single_row_csv.length !== 0
				? new Blob([Papa.unparse(metrics.single_row_csv)], { type: 'text/csv' })
				: null,
		multi_row_csv:
			metrics.multi_row_csv.length !== 0
				? new Blob([Papa.unparse(metrics.multi_row_csv)], { type: 'text/csv' })
				: null,
		json:
			metrics.json.length !== 0
				? new Blob([JSON.stringify(metrics.json)], { type: 'application/json' })
				: null
	};
}

function* filesInDirectory(
	file: ChromiumDatasetParsedFileDataTree
): Generator<ChromiumDatasetParsedFileData> {
	if (file.type === 'file' && file.parsed_contents) {
		yield file.parsed_contents;
	} else if (file.type === 'directory') {
		for (const f of file.children) {
			yield* filesInDirectory(f);
		}
	}
}
