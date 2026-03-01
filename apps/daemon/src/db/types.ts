export interface PreparedStatement<Row = unknown> {
  run: (...params: unknown[]) => unknown;
  get: (...params: unknown[]) => Row | null;
  all: (...params: unknown[]) => Row[];
}

export interface DB {
  exec: (sql: string) => void;
  prepare: <Row = unknown>(sql: string) => PreparedStatement<Row>;
  close: () => void;
}
