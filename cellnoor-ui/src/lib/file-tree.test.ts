import { expect, test } from "bun:test";
import { createFileTree, type DirectoryNode, type FileNode } from "./file-tree";

test("file-tree nesting", () => {
  const files: FileNode[] = [
    {
      name: "per_sample_outs/sample1/metrics_summary.csv",
      src: "per_sample_outs/sample1/metrics_summary.csv",
      content: {},
      type: "json",
    },
    {
      name: "per_sample_outs/sample1/web_summary.html",
      src: "per_sample_outs/sample1/web_summary.html",
      type: "html",
    },
    {
      name: "per_sample_outs/sample2/metrics_summary.csv",
      src: "per_sample_outs/sample2/metrics_summary.csv",
      content: {},
      type: "json",
    },
    {
      name: "per_sample_outs/sample2/web_summary.html",
      src: "per_sample_outs/sample2/web_summary.html",
      type: "html",
    },
    {
      name: "qc_report.html",
      src: "qc_report.html",
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
            {
              name: "metrics_summary.csv",
              src: "per_sample_outs/sample1/metrics_summary.csv",
              content: {},
              type: "json",
            },
            {
              name: "web_summary.html",
              src: "per_sample_outs/sample1/web_summary.html",
              type: "html",
            },
          ],
        },
        {
          name: "sample2",
          children: [
            {
              name: "metrics_summary.csv",
              src: "per_sample_outs/sample2/metrics_summary.csv",
              content: {},
              type: "json",
            },
            {
              name: "web_summary.html",
              src: "per_sample_outs/sample2/web_summary.html",
              type: "html",
            },
          ],
        },
      ],
    },
    { name: "qc_report.html", src: "qc_report.html", type: "html" },
  ];

  expect(createdFileTree).toStrictEqual(expectedFileTree);
});
