import Button from "../base/Button";

interface PaginationProps {
  currentPage?: number;
  pageSize?: number;
  total?: number;
  onChange?: (page: number) => void;
}

export default function Pagination({ currentPage = 1, pageSize = 10, total = 0, onChange }: PaginationProps) {
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  if (totalPages <= 1) {
    return null;
  }

  const infoText =
    total === 0
      ? "共 0 条"
      : `共 ${total} 条，本页 ${(currentPage - 1) * pageSize + 1} - ${Math.min(currentPage * pageSize, total)}`;
  const visiblePages = Array.from(new Set([1, Math.max(1, currentPage - 1), currentPage, Math.min(totalPages, currentPage + 1), totalPages]))
    .filter((page) => page >= 1 && page <= totalPages)
    .sort((a, b) => a - b);

  return (
    <div className="pagination">
      <span className="pagination-info">{infoText}</span>
      <div className="pagination-actions">
        <Button variant="ghost" size="sm" disabled={currentPage === 1} onClick={() => onChange?.(currentPage - 1)}>
          <span className="material-symbols-rounded">chevron_left</span>
          上一页
        </Button>
        {visiblePages.map((page) => (
          <button key={page} className={`page-btn ${page === currentPage ? "active" : ""}`} type="button" onClick={() => onChange?.(page)}>
            {page}
          </button>
        ))}
        <Button variant="ghost" size="sm" disabled={currentPage === totalPages} onClick={() => onChange?.(currentPage + 1)}>
          下一页
          <span className="material-symbols-rounded">chevron_right</span>
        </Button>
      </div>
    </div>
  );
}
