/** Keeps asynchronously registered listeners scoped to their owner's lifetime. */
export function registerScopedListener<T>(
  register: (handler: (event: T) => void) => Promise<() => void>,
  handler: (event: T) => void,
): () => void {
  let disposed = false;
  let unlisten: (() => void) | undefined;

  void register((event) => {
    if (!disposed) handler(event);
  }).then((stop) => {
    if (disposed) stop();
    else unlisten = stop;
  });

  return () => {
    disposed = true;
    unlisten?.();
  };
}
