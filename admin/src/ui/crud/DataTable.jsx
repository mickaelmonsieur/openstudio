import { useState } from 'react';

export function DataTable({
  columns,
  rows,
  primaryKey,
  selectedId,
  onEdit,
  onDelete,
  onSelect,
  reorderable = false,
  onReorder,
  renderRowActions,
  isRowLocked
}) {
  const [draggingId, setDraggingId] = useState(null);

  function moveRow(targetRow) {
    if (!reorderable || !onReorder || !draggingId) return;
    const targetId = targetRow[primaryKey];
    if (targetId === draggingId) return;

    const fromIndex = rows.findIndex((row) => row[primaryKey] === draggingId);
    const toIndex = rows.findIndex((row) => row[primaryKey] === targetId);
    if (fromIndex < 0 || toIndex < 0) return;

    const nextRows = [...rows];
    const [moved] = nextRows.splice(fromIndex, 1);
    nextRows.splice(toIndex, 0, moved);
    onReorder(nextRows);
  }

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
            rows.map((row, rowIndex) => (
              <tr
                className={[
                  selectedId === row[primaryKey] ? 'selected-row' : '',
                  reorderable ? 'draggable-row' : '',
                  draggingId === row[primaryKey] ? 'dragging-row' : ''
                ].filter(Boolean).join(' ')}
                draggable={reorderable}
                key={row[primaryKey]}
                onClick={() => onSelect?.(row)}
                onDragEnd={() => setDraggingId(null)}
                onDragOver={(event) => {
                  if (!reorderable) return;
                  event.preventDefault();
                }}
                onDragStart={(event) => {
                  if (!reorderable) return;
                  setDraggingId(row[primaryKey]);
                  event.dataTransfer.effectAllowed = 'move';
                  event.dataTransfer.setData('text/plain', String(row[primaryKey]));
                }}
                onDrop={(event) => {
                  if (!reorderable) return;
                  event.preventDefault();
                  moveRow(row);
                  setDraggingId(null);
                }}
              >
                {columns.map((column) => (
                  <td key={column.key}>{formatCell(row[column.key])}</td>
                ))}
                <td className="row-actions">
                  {renderRowActions?.(row, rowIndex)}
                  {!isRowLocked?.(row) && (
                    <>
                      <button
                        aria-label="Edit"
                        className="ghost-button table-icon-button"
                        onClick={(event) => { event.stopPropagation(); onEdit(row); }}
                        title="Edit"
                        type="button"
                      >
                        <i className="bi bi-pencil" aria-hidden="true" />
                      </button>
                      <button
                        aria-label="Delete"
                        className="danger-button table-icon-button"
                        onClick={(event) => { event.stopPropagation(); onDelete(row); }}
                        title="Delete"
                        type="button"
                      >
                        <i className="bi bi-trash" aria-hidden="true" />
                      </button>
                    </>
                  )}
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
