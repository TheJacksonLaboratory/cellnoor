export type ParsedContents = {
	format: string;
	data: Record<string, unknown> | Record<string, unknown>[];
};

export type FileLeaf = {
	type: 'file';
	path: string;
	name: string;
	raw_file_link: string;
	parsed_contents?: ParsedContents | null;
};

export type FileNode =
	| FileLeaf
	| {
			type: 'directory';
			path: string;
			name: string;
			children: FileNode[];
	  };
