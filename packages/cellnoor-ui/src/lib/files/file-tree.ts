export type ParsedContents = { data: Record<string, unknown> | Record<string, unknown>[] };

export type FileLeaf = {
	type: 'file';
	name: string;
	raw_file_link: string;
	parsed_contents?: ParsedContents | null;
};

export type FileNode = FileLeaf | { type: 'directory'; name: string; children: FileNode[] };

export type ValuedFileNode = (
	| FileLeaf
	| { type: 'directory'; name: string; children: ValuedFileNode[] }
) & { value: string };

export function withValues(nodes: FileNode[], parent = ''): ValuedFileNode[] {
	return nodes.map((node) => {
		const value = `${parent}/${node.name}`;

		return node.type === 'file'
			? { ...node, value }
			: { ...node, value, children: withValues(node.children, value) };
	});
}
