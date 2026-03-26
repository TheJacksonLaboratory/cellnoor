import { expect, test } from "bun:test";
import { createFileTree, type DirectoryNode, type FileNode } from "./file-tree";

test("file-tree nesting", () => {
  const files: FileNode[] = [
    {
      name: "per_sample_outs/sample1/metrics_summary.csv",
      content: null,
      type: "json",
    },
    {
      name: "per_sample_outs/sample1/web_summary.html",
      content: null,
      type: "html",
    },
    {
      name: "per_sample_outs/sample2/metrics_summary.csv",
      content: null,
      type: "json",
    },
    {
      name: "per_sample_outs/sample2/web_summary.html",
      content: null,
      type: "html",
    },
    {
      name: "qc_report.html",
      content: null,
      type: "html",
    },
  ];

  const createdFileTree = createFileTree(files);

  const expectedFileTree: (DirectoryNode | FileNode)[] = [
    {
      name: "per_sample_outs",
      children: [
        {
          name: "sample1",
          children: [
            { name: "metrics_summary.csv", content: null, type: "json" },
            { name: "web_summary.html", content: null, type: "html" },
          ],
        },
        {
          name: "sample2",
          children: [
            { name: "metrics_summary.csv", content: null, type: "json" },
            { name: "web_summary.html", content: null, type: "html" },
          ],
        },
      ],
    },
    { name: "qc_report.html", content: null, type: "html" },
  ];

  expect(createdFileTree).toStrictEqual(expectedFileTree);
});
