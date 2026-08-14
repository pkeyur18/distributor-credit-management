// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter, Routes, Route, useNavigate } from "react-router";

import { NavigationHistoryProvider, useRouteLabel, useBackTarget } from "./navigation-history";

function Home() {
  useRouteLabel("Home");
  const navigate = useNavigate();
  return (
    <div>
      <button onClick={() => navigate("/detail")}>Open detail</button>
      <button onClick={() => navigate("/detail-b")}>Open detail B</button>
    </div>
  );
}

function Detail() {
  useRouteLabel("Detail");
  const { label, go } = useBackTarget();
  return <button onClick={go}>Back to {label}</button>;
}

function DetailB() {
  useRouteLabel("Detail B");
  const { label, go } = useBackTarget();
  const navigate = useNavigate();
  return (
    <div>
      <button onClick={() => navigate("/detail-b-2", { replace: true })}>Rename</button>
      <button onClick={go}>Back to {label}</button>
    </div>
  );
}

function DetailB2() {
  useRouteLabel("Detail B renamed");
  const { label, go } = useBackTarget();
  return <button onClick={go}>Back to {label}</button>;
}

function Reports() {
  const { label } = useBackTarget();
  return <span>Back to {label}</span>;
}

function harness(initialPath: string) {
  return render(
    <MemoryRouter initialEntries={[initialPath]}>
      <NavigationHistoryProvider>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/detail" element={<Detail />} />
          <Route path="/detail-b" element={<DetailB />} />
          <Route path="/detail-b-2" element={<DetailB2 />} />
          <Route path="/reports" element={<Reports />} />
        </Routes>
      </NavigationHistoryProvider>
    </MemoryRouter>,
  );
}

describe("navigation-history", () => {
  it("labels the back link with the screen that pushed the navigation", async () => {
    harness("/");
    fireEvent.click(screen.getByText("Open detail"));
    expect(await screen.findByText("Back to Home")).toBeInTheDocument();
  });

  it("pops back to the previous screen when the back link is clicked", async () => {
    harness("/");
    fireEvent.click(screen.getByText("Open detail"));
    fireEvent.click(await screen.findByText("Back to Home"));
    expect(await screen.findByText("Open detail")).toBeInTheDocument();
  });

  it("falls back to the static route label when nothing has been pushed yet", () => {
    harness("/reports");
    expect(screen.getByText("Back to Home")).toBeInTheDocument();
  });

  it("does not disturb the back target on a replace navigation", async () => {
    harness("/");
    fireEvent.click(screen.getByText("Open detail B"));
    fireEvent.click(await screen.findByText("Rename"));
    // still on the replaced screen, back target is still Home (the PUSH
    // that got us to /detail-b in the first place), not /detail-b itself.
    expect(await screen.findByText("Back to Home")).toBeInTheDocument();
  });
});
