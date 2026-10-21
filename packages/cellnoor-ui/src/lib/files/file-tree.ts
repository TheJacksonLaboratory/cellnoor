export type ParsedContents = {
	format: string;
	data: Record<string, unknown> | Record<string, unknown>[];
};

type FileNodeBase = { path: string; name: string };

export type FileLeaf = FileNodeBase & {
	type: 'file';
	raw_file_link: string;
	parsed_contents?: ParsedContents | null;
};

export type FileNode =
	| FileLeaf
	| (FileNodeBase & {
			type: 'directory';
			path: string;
			name: string;
			children: FileNode[];
	  });
