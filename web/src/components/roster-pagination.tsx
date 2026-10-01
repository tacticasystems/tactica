import { ChevronLeft, ChevronRight } from "lucide-react";

import { Button } from "./ui/button";
import { Pagination, PaginationContent, PaginationItem } from "./ui/pagination";

export function RosterPagination({
  offset,
  count,
  total,
  pending,
  onPageChange,
}: {
  offset: number;
  count: number;
  total: number;
  pending: boolean;
  onPageChange: (offset: number) => void;
}) {
  return (
    <Pagination className="pagination" aria-label="Personnel pages">
      <span>
        Showing {offset + (count ? 1 : 0)}–{offset + count} of {total}
      </span>
      <PaginationContent>
        <PaginationItem>
          <Button
            variant="outline"
            disabled={offset === 0 || pending}
            onClick={() => onPageChange(Math.max(0, offset - 20))}
          >
            <ChevronLeft size={16} /> Previous
          </Button>
        </PaginationItem>
        <PaginationItem>
          <Button
            variant="outline"
            disabled={count < 20 || offset + count >= total || pending}
            onClick={() => onPageChange(offset + 20)}
          >
            Next <ChevronRight size={16} />
          </Button>
        </PaginationItem>
      </PaginationContent>
    </Pagination>
  );
}
