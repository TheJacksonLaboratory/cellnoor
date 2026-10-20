import type {
	ChromiumDatasetDetailed,
	ChromiumDatasetParsedFileData,
	ChromiumDatasetParsedFileDataTree
} from 'cellnoor-client/cellnoor-types.ts';
import Papa from 'papaparse';

type MetricsRow = { dataset_name: string } & Record<string, unknown>;

export function convertMetricsToFiles(datasets: Iterable<ChromiumDatasetDetailed>) {
	const metrics: Record<ChromiumDatasetParsedFileData['format'], MetricsRow[]> = {
		single_row_csv: [],
		multi_row_csv: [],
		json: []
	};

	for (const ds of datasets) {
		for (const file of ds.files) {
			for (const { format, data } of filesInDirectory(file)) {
				const rows = Array.isArray(data) ? data : [data];
				metrics[format].push(...rows.map((row) => ({ dataset_name: ds.name, ...row })));
			}
		}
	}

	return {
		single_row_csv: toCsv(metrics.single_row_csv),
		multi_row_csv: toCsv(metrics.multi_row_csv),
		json:
			metrics.json.length !== 0
				? new Blob([JSON.stringify(metrics.json)], { type: 'application/json' })
				: null
	};
}

function toCsv(rows: MetricsRow[]) {
	if (rows.length === 0) {
		return null;
	}

	// Papa takes the header from the first row's keys unless the columns are given explicitly
	const columns = [...new Set(rows.flatMap(Object.keys))];

	return new Blob([Papa.unparse(rows, { columns })], { type: 'text/csv' });
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
