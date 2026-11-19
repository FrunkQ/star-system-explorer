// Per-device conveniences only (the chosen mode). Never universe state.
export function loadPref(key: string): string | null {
  try {
    return localStorage.getItem(`sse4.${key}`);
  } catch {
    return null;
  }
}

export function savePref(key: string, value: string): void {
  try {
    localStorage.setItem(`sse4.${key}`, value);
  } catch {
    /* private window or blocked storage: the choice just is not remembered */
  }
}
