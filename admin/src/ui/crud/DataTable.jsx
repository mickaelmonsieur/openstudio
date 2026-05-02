export function DataTable({
  columns,
  rows,
  primaryKey,
  selectedId,
  onEdit,
  onDelete,
  onSelect
}) {
  return (
    <div className="data-table-wrap">
      <table className="data-table">
        <thead>
          <tr>
            {columns.map((column) => (
              <th key={column.key} style={{ width: column.width }}>
                {column.label}
              </th>
            ))}
            <th className="actions-column">Actions</th>
          </tr>
        </thead>
        <tbody>
          {rows.length === 0 ? (
            <tr>
              <td className="empty-cell" colSpan={columns.length + 1}>
                No records.
              </td>
            </tr>
          ) : (
            rows.map((row) => (
              <tr
                className={selectedId === row[primaryKey] ? 'selected-row' : ''}
                key={row[primaryKey]}
                onClick={() => onSelect?.(row)}
              >
                {columns.map((column) => (
                  <td key={column.key}>{formatCell(row[column.key])}</td>
                ))}
                <td className="row-actions">
                  <button
                    className="ghost-button"
                    onClick={(event) => {
                      event.stopPropagation();
                      onEdit(row);
                    }}
                    type="button"
                  >
                    Edit
                  </button>
                  <button
                    className="danger-button"
                    onClick={(event) => {
                      event.stopPropagation();
                      onDelete(row);
                    }}
                    type="button"
                  >
                    Delete
                  </button>
                </td>
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}

function formatCell(value) {
  if (value === null || value === undefined || value === '') {
    return '—';
  }

  if (typeof value === 'boolean') {
    return value ? 'Yes' : 'No';
  }

  return String(value);
}
