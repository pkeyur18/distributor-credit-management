import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";
import { applyTheme, readStoredTheme } from "@/lib/theme";

// Auth screens (login, setup, ...) render standalone, outside AppShell, so
// sidebar.tsx's useTheme never mounts for them — apply the stored
// preference here, before first paint, so it's not just the post-login
// shell that respects it.
applyTheme(document.documentElement, readStoredTheme(window.localStorage));

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
