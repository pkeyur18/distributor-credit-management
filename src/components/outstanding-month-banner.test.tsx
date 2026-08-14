import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";

import { OutstandingMonthBanner } from "./outstanding-month-banner";

// Rule-20/T-M5.2-2/T-M5.2-6: no dismiss control of any kind, not even a
// disguised one. The full WebdriverIO variant this task calls for can't be
// reached the way this project's E2E harness is designed — there is no
// DB-seeding hook (`e2e/helpers/seed.js`'s own doc comment: every fixture
// is built by driving the real UI), and reaching `awaiting_close` requires
// a real calendar-month boundary to elapse. This component-level test is
// the practical substitute: it asserts the actual rendered DOM never
// carries a close icon, dismiss button or acknowledge action — only the
// one navigation link to the close screen.
describe("OutstandingMonthBanner", () => {
  it("renders nothing when there is no outstanding month", () => {
    const { container } = render(
      <MemoryRouter>
        <OutstandingMonthBanner alert={{ outstandingMonths: [], currentMonth: "2026-08" }} />
      </MemoryRouter>,
    );
    expect(container).toBeEmptyDOMElement();
  });

  it("has no dismiss, close or acknowledge control — only the Close-month link", () => {
    render(
      <MemoryRouter>
        <OutstandingMonthBanner
          alert={{ outstandingMonths: ["2026-06"], currentMonth: "2026-08" }}
        />
      </MemoryRouter>,
    );

    const buttons = screen.queryAllByRole("button");
    expect(buttons).toHaveLength(0);

    const links = screen.getAllByRole("link");
    expect(links).toHaveLength(1);
    expect(links[0]).toHaveTextContent("Close June 2026");
  });

  it("names every outstanding month, not only the oldest", () => {
    render(
      <MemoryRouter>
        <OutstandingMonthBanner
          alert={{ outstandingMonths: ["2026-05", "2026-06", "2026-07"], currentMonth: "2026-08" }}
        />
      </MemoryRouter>,
    );

    expect(screen.getByText(/May 2026 has ended and is awaiting close\./)).toBeInTheDocument();
    expect(screen.getByText(/2 more months are outstanding after that\./)).toBeInTheDocument();
  });
});
