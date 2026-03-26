export interface FileNode {
  name: string;
  content: any;
  type: string;
}

export interface DirectoryNode {
  name: string;
  children: (DirectoryNode | FileNode)[];
}

export function createFileTree(files: FileNode[]): (DirectoryNode | FileNode)[] {
  const fileTree: (DirectoryNode | FileNode)[] = [];
  const groups = new Map<string, DirectoryNode>();

  // First, group the files into directories
  for (const file of files) {
    const indexOfFirstSlash = file.name.indexOf("/");
    const noSlashFound = indexOfFirstSlash === -1;

    // recursive basecase
    if (noSlashFound) {
      fileTree.push(file);
      continue;
    }

    const rootName = file.name.slice(0, indexOfFirstSlash);
    const child = { ...file, name: file.name.slice(indexOfFirstSlash + 1) };

    let dir = groups.get(rootName);
    if (!dir) {
      dir = { name: rootName, children: [] };
      groups.set(rootName, dir);
      fileTree.push(dir);
    }
    dir.children.push(child);
  }

  for (const dir of groups.values()) {
    dir.children = createFileTree(dir.children as FileNode[]);
  }

  return fileTree;
}
