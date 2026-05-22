import type { StoreApi } from "zustand/vanilla";

type PropertyKeyPath = Array<string | number | symbol>;

export function createMutableZustandState<State extends object>(store: StoreApi<State>): State {
  function valueAt(path: PropertyKeyPath) {
    return path.reduce<unknown>((value, key) => (value as Record<PropertyKey, unknown>)[key], store.getState());
  }

  function cloneWith(path: PropertyKeyPath, key: string | number | symbol, value: unknown) {
    const root = store.getState();
    if (path.length === 0) {
      return {
        ...root,
        [key]: value,
      };
    }

    const nextRoot = Array.isArray(root) ? [...root] : { ...root };
    let sourceCursor: unknown = root;
    let targetCursor: unknown = nextRoot;
    for (const pathKey of path) {
      const sourceChild = (sourceCursor as Record<PropertyKey, unknown>)[pathKey];
      const nextChild = Array.isArray(sourceChild) ? [...sourceChild] : { ...(sourceChild as object) };
      (targetCursor as Record<PropertyKey, unknown>)[pathKey] = nextChild;
      sourceCursor = sourceChild;
      targetCursor = nextChild;
    }
    (targetCursor as Record<PropertyKey, unknown>)[key] = value;
    return nextRoot as State;
  }

  function deleteWith(path: PropertyKeyPath, key: string | number | symbol) {
    const parent = valueAt(path);
    if (!parent || typeof parent !== "object") {
      return store.getState();
    }
    const nextParent = Array.isArray(parent) ? [...parent] : { ...(parent as object) };
    delete (nextParent as Record<PropertyKey, unknown>)[key];
    return path.length === 0
      ? (nextParent as State)
      : cloneWith(path.slice(0, -1), path[path.length - 1], nextParent);
  }

  function proxyAt(path: PropertyKeyPath): unknown {
    return new Proxy(
      {},
      {
        get(_target, key) {
          const value = (valueAt(path) as Record<PropertyKey, unknown>)[key];
          if (typeof value === "function") {
            return value.bind(valueAt(path));
          }
          if (value && typeof value === "object") {
            return proxyAt([...path, key]);
          }
          return value;
        },
        set(_target, key, value) {
          store.setState(cloneWith(path, key, value), true);
          return true;
        },
        deleteProperty(_target, key) {
          store.setState(deleteWith(path, key), true);
          return true;
        },
        ownKeys() {
          return Reflect.ownKeys(valueAt(path) as object);
        },
        getOwnPropertyDescriptor(_target, key) {
          const parent = valueAt(path) as object;
          return {
            configurable: true,
            enumerable: true,
            writable: true,
            value: (parent as Record<PropertyKey, unknown>)[key],
          };
        },
      },
    );
  }

  return proxyAt([]) as State;
}
