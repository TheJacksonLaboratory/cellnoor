import type { ChromiumDatasetParsedFileDataFileTree } from 'cellnoor-client/cellnoor-types.ts';

export type FileNode = ChromiumDatasetParsedFileDataFileTree;

export type FileLeaf = Extract<FileNode, { type: 'file' }>;
