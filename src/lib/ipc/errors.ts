// T-UI.5-2: maps every typed Rust AppError kind to its user-facing presentation.
//
// `period_not_accepting_entries` and `period_closed` are reserved slots —
// no Rust command produces them yet (US-M2.4 lands S12/S13). They're typed
// here now so that story plugs into an existing pattern instead of inventing
// its own per-screen error handling.
//
// `validation`/`not_found`/`conflict`/`auth_required`/`not_implemented`
// land in S4 with the command-surface scaffold (error.rs, src-tauri) —
// see AppError there for the Rust side of each.
//
// The retired `period_locked` variant must never be reintroduced under a new
// meaning — it falls through to "unknown" like any other unrecognized kind.

export type AppErrorKind =
  | "database"
  | "io"
  | "period_not_accepting_entries"
  | "period_closed"
  | "validation"
  | "not_found"
  | "conflict"
  | "auth_required"
  | "not_implemented";

export interface AppErrorPresentation {
  kind: AppErrorKind | "unknown";
  message: string;
  month?: string;
  blockingMonth?: string;
  field?: string;
}

interface RawAppError {
  kind?: string;
  message?: string;
  month?: string;
  blockingMonth?: string;
  field?: string;
}

const PRESENTATIONS: Record<AppErrorKind, (raw: RawAppError) => string> = {
  database: () => "Something went wrong saving that. Try again.",
  io: () => "Something went wrong reading a file. Try again.",
  period_not_accepting_entries: (raw) =>
    `${raw.month ?? "That month"} isn't open for entry until ${
      raw.blockingMonth ?? "the outstanding month"
    } is closed.`,
  period_closed: (raw) =>
    `${raw.month ?? "That month"} is closed — use the correction panel instead.`,
  validation: (raw) => raw.message ?? "Check the highlighted field and try again.",
  not_found: (raw) => raw.message ?? "That couldn't be found.",
  conflict: (raw) => raw.message ?? "That conflicts with an existing record.",
  auth_required: () => "Sign in to do that.",
  not_implemented: () => "This isn't built yet.",
};

function isKnownKind(kind: string | undefined): kind is AppErrorKind {
  return kind !== undefined && kind in PRESENTATIONS;
}

export function toErrorPresentation(raw: unknown): AppErrorPresentation {
  const err: RawAppError = typeof raw === "object" && raw !== null ? raw : {};

  if (!isKnownKind(err.kind)) {
    return {
      kind: "unknown",
      message: err.message ?? "Something went wrong.",
    };
  }

  return {
    kind: err.kind,
    message: PRESENTATIONS[err.kind](err),
    month: err.month,
    blockingMonth: err.blockingMonth,
    field: err.field,
  };
}
