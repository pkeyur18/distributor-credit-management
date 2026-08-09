import { invoke } from "@tauri-apps/api/core";
import { toErrorPresentation, type AppErrorPresentation } from "./errors";

/**
 * Single call-through point for every Tauri command wrapper in this
 * directory — never call `invoke` directly from a command module, so every
 * failure is normalized to an {@link AppErrorPresentation} in one place.
 */
export async function invokeCommand<Res>(
  command: string,
  payload?: Record<string, unknown>,
): Promise<Res> {
  try {
    return await invoke<Res>(command, payload);
  } catch (raw) {
    throw toErrorPresentation(raw) satisfies AppErrorPresentation;
  }
}
