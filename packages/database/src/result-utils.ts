import type { DatabaseQueryResult } from "@unfour/command-client";

export function serializeDatabaseResult(
  result: DatabaseQueryResult,
  delimiter: "," | "\t",
) {
  const header = result.columns
    .map((column) => serializeCell(column.name, delimiter))
    .join(delimiter);
  const rows = result.rows.map((row) =>
    result.columns
      .map((_, index) => serializeCell(row[index] ?? "", delimiter))
      .join(delimiter),
  );
  return [header, ...rows].join("\r\n");
}

function serializeCell(value: string, delimiter: "," | "\t") {
  const needsQuotes =
    value.includes(delimiter) ||
    value.includes("\"") ||
    value.includes("\n") ||
    value.includes("\r");
  if (!needsQuotes) {
    return value;
  }
  return `"${value.replace(/"/g, "\"\"")}"`;
}

export function isConfirmationRequired(error: unknown) {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    (error as { code: unknown }).code === "CONFIRMATION_REQUIRED"
  );
}

export function confirmationMessage(error: unknown) {
  if (typeof error === "object" && error !== null && "details" in error) {
    const details = (error as { details?: { classification?: unknown } }).details;
    if (details?.classification) {
      return `Confirmation required for ${String(details.classification)} SQL. Review the statement, then click Confirm run.`;
    }
  }
  return "Confirmation required. Review the SQL statement, then click Confirm run.";
}
