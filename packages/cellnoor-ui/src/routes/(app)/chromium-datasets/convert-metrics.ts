import type {
	ChromiumDatasetDetailed,
	ChromiumDatasetParsedFileData,
	ChromiumDatasetParsedFileDataFileTree
} from 'cellnoor-client/cellnoor-types.ts';
import Papa from 'papaparse';

type MetricsRow = { dataset_name: string } & Record<string, unknown>;

export function convertMetricsToFiles(datasets: Iterable<ChromiumDatasetDetailed>) {
	const metrics: {
		standardMetrics: MetricsRow[];
		cellrangerMultiMetrics: MetricsRow[];
		json: MetricsRow[];
	} = {
		standardMetrics: [],
		cellrangerMultiMetrics: [],
		json: []
	};

	for (const ds of datasets) {
		for (const file of ds.files) {
			for (const { isCellrangerMulti, format, data } of filesInDirectory(file)) {
				const rows = Array.isArray(data) ? data : [data];
				const transformed = rows.map((row) => ({ dataset_name: ds.name, ...row }));

				if (isCellrangerMulti) {
					metrics.cellrangerMultiMetrics.push(...transformed);
				} else if (format === 'json') {
					metrics.json.push(...transformed);
				} else {
					metrics.standardMetrics.push(...transformed);
				}
			}
		}
	}

	return {
		standardMetrics: toCsv(metrics.standardMetrics),
		cellrangerMultiMetrics: toCsv(metrics.cellrangerMultiMetrics),
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
	file: ChromiumDatasetParsedFileDataFileTree
): Generator<{ isCellrangerMulti: boolean } & ChromiumDatasetParsedFileData> {
	if (file.type === 'file' && file.parsed_contents) {
		if (file.parsed_contents.format === 'multi_row_csv') {
			yield {
				isCellrangerMulti: isCellrangerMultiCsv(file.parsed_contents.data),
				...file.parsed_contents
			};
		}

		yield { isCellrangerMulti: false, ...file.parsed_contents };
	} else if (file.type === 'directory') {
		for (const f of file.children) {
			yield* filesInDirectory(f);
		}
	}
}

function isCellrangerMultiCsv(data: Record<string, unknown>[]) {
	// We know that "Category" is always a column in the metrics summaries of multiplexed datasets, so just check for its existence. Even if the
	if (data[0].Category) {
		return true;
	}

	return false;
}
