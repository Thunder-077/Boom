export interface PageQuery {
  page?: number;
  pageSize?: number;
}

export type FilterState = Record<string, string | number | undefined>;

export interface ListResult<T> {
  items: T[];
  total: number;
}
