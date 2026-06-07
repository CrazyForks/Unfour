export const defaultSql =
  "select name, type\nfrom sqlite_master\nwhere type in ('table', 'view')\nlimit 100;";

export const databaseKnownGaps = [
  "Monaco Editor theme is not fully tokenized yet.",
  "Table filtering, cell editing, and export paging are extension points only.",
  "DDL, index, constraint, and execution-plan data are unavailable until backend support lands.",
];
