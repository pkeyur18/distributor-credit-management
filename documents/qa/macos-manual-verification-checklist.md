# macOS manual verification checklist

`T-QA.3-3` / `T-REL.5-4` (PI/01-backlog.md, PI/02-roadmap.md PR-2). `tauri-driver`
drives WebView2 (Windows) and webkit2gtk (Linux) only — WKWebView exposes no
WebDriver, so macOS has no automated E2E coverage (D-8). This checklist is
that platform's actual verification, not a nice-to-have: run it in full on
every release candidate (`US-REL.2`), and after any change touching a
screen it covers.

Record the date, build version, and macOS version at the top of a copy of
this checklist each time it's run, and keep the completed copy — this is
the evidence the release gate asks for.

Steps grow as each story ships. Currently covers S1–S7 (through
US-M2.1/M2.2/M8.3, US-QA.3).

---

## 1. First run and setup (US-M8.1)

- [ ] Launch a build against a fresh app-data directory. The Setup screen appears, not Login.
- [ ] Choose PIN mode, enter a 6-digit PIN twice with a mismatch — refused with a clear message.
- [ ] Enter matching PINs, continue. Ten recovery codes are shown, each once.
- [ ] "Enter the console" stays disabled until the "I have saved these…" checkbox is ticked.
- [ ] After entering the console, relaunch the app — Login appears, not Setup (setup is not re-offered).

## 2. Login and lockout (US-M8.2)

- [ ] Log in with the correct PIN — reaches Home.
- [ ] Log out (sidebar), log in with a wrong PIN 5 times — locked out, countdown shown.
- [ ] Wait out the countdown (or use a short-tier test build) — the correct PIN now succeeds.
- [ ] Quit and relaunch the app mid-lockout — the lockout is still in effect (not reset by relaunch).
- [ ] Switch to Password mode on the login screen and confirm it authenticates identically.

## 3. Member onboarding and directory (US-M1.1–US-M1.4)

- [ ] Add the root member (Home → Add member, no Reference ID field for the first member).
- [ ] Add a second member with the root as Reference ID — resolves and saves.
- [ ] Try adding a member with a phone number already in use by an active member — refused with a named reason.
- [ ] Deactivate a member — a distinct colour and pill appear everywhere they're listed.
- [ ] Reactivate the same member — original ID and history are unchanged, no duplicate created.
- [ ] Search by partial name, by 6-digit ID, and by a phone number typed with and without `+91`/leading zero — all three find the same member.

## 4. Business Volume entry (US-M2.1)

- [ ] Business Volume Entry screen, search for a member, select them.
- [ ] Date field is bounded to the current month (cannot pick a future date, cannot pick a date before this month started).
- [ ] Save with amount `0` — refused. Save with a negative amount — refused (field cannot go negative).
- [ ] Save a valid entry (e.g. `1000.00`) — a success toast appears and the entry shows in the "Recorded this session" list.
- [ ] No "recalculate" button exists anywhere on this screen or elsewhere in the app (Rule-26).

## 5. Closed-month correction (US-M2.2)

> Until US-M5.1 (S11) ships, no period ever actually reaches `closed` through the UI — this section is testable only against a manually-prepared database until then. Skip with a note until S11 lands, don't mark it passed.

- [ ] Correction Panel: entering an out-of-range or non-numeric Entry ID and saving is refused, not silently accepted.
- [ ] Correcting a real closed-period entry ID with a valid amount/date succeeds and the confirmation line shows the corrected figure.
- [ ] Attempting to move the entry's date into a different month is refused, naming the reason.

## 6. Session lock (US-M8.3)

- [ ] Sidebar → "Lock session" — immediately returns to the Locked screen (PIN/password entry), not the plain Login screen.
- [ ] On the Locked screen, the correct credential returns to exactly the screen/state left before locking.
- [ ] A wrong credential on the Locked screen counts toward the same lockout ladder as Login (verify via the attempts-remaining/lockout message).
- [ ] Leave the app idle for the configured inactivity timeout (15 minutes by default) — the app locks on its own, without any click.
- [ ] From the Locked screen, "Sign out instead" returns to the plain Login screen (not Locked) on next launch of that flow.
- [ ] After locking, quit and relaunch the app — Login (not Setup) appears, and no data has been lost.

## 7. General

- [ ] Both light and dark theme render correctly on every screen touched above (sidebar theme toggle).
- [ ] No console errors in Safari's Web Inspector (attach via `Develop → [device] → localhost` if enabled) during the flows above.
- [ ] Window resizes cleanly down to the design's minimum supported width without overlapping content.
